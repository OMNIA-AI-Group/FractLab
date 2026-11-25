//! Étude paramétrique (calcul en lot) : un paramètre varie entre deux bornes,
//! le moteur complet est recalculé à chaque valeur et les grandeurs clés sont
//! tracées, tabulées et exportables en CSV.

use crate::compute::Results;
use crate::geometry::MotorSpec;
use crate::optimize::Var;

/// Paramètre que l'on fait varier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SweepVar {
    /// Un des réglages géométriques de l'Ajustement I.A.
    Geo(Var),
    /// Induction rémanente des aimants [T].
    Br,
    /// Longueur active [mm].
    StackLen,
    /// Courant de phase crête [A].
    Current,
    /// Angle du courant / FEM [°él].
    CurrentAngle,
    /// Spires par bobine.
    Turns,
    /// Vitesse [tr/min].
    Rpm,
}

/// Tous les paramètres proposés.
pub const ALL: [SweepVar; 12] = [
    SweepVar::Geo(Var::ArcRatio),
    SweepVar::Geo(Var::SlotOpen),
    SweepVar::Geo(Var::MagnetTh),
    SweepVar::Geo(Var::SkewAmp),
    SweepVar::Geo(Var::TipH),
    SweepVar::Geo(Var::Airgap),
    SweepVar::Br,
    SweepVar::StackLen,
    SweepVar::Current,
    SweepVar::CurrentAngle,
    SweepVar::Turns,
    SweepVar::Rpm,
];

impl SweepVar {
    /// Nom français (clé de traduction).
    pub fn name(self) -> &'static str {
        match self {
            SweepVar::Geo(v) => v.name(),
            SweepVar::Br => "Induction rémanente Br",
            SweepVar::StackLen => "Longueur active",
            SweepVar::Current => "Courant de phase crête",
            SweepVar::CurrentAngle => "Angle du courant / FEM",
            SweepVar::Turns => "Spires par bobine",
            SweepVar::Rpm => "Vitesse",
        }
    }
    /// Unité affichée.
    pub fn unit(self) -> &'static str {
        match self {
            SweepVar::Geo(Var::ArcRatio) => "",
            SweepVar::Geo(Var::SkewAmp) => "°",
            SweepVar::Geo(_) | SweepVar::StackLen => "mm",
            SweepVar::Br => "T",
            SweepVar::Current => "A",
            SweepVar::CurrentAngle => "°él",
            SweepVar::Turns => "",
            SweepVar::Rpm => "tr/min",
        }
    }
    /// Valeur actuelle.
    pub fn get(self, s: &MotorSpec) -> f64 {
        match self {
            SweepVar::Geo(v) => v.get(s),
            SweepVar::Br => s.br_magnet,
            SweepVar::StackLen => s.stack_len,
            SweepVar::Current => s.load.i_peak,
            SweepVar::CurrentAngle => s.load.angle_deg,
            SweepVar::Turns => s.turns_per_coil,
            SweepVar::Rpm => s.rpm,
        }
    }
    /// Applique une valeur.
    pub fn set(self, s: &mut MotorSpec, x: f64) {
        match self {
            SweepVar::Geo(v) => v.set(s, x),
            SweepVar::Br => s.br_magnet = x,
            SweepVar::StackLen => s.stack_len = x,
            SweepVar::Current => s.load.i_peak = x,
            SweepVar::CurrentAngle => s.load.angle_deg = x,
            SweepVar::Turns => {
                s.turns_per_coil = x.round().max(1.0);
                for c in &mut s.coils {
                    c.turns = s.turns_per_coil;
                }
            }
            SweepVar::Rpm => s.rpm = x,
        }
    }
    /// Plage proposée par défaut autour de la valeur actuelle.
    pub fn default_range(self, s: &MotorSpec) -> (f64, f64) {
        let v = self.get(s);
        match self {
            SweepVar::Geo(g) => g.bounds(s),
            SweepVar::CurrentAngle => (-90.0, 90.0),
            SweepVar::Br => (0.9, 1.5),
            SweepVar::Turns => ((v * 0.5).round().max(1.0), (v * 1.5).round().max(2.0)),
            _ => (0.5 * v, 1.5 * v.max(1e-6)),
        }
    }
}

/// Grandeurs calculées à chaque point : (nom français, unité).
pub const OUTPUTS: [(&str, &str); 11] = [
    ("Cogging crête-à-crête", "mN·m"),
    ("Cogging efficace", "mN·m"),
    ("Force radiale max", "N"),
    ("FEM crête", "V"),
    ("THD de la FEM", "%"),
    ("Couple moyen", "N·m"),
    ("Ondulation", "%"),
    ("Inductance synchrone", "mH"),
    ("Pertes Joule", "W"),
    ("Écart minimal entre aimants", "°"),
    ("KV estimé", "tr/min/V"),
];

/// Calcule les grandeurs de `OUTPUTS` pour un résultat.
pub fn outputs(r: &Results) -> Vec<f64> {
    let rep = &r.report;
    let emf = r.emf.emf[0].iter().fold(0.0_f64, |a, v| a.max(v.abs()));
    vec![
        rep.cogging.cogging_pp_mnm,
        rep.cogging.cogging_rms_mnm,
        rep.force_max_n,
        emf,
        rep.emf_thd_pct,
        r.load.avg_torque,
        r.load.ripple_pct,
        r.load.ls * 1e3,
        r.load.p_joule,
        rep.min_magnet_gap_deg,
        rep.kv_estime,
    ]
}

/// Tableau CSV (séparateur « ; », point décimal).
pub fn csv(var: SweepVar, rows: &[(f64, Vec<f64>)]) -> String {
    let mut s = format!("{} [{}]", var.name(), var.unit());
    for (n, u) in OUTPUTS {
        s += &format!(";{n} [{u}]");
    }
    s.push('\n');
    for (v, o) in rows {
        s += &format!("{v}");
        for x in o {
            s += &format!(";{x}");
        }
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::{compute_results, Settings};

    /// Le couple moyen croît avec le courant, la FEM avec Br.
    #[test]
    fn tendances_physiques() {
        let st = Settings { n_cog: 90, n_emf: 90, ..Default::default() };
        let base = MotorSpec::irregular_example();
        let at = |var: SweepVar, x: f64| {
            let mut s = base.clone();
            var.set(&mut s, x);
            outputs(&compute_results(s, &st).unwrap())
        };
        assert!(at(SweepVar::Current, 10.0)[5] > at(SweepVar::Current, 5.0)[5] * 1.5);
        assert!(at(SweepVar::Br, 1.4)[3] > at(SweepVar::Br, 1.0)[3] * 1.3);
        let c = csv(SweepVar::Br, &[(1.0, vec![0.0; OUTPUTS.len()])]);
        assert_eq!(c.lines().count(), 2);
    }
}

#[cfg(test)]
mod tests_noms {
    use super::*;
    /// Noms des paramètres et des grandeurs traduits dans les 4 langues.
    #[test]
    fn noms_traduits() {
        let have = |k: &str| crate::i18n::TABLE.iter().any(|(f, _, _, _)| *f == k);
        let mut manq: Vec<&str> = ALL.iter().map(|v| v.name()).chain(OUTPUTS.iter().map(|o| o.0)).filter(|k| !have(k)).collect();
        for k in ["Paramétrique", "étude paramétrique", "Exporté :", "Appliquer"] {
            if !have(k) {
                manq.push(k);
            }
        }
        assert!(manq.is_empty(), "{manq:?}");
    }
}
