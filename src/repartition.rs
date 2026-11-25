//! Répartitions des aimants, projet de départ et seuils des critères.
//!
//! La version publique connaît la répartition régulière et les angles
//! personnalisés (saisis ou importés). Un module privé facultatif
//! (`../prive/phi.rs`, compilé avec l'option `phi`, voir `build.rs`) peut
//! ajouter ses propres répartitions, son projet de référence, ses seuils de
//! critères, ses traductions et son chapitre d'aide : tout le reste de
//! l'application passe par ce module et fonctionne de la même façon avec ou
//! sans lui.

use crate::geometry::MotorSpec;

#[cfg(phi)]
#[path = "../../prive/phi.rs"]
pub mod prive;

/// Nom de la répartition régulière (aimants au pas polaire).
pub const REGULIERE: &str = "Régulière";

/// Seuils des critères de la synthèse (`None` = pas de seuil, valeur affichée seule).
#[derive(Clone, Copy, Debug)]
pub struct Criteria {
    /// Force radiale résultante maximale [N].
    pub force_max_n: Option<f64>,
    /// Balourd maximal [mg·cm].
    pub balourd_max_mg_cm: Option<f64>,
    /// Autocorrélation maximale des écarts angulaires (et plancher atteignable).
    pub autocorr_max: Option<(f64, f64)>,
    /// Taux de distorsion maximal de la FEM [%].
    pub thd_max_pct: Option<f64>,
    /// Gain de cogging visé par rapport à une répartition de référence : (nom, %).
    pub cogging_target: Option<(&'static str, f64)>,
    /// Origine des seuils (clé de traduction).
    pub source: &'static str,
}

/// Seuils publics : seule la distorsion de la FEM a un seuil d'usage courant.
pub const CRITERES_PUBLICS: Criteria = Criteria {
    force_max_n: None,
    balourd_max_mg_cm: None,
    autocorr_max: None,
    thd_max_pct: Some(5.0),
    cogging_target: None,
    source: "critères généraux",
};

/// Répartitions proposées dans le panneau de gauche.
pub fn variants() -> Vec<&'static str> {
    #[cfg(phi)]
    let v = {
        // garde la marque du module privé dans le binaire (contrôle des paquets publics)
        std::hint::black_box(prive::MARQUE);
        prive::VARIANTS.to_vec()
    };
    #[cfg(not(phi))]
    let v = vec![REGULIERE];
    v
}

/// Description d'une répartition (clé de traduction).
pub fn describe(v: &str) -> &'static str {
    #[cfg(phi)]
    if let Some(d) = prive::describe(v) {
        return d;
    }
    if v == REGULIERE { "aimants au pas polaire" } else { "" }
}

/// La répartition `v` est-elle régulière (aimants au pas polaire) ? Une
/// répartition inconnue de cette version est traitée comme régulière.
pub fn is_regular_name(v: &str) -> bool {
    #[cfg(phi)]
    if let Some(r) = prive::is_regular(v) {
        return r;
    }
    let _ = v;
    true
}

/// Angles des aimants [°] d'une répartition non régulière, `None` si la
/// répartition est régulière ou inconnue de cette version.
pub fn angles(spec: &MotorSpec) -> Option<Vec<f64>> {
    #[cfg(phi)]
    let a = prive::angles(spec);
    #[cfg(not(phi))]
    let a = {
        let _ = spec;
        None
    };
    a
}

/// Réglages associés au choix d'une répartition (cotes imposées, etc.).
pub fn on_select(spec: &mut MotorSpec) {
    #[cfg(phi)]
    prive::on_select(spec);
    let _ = spec;
}

/// Les répartitions utilisent-elles l'amplitude des décalages (`skew_max_deg`) ?
pub fn uses_offsets() -> bool {
    cfg!(phi)
}

/// Projet de départ de l'application.
pub fn default_project() -> MotorSpec {
    #[cfg(phi)]
    let s = prive::default_project();
    #[cfg(not(phi))]
    let s = crate::demo::spec("drone");
    s
}

/// Projet de référence pour la répartition `v` (sans module privé : le
/// moteur de démonstration par défaut, répartition régulière).
pub fn reference(v: &str) -> MotorSpec {
    #[cfg(phi)]
    let s = prive::reference(v);
    #[cfg(not(phi))]
    let s = {
        let _ = v;
        crate::demo::spec("drone")
    };
    s
}

/// Seuils des critères de la synthèse.
pub fn criteria() -> Criteria {
    #[cfg(phi)]
    let c = prive::CRITERIA;
    #[cfg(not(phi))]
    let c = CRITERES_PUBLICS;
    c
}

/// Traductions supplémentaires (clé française, anglais, espagnol, allemand).
pub fn extra_translations() -> &'static [(&'static str, &'static str, &'static str, &'static str)] {
    #[cfg(phi)]
    let t = prive::TABLE;
    #[cfg(not(phi))]
    let t = &[];
    t
}

/// Chapitres d'aide supplémentaires.
pub fn extra_help() -> &'static [crate::help::Section] {
    #[cfg(phi)]
    let h = prive::HELP;
    #[cfg(not(phi))]
    let h = &[];
    h
}

/// Répartition servant de référence aux comparaisons (écart de cogging).
pub fn reference_name() -> &'static str {
    criteria().cogging_target.map(|c| c.0).unwrap_or_else(|| variants()[0])
}
