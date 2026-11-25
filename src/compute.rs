//! Calculs en tâche de fond (l'interface reste fluide pendant les balayages).

use crate::analysis::{self, CoggingCurve, EmfCurve, ForceCurve, Report};
use crate::geometry::MotorSpec;
use crate::subdomain::{Model, Resolution};
use crate::view3d::Surface;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use web_time::Instant;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
/// Réglages de calcul choisis dans l'interface.
pub struct Settings {
    /// Résolution harmonique.
    pub res: Resolution,
    /// Nombre de positions pour le cogging.
    pub n_cog: usize,
    /// Nombre de positions pour la FEM (sur un tour).
    pub n_emf: usize,
    /// fenêtre commune de comparaison du cogging [°] (0 = période propre)
    pub common_span_deg: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { res: Resolution::default(), n_cog: 720, n_emf: 720, common_span_deg: 30.0 }
    }
}

/// Résultats complets du calcul principal.
pub struct Results {
    /// Spécification calculée.
    pub spec: MotorSpec,
    /// Modèle (réutilisé pour les vues interactives).
    pub model: Arc<Model>,
    /// Couple de détente.
    pub cog: CoggingCurve,
    /// FEM à vide.
    pub emf: EmfCurve,
    /// Forces radiales.
    pub forces: ForceCurve,
    /// Rapport de synthèse.
    pub report: Report,
    /// Induction dans les dents et la culasse sur un tour.
    pub iron: crate::dynamics::IronSweep,
    /// Fonctionnement en charge.
    pub load: crate::analysis::LoadCurve,
    /// Saturation magnétique (facteur de saturation).
    pub sat: crate::saturation::SatCurve,
    /// Durée du calcul [s].
    pub seconds: f64,
}

/// Résultat d'une variante dans la comparaison.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VariantResult {
    /// Nom de la variante.
    pub name: String,
    /// Couple de détente.
    pub cog: CoggingCurve,
    /// Rapport de synthèse.
    pub report: Report,
    /// Fonctionnement en charge selon la commande.
    #[serde(default)]
    pub drive: VariantDrive,
}

/// Couple moyen et ondulation d'une variante selon la commande.
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct VariantDrive {
    /// Couple moyen en courants sinusoïdaux [N·m].
    pub torque_sin: f64,
    /// Ondulation crête-à-crête en courants sinusoïdaux [%].
    pub ripple_sin: f64,
    /// Couple moyen en commutation trapézoïdale 120° [N·m].
    pub torque_trap: f64,
    /// Ondulation crête-à-crête en commutation trapézoïdale [%].
    pub ripple_trap: f64,
}

/// Messages renvoyés par les tâches de fond.
pub enum Msg {
    /// Résultat (ou erreur) du calcul principal.
    Main(Result<Results, String>),
    /// Résultats de la comparaison des variantes.
    Compare(Vec<VariantResult>),
    /// Carte 3D calculée (identifiant, surface).
    Surface(String, Surface),
    /// Message d'avancement.
    Progress(String),
    /// Validation : (θ°, analytique mN·m, numérique mN·m) et ouverture alignée [mm].
    Validation(Vec<(f64, f64, f64)>, f64),
    /// Étude paramétrique : un point (valeur du paramètre, sorties).
    SweepRow(f64, Vec<f64>),
    /// Étude paramétrique terminée.
    SweepDone,
    /// Comparaison de solutions : résultat complet de la solution n° k.
    Solution(usize, Box<ResultsWire>),
    /// Comparaison de solutions : la solution n° k ne se calcule pas.
    SolutionError(usize, String),
    /// Comparaison de solutions terminée.
    SolutionsDone,
}

/// Tâche de fond en cours.
pub struct Job {
    /// Canal de réception des messages.
    pub rx: Receiver<Msg>,
    /// Web Worker qui exécute la tâche (navigateur) ; arrêté avec la tâche.
    pub _worker: Option<WorkerHandle>,
}

/// Web Worker d'une tâche (rien sur ordinateur).
#[cfg(target_arch = "wasm32")]
pub type WorkerHandle = crate::web::WorkerGuard;
/// Web Worker d'une tâche (rien sur ordinateur).
#[cfg(not(target_arch = "wasm32"))]
pub type WorkerHandle = ();

fn span(spec: &MotorSpec, st: &Settings) -> f64 {
    if st.common_span_deg > 0.0 { st.common_span_deg } else { spec.cogging_period_deg() }
}

impl Results {
    /// Copie légère (le modèle est partagé) pour les calculs de fond.
    pub fn light_clone(&self) -> Results {
        Results {
            spec: self.spec.clone(),
            model: self.model.clone(),
            cog: crate::analysis::CoggingCurve { angles_deg: self.cog.angles_deg.clone(), torque_nm: self.cog.torque_nm.clone(), spectrum: self.cog.spectrum.clone(), metrics: self.cog.metrics.clone() },
            emf: self.emf.clone(),
            forces: self.forces.clone(),
            report: self.report.clone(),
            iron: self.iron.clone(),
            load: self.load.clone(),
            sat: self.sat.clone(),
            seconds: self.seconds,
        }
    }
}

/// Calcul complet d'un moteur (cogging, FEM, forces, pertes fer, charge, rapport).
pub fn compute_results(spec: MotorSpec, st: &Settings) -> Result<Results, String> {
    let t = Instant::now();
    if let Some(e) = spec.geometry_problem() {
        return Err(e);
    }
    let model = Arc::new(Model::new(&spec, st.res)?);
    let sp = span(&spec, st);
    let cog = analysis::cogging(&model, sp, st.n_cog);
    let emf = analysis::emf(&model, st.n_emf);
    let forces = analysis::forces(&model, 360.0, 360);
    let iron = crate::dynamics::iron_sweep(&model, 360);
    let load = analysis::load(&model, &emf, &cog, 120);
    let sat = crate::saturation::analyse(&model, &emf, load.avg_torque);
    let mut report = analysis::report_from(&spec, &cog, &emf, &forces);
    report.variant = spec.name.clone();
    Ok(Results { spec, model, cog, emf, forces, report, iron, load, sat, seconds: t.elapsed().as_secs_f64() })
}

/// Requête de calcul de fond. Sur ordinateur elle s'exécute dans un fil ;
/// dans le navigateur, dans un Web Worker (l'interface ne gèle jamais).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Request {
    /// Calcul principal.
    Main(MotorSpec, Settings),
    /// Comparaison des variantes.
    Compare(MotorSpec, Settings),
    /// Carte 3D cogging × amplitude des décalages : (amplitude max, nombre d'amplitudes, positions).
    SurfaceAmplitude(MotorSpec, Settings, f64, usize, usize),
    /// Carte 3D Br(θ, θr) : (points en θ, positions, étendue [°]).
    SurfaceBr(MotorSpec, Settings, usize, usize, f64),
    /// Validation par volumes finis : (cellules en θ, positions).
    Validation(MotorSpec, Settings, usize, usize),
    /// Étude paramétrique : (paramètre, de, à, points).
    Sweep(MotorSpec, Settings, crate::sweep::SweepVar, f64, f64, usize),
    /// Comparaison de solutions : chaque projet en calcul complet.
    Solutions(Vec<MotorSpec>, Settings),
}

/// Résultats sans le modèle, pour passer d'un Web Worker à l'interface (le
/// modèle est reconstruit à l'arrivée).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ResultsWire {
    /// Spécification calculée.
    pub spec: MotorSpec,
    /// Résolution du modèle.
    pub res: Resolution,
    /// Couple de détente.
    pub cog: CoggingCurve,
    /// FEM à vide.
    pub emf: EmfCurve,
    /// Forces radiales.
    pub forces: ForceCurve,
    /// Rapport.
    pub report: Report,
    /// Inductions fer.
    pub iron: crate::dynamics::IronSweep,
    /// Charge.
    pub load: crate::analysis::LoadCurve,
    /// Saturation.
    pub sat: crate::saturation::SatCurve,
    /// Durée [s].
    pub seconds: f64,
}

impl Results {
    /// Version transportable (sans le modèle).
    pub fn to_wire(&self) -> ResultsWire {
        ResultsWire {
            spec: self.spec.clone(),
            res: self.model.res,
            cog: self.cog.clone(),
            emf: self.emf.clone(),
            forces: self.forces.clone(),
            report: self.report.clone(),
            iron: self.iron.clone(),
            load: self.load.clone(),
            sat: self.sat.clone(),
            seconds: self.seconds,
        }
    }
}

impl ResultsWire {
    /// Reconstruit les résultats complets (le modèle est recalculé).
    pub fn into_results(self) -> Result<Results, String> {
        let model = Arc::new(Model::new(&self.spec, self.res)?);
        Ok(Results { spec: self.spec, model, cog: self.cog, emf: self.emf, forces: self.forces, report: self.report, iron: self.iron, load: self.load, sat: self.sat, seconds: self.seconds })
    }
}

/// Message transportable (même contenu que `Msg`, résultats sans modèle).
#[derive(serde::Serialize, serde::Deserialize)]
pub enum WireMsg {
    /// Calcul principal.
    Main(Result<ResultsWire, String>),
    /// Autre message, inchangé.
    Other(OtherMsg),
}

/// Messages qui ne portent pas de modèle.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum OtherMsg {
    /// Comparaison.
    Compare(Vec<VariantResult>),
    /// Carte 3D.
    Surface(String, Surface),
    /// Avancement.
    Progress(String),
    /// Validation.
    Validation(Vec<(f64, f64, f64)>, f64),
    /// Point d'étude paramétrique.
    SweepRow(f64, Vec<f64>),
    /// Étude terminée.
    SweepDone,
    /// Solution calculée.
    Solution(usize, Box<ResultsWire>),
    /// Solution impossible.
    SolutionError(usize, String),
    /// Comparaison de solutions terminée.
    SolutionsDone,
}

impl Msg {
    /// Version transportable.
    pub fn to_wire(self) -> WireMsg {
        match self {
            Msg::Main(r) => WireMsg::Main(r.map(|r| r.to_wire())),
            Msg::Compare(v) => WireMsg::Other(OtherMsg::Compare(v)),
            Msg::Surface(k, s) => WireMsg::Other(OtherMsg::Surface(k, s)),
            Msg::Progress(p) => WireMsg::Other(OtherMsg::Progress(p)),
            Msg::Validation(r, o) => WireMsg::Other(OtherMsg::Validation(r, o)),
            Msg::SweepRow(v, o) => WireMsg::Other(OtherMsg::SweepRow(v, o)),
            Msg::SweepDone => WireMsg::Other(OtherMsg::SweepDone),
            Msg::Solution(k, w) => WireMsg::Other(OtherMsg::Solution(k, w)),
            Msg::SolutionError(k, e) => WireMsg::Other(OtherMsg::SolutionError(k, e)),
            Msg::SolutionsDone => WireMsg::Other(OtherMsg::SolutionsDone),
        }
    }
}

impl WireMsg {
    /// Message complet (le modèle du calcul principal est reconstruit).
    pub fn into_msg(self) -> Msg {
        match self {
            WireMsg::Main(r) => Msg::Main(r.and_then(|w| w.into_results())),
            WireMsg::Other(OtherMsg::Compare(v)) => Msg::Compare(v),
            WireMsg::Other(OtherMsg::Surface(k, s)) => Msg::Surface(k, s),
            WireMsg::Other(OtherMsg::Progress(p)) => Msg::Progress(p),
            WireMsg::Other(OtherMsg::Validation(r, o)) => Msg::Validation(r, o),
            WireMsg::Other(OtherMsg::SweepRow(v, o)) => Msg::SweepRow(v, o),
            WireMsg::Other(OtherMsg::SweepDone) => Msg::SweepDone,
            WireMsg::Other(OtherMsg::Solution(k, w)) => Msg::Solution(k, w),
            WireMsg::Other(OtherMsg::SolutionError(k, e)) => Msg::SolutionError(k, e),
            WireMsg::Other(OtherMsg::SolutionsDone) => Msg::SolutionsDone,
        }
    }
}

/// Exécute une requête ; chaque message est passé à `send` dès qu'il est prêt.
pub fn execute(req: Request, send: &mut dyn FnMut(Msg)) {
    match req {
        Request::Main(spec, st) => send(Msg::Main(compute_results(spec, &st))),
        Request::Sweep(base, st, var, from, to, n) => {
            let n = n.max(2);
            for k in 0..n {
                let v = from + (to - from) * k as f64 / (n - 1) as f64;
                let mut s = base.clone();
                var.set(&mut s, v);
                if let Ok(r) = compute_results(s, &st) {
                    send(Msg::SweepRow(v, crate::sweep::outputs(&r)));
                }
                send(Msg::Progress(format!("{} {}/{}", crate::i18n::tr("étude paramétrique"), k + 1, n)));
            }
            send(Msg::SweepDone);
        }
        Request::Solutions(specs, st) => {
            for (k, s) in specs.into_iter().enumerate() {
                match compute_results(s, &st) {
                    Ok(r) => send(Msg::Solution(k, Box::new(r.to_wire()))),
                    Err(e) => send(Msg::SolutionError(k, e)),
                }
            }
            send(Msg::SolutionsDone);
        }
        Request::Compare(base, st) => {
            let names = crate::repartition::variants();
            let out: Vec<Option<VariantResult>> = crate::par::map(names.len(), |i| {
                let mut s = base.clone();
                s.variant = names[i].to_string();
                s.name = names[i].to_string();
                s.custom_angles_deg = None;
                crate::repartition::on_select(&mut s);
                let m = Model::new(&s, st.res).ok()?;
                let sp = span(&s, &st);
                let cog = analysis::cogging(&m, sp, st.n_cog);
                let emf = analysis::emf(&m, st.n_emf.min(360));
                let f = analysis::forces(&m, 360.0, 180);
                let report = analysis::report_from(&s, &cog, &emf, &f);
                // en charge, sous les deux commandes (même crête de courant)
                let mut drive = VariantDrive::default();
                for trap in [false, true] {
                    let mut sc = s.clone();
                    sc.load.commutation = if trap { crate::geometry::Commutation::Trapeze } else { crate::geometry::Commutation::Sinus };
                    let Ok(mc) = Model::new(&sc, st.res) else { continue };
                    let (t, r) = analysis::ripple_quick(&mc, &emf, 72);
                    if trap {
                        (drive.torque_trap, drive.ripple_trap) = (t, r);
                    } else {
                        (drive.torque_sin, drive.ripple_sin) = (t, r);
                    }
                }
                Some(VariantResult { name: s.name.clone(), cog, report, drive })
            });
            send(Msg::Compare(out.into_iter().flatten().collect()));
        }
        Request::SurfaceAmplitude(base, st, max_amp, n_amp, n_pos) => {
            let amps: Vec<f64> = (0..n_amp).map(|k| max_amp * k as f64 / (n_amp - 1) as f64).collect();
            let sp = 360.0 / base.n_slots as f64;
            let xs: Vec<f64> = (0..n_pos).map(|i| sp * i as f64 / (n_pos - 1) as f64).collect();
            let mut rows: Vec<Vec<f64>> = Vec::with_capacity(n_amp);
            // répartition à décalages : celle du projet, sinon la première proposée
            let irr = if crate::repartition::is_regular_name(&base.variant) {
                crate::repartition::variants().into_iter().find(|v| !crate::repartition::is_regular_name(v)).map(String::from)
            } else {
                Some(base.variant.clone())
            };
            let Some(irr) = irr else { return };
            for (k, &a) in amps.iter().enumerate() {
                let mut s = base.clone();
                s.custom_angles_deg = None;
                s.variant = irr.clone();
                s.skew_max_deg = a.max(1e-9);
                let Ok(m) = Model::new(&s, st.res) else { return };
                rows.push(crate::par::map(xs.len(), |i| m.torque(&m.solve(xs[i].to_radians())) * 1e3));
                send(Msg::Progress(format!("{} {}/{}", crate::i18n::tr("Cartes 3D"), k + 1, n_amp)));
            }
            send(Msg::Surface(
                "cogging_amplitude".into(),
                Surface {
                    xs,
                    ys: amps,
                    z: rows.concat(),
                    x_label: "position rotor θr [°]".into(),
                    y_label: format!("amplitude des décalages max [°] ({irr})"),
                    z_label: "couple de détente [mN·m]".into(),
                },
            ));
        }
        Request::SurfaceBr(spec, st, n_th, n_pos, span_deg) => {
            let m = match Model::new(&spec, st.res) {
                Ok(m) => m,
                Err(e) => return send(Msg::Progress(e)),
            };
            let ys: Vec<f64> = (0..n_pos).map(|i| span_deg * i as f64 / (n_pos - 1).max(1) as f64).collect();
            let xs: Vec<f64> = (0..n_th).map(|i| 360.0 * i as f64 / n_th as f64).collect();
            let z: Vec<f64> = crate::par::map(ys.len(), |i| m.airgap_b(&m.solve(ys[i].to_radians()), m.r_mid_gap(), n_th).0).concat();
            send(Msg::Surface(
                "br".into(),
                Surface {
                    xs,
                    ys,
                    z,
                    x_label: "θ stator [°]".into(),
                    y_label: "position rotor θr [°]".into(),
                    z_label: if spec.topology == crate::geometry::Topology::Axial { "Bz entrefer, tranche de référence [T]".into() } else { "Br entrefer [T]".into() },
                },
            ));
        }
        Request::Validation(spec, st, nt, n) if spec.topology == crate::geometry::Topology::Axial => {
            // flux axial : tranche de référence en géométrie plane (circonférence
            // développée), résolue par les volumes finis en mode plan
            let m = match Model::new(&spec, st.res) {
                Ok(m) => m,
                Err(e) => return send(Msg::Progress(e)),
            };
            let k = m.reference_index();
            let mut sk = spec.clone();
            sk.magnet_arc_ratio = m.slice_arc_ratio(k);
            let pieces = crate::magnets::pieces(&sk);
            let zeros = vec![0.0; spec.n_slots];
            let (fd, ge) = crate::fdcheck::FdModel::general(&m.sections()[k].g, pieces.clone(), spec.br_magnet, nt, 1, &zeros);
            let r0 = m.slice_radius(k).0;
            let open = ge.beta_o * r0 * 1e3;
            let sec = match crate::section::Section::new(ge, m.sections()[k].h) {
                Ok(s) => s,
                Err(e) => return send(Msg::Progress(e)),
            };
            let mf = crate::magnets::fourier(&pieces, spec.br_magnet, m.sections()[k].h.nh);
            let per = spec.cogging_period_deg();
            send(Msg::Progress(format!("validation : tranche r = {:.1} mm, {n} positions", r0 * 1e3)));
            let rows: Vec<(f64, f64, f64)> = crate::par::map(n, |i| {
                let th = per * (i as f64 + 0.5) / n as f64;
                let ta = sec.rotor_torque(&sec.solve(&mf.rotated(th.to_radians()), &zeros)) * 1e3;
                let tf = fd.torque(&fd.solve(th.to_radians())) * 1e3;
                (th, ta, tf)
            });
            send(Msg::Validation(rows, open));
        }
        Request::Validation(spec, st, nt, n) => {
            let fd = crate::fdcheck::FdModel::new(&spec, nt, 1);
            let s2 = fd.spec.clone();
            let an = match Model::new(&s2, st.res) {
                Ok(m) => m,
                Err(e) => return send(Msg::Progress(e)),
            };
            let per = s2.cogging_period_deg();
            send(Msg::Progress(format!("validation : {n} positions ({} cœurs)", crate::par::threads())));
            let rows: Vec<(f64, f64, f64)> = crate::par::map(n, |k| {
                let th = per * (k as f64 + 0.5) / n as f64;
                let ta = an.torque(&an.solve(th.to_radians())) * 1e3;
                let tf = fd.torque(&fd.solve(th.to_radians())) * 1e3;
                (th, ta, tf)
            });
            send(Msg::Validation(rows, s2.slot_open));
        }
    }
}

/// Lance une requête en arrière-plan.
pub fn launch(req: Request) -> Job {
    let (tx, rx) = channel();
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        execute(req, &mut |m| {
            let _ = tx.send(m);
        })
    });
    #[cfg(target_arch = "wasm32")]
    {
        let w = crate::web::worker(crate::bg::Request::Compute(req), move |r| {
            if let crate::bg::Reply::Msg(m) = r {
                let _ = tx.send(m.into_msg());
            }
        });
        return Job { rx, _worker: Some(w) };
    }
    #[allow(unreachable_code)]
    Job { rx, _worker: None }
}

/// Lance le calcul principal (cogging, FEM, forces, rapport).
pub fn run_main(spec: MotorSpec, st: Settings) -> Job {
    launch(Request::Main(spec, st))
}

/// Étude paramétrique : un calcul complet par valeur du paramètre, chaque
/// point envoyé dès qu'il est prêt.
pub fn run_sweep(base: MotorSpec, st: Settings, var: crate::sweep::SweepVar, from: f64, to: f64, n: usize) -> Job {
    launch(Request::Sweep(base, st, var, from, to, n))
}

/// Toutes les répartitions proposées avec les mêmes paramètres que `base`.
pub fn run_compare(base: MotorSpec, st: Settings) -> Job {
    launch(Request::Compare(base, st))
}

/// Carte 3D du cogging : position rotor × amplitude des décalages.
pub fn run_surface_amplitude(base: MotorSpec, st: Settings, max_amp: f64, n_amp: usize, n_pos: usize) -> Job {
    launch(Request::SurfaceAmplitude(base, st, max_amp, n_amp, n_pos))
}

/// Carte 3D de l'induction radiale d'entrefer Br(θ, θr).
pub fn run_surface_br(spec: MotorSpec, st: Settings, n_th: usize, n_pos: usize, span_deg: f64) -> Job {
    launch(Request::SurfaceBr(spec, st, n_th, n_pos, span_deg))
}

/// Validation croisée : analytique vs volumes finis sur `n` positions.
pub fn run_validation(spec: MotorSpec, st: Settings, nt: usize, n: usize) -> Job {
    launch(Request::Validation(spec, st, nt, n))
}
