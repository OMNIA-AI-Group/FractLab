//! Requêtes et réponses des calculs de fond, sous forme transportable
//! (sérialisée en binaire) : c'est ce qui circule entre l'interface et le
//! Web Worker dans la version navigateur. Sur ordinateur, les mêmes calculs
//! tournent dans des fils, sans passer par ici.

use crate::compute::{self, ResultsWire, WireMsg};
use crate::dynsim::{SimResult, SimSetup};
use crate::geometry::MotorSpec;
use crate::optimize::{self, OptMsg};
use crate::recalage::{self, FitMsg};
use std::sync::atomic::AtomicBool;

/// Requête envoyée au Worker.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Request {
    /// Calcul (principal, comparaison, cartes, validation, étude paramétrique).
    Compute(compute::Request),
    /// Ajustement I.A.
    Opt(MotorSpec, optimize::Setup),
    /// Recalage sur mesures.
    Fit(MotorSpec, recalage::FitSetup),
    /// Simulation dynamique.
    Sim(ResultsWire, SimSetup),
    /// Script de la console (projet, réglages, texte, langue des messages).
    Script(MotorSpec, compute::Settings, String, crate::i18n::Lang),
}

/// Réponse renvoyée par le Worker (plusieurs par requête : avancement, résultat).
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Reply {
    /// Message de calcul.
    Msg(WireMsg),
    /// Message de l'ajustement.
    Opt(OptMsg),
    /// Message du recalage.
    Fit(FitMsg),
    /// Résultat de simulation.
    Sim(SimResult),
    /// Message de la console de script.
    Script(crate::script::ScriptMsg),
}

/// Exécute une requête ; chaque réponse est passée à `send` dès qu'elle est prête.
pub fn execute(req: Request, send: &mut dyn FnMut(Reply)) {
    let stop = AtomicBool::new(false);
    match req {
        Request::Compute(r) => compute::execute(r, &mut |m| send(Reply::Msg(m.to_wire()))),
        Request::Opt(base, st) => optimize::execute(base, st, &stop, &mut |m| send(Reply::Opt(m))),
        Request::Fit(base, setup) => recalage::execute(base, setup, &stop, &mut |m| send(Reply::Fit(m))),
        Request::Sim(w, setup) => match w.into_results() {
            Ok(r) => send(Reply::Sim(crate::dynsim::simulate(&r, &setup))),
            Err(_) => send(Reply::Sim(SimResult::default())),
        },
        Request::Script(spec, st, src, lang) => {
            crate::i18n::set_lang(lang);
            crate::script::execute(spec, st, &src, &stop, &mut |m| send(Reply::Script(m)))
        }
    }
}

/// Encodage binaire.
pub fn encode<T: serde::Serialize>(v: &T) -> Vec<u8> {
    bincode::serialize(v).unwrap_or_default()
}

/// Décodage binaire.
pub fn decode<T: serde::de::DeserializeOwned>(b: &[u8]) -> Option<T> {
    bincode::deserialize(b).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::{Msg, Settings};

    /// Aller-retour complet comme dans le navigateur : requête encodée,
    /// exécutée, réponses encodées puis décodées ; les résultats sont
    /// identiques à ceux du calcul direct.
    #[test]
    fn aller_retour_identique() {
        let st = Settings { n_cog: 90, n_emf: 90, ..Default::default() };
        let spec = MotorSpec::irregular_example();
        let req: Request = decode(&encode(&Request::Compute(compute::Request::Main(spec.clone(), st)))).unwrap();
        let mut replies = Vec::new();
        execute(req, &mut |r| replies.push(encode(&r)));
        assert_eq!(replies.len(), 1);
        let Some(Reply::Msg(w)) = decode::<Reply>(&replies[0]) else { panic!("réponse illisible") };
        let Msg::Main(Ok(r)) = w.into_msg() else { panic!("pas de résultat") };
        let direct = compute::compute_results(spec, &st).unwrap();
        assert_eq!(r.report.cogging.cogging_pp_mnm, direct.report.cogging.cogging_pp_mnm);
        assert_eq!(r.emf.emf[0], direct.emf.emf[0]);
        // le modèle reconstruit donne le même champ
        let a = r.model.torque(&r.model.solve(0.3));
        let b = direct.model.torque(&direct.model.solve(0.3));
        assert!((a - b).abs() < 1e-12);
        // simulation par le même chemin
        let mut got = None;
        execute(decode(&encode(&Request::Sim(direct.to_wire(), SimSetup { duration: 0.02, ..Default::default() }))).unwrap(), &mut |r| got = Some(encode(&r)));
        assert!(matches!(decode::<Reply>(&got.unwrap()), Some(Reply::Sim(s)) if !s.t.is_empty()));
    }
}
