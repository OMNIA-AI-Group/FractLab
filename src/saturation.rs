//! Saturation magnétique par la **méthode du facteur de saturation**
//! (circuit magnétique équivalent, pratique courante de dimensionnement).
//!
//! Le modèle en sous-domaines suppose un fer de perméabilité infinie. Ici,
//! les inductions qu'il donne dans les dents, la culasse du stator et la
//! culasse du rotor servent à évaluer, avec une vraie courbe B(H), la force
//! magnétomotrice consommée par le fer sur le trajet d'une paire de pôles :
//!
//! `F_fer(s) = 2·h_dent·H(s·B_dent) + τ_culasse·⟨H(s·B_culasse·sin)⟩ + τ_rotor·⟨H(s·B_rotor·sin)⟩`
//!
//! où `s` est le facteur de réduction du flux. La source (aimants et
//! courants) fournit la même force magnétomotrice qu'en linéaire,
//! `F_entrefer = 2·B_entrefer·g'/μ0` avec `g' = g + h_aimant/μr`. Le point de
//! fonctionnement est la solution de `s·F_entrefer + F_fer(s) = F_entrefer`.
//! On en déduit la FEM corrigée (à vide), le couple corrigé (en charge) et la
//! courbe couple / courant. C'est une estimation d'ingénieur : elle ne
//! remplace pas un calcul par éléments finis non linéaires.

use crate::analysis::EmfCurve;
use crate::geometry::Topology;
use crate::subdomain::Model;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

const MU0: f64 = 4e-7 * PI;

/// Matériau magnétique (courbe B(H) typique).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IronMat {
    /// Tôles fer-silicium M270-35A.
    M270_35A,
    /// Tôles fer-silicium M400-50A.
    M400_50A,
    /// Composite magnétique doux (poudre de fer, type SMC).
    Smc,
    /// Acier massif doux (type 1010), culasses de rotor.
    Steel1010,
}

/// Tous les matériaux proposés.
pub const ALL_MATS: [IronMat; 4] = [IronMat::M270_35A, IronMat::M400_50A, IronMat::Smc, IronMat::Steel1010];

impl IronMat {
    /// Nom affiché.
    pub fn name(self) -> &'static str {
        match self {
            IronMat::M270_35A => "M270-35A",
            IronMat::M400_50A => "M400-50A",
            IronMat::Smc => "SMC",
            IronMat::Steel1010 => "Acier 1010",
        }
    }
    /// Courbe B(H) typique : points (B [T], H [A/m]).
    fn curve(self) -> &'static [(f64, f64)] {
        match self {
            IronMat::M270_35A => &[(0.0, 0.0), (0.5, 45.0), (1.0, 90.0), (1.2, 130.0), (1.4, 300.0), (1.5, 700.0), (1.6, 2000.0), (1.7, 5000.0), (1.8, 10000.0), (1.9, 18000.0), (2.0, 32000.0)],
            IronMat::M400_50A => &[(0.0, 0.0), (0.5, 70.0), (1.0, 120.0), (1.2, 170.0), (1.4, 400.0), (1.5, 1000.0), (1.6, 2600.0), (1.7, 6000.0), (1.8, 11000.0), (1.9, 20000.0), (2.0, 35000.0)],
            IronMat::Smc => &[(0.0, 0.0), (0.5, 600.0), (1.0, 1600.0), (1.2, 2500.0), (1.4, 5000.0), (1.5, 8000.0), (1.6, 13000.0), (1.7, 22000.0), (1.8, 40000.0)],
            IronMat::Steel1010 => &[(0.0, 0.0), (0.5, 150.0), (1.0, 300.0), (1.2, 500.0), (1.4, 1000.0), (1.5, 1600.0), (1.6, 3000.0), (1.7, 5800.0), (1.8, 10000.0), (1.9, 18000.0), (2.0, 32000.0)],
        }
    }
    /// Champ H [A/m] pour une induction B [T] (interpolation linéaire, puis
    /// pente μ0 au-delà du dernier point).
    pub fn h_of_b(self, b: f64) -> f64 {
        let b = b.abs();
        let c = self.curve();
        for w in c.windows(2) {
            if b <= w[1].0 {
                return w[0].1 + (b - w[0].0) / (w[1].0 - w[0].0) * (w[1].1 - w[0].1);
            }
        }
        let (bl, hl) = c[c.len() - 1];
        hl + (b - bl) / MU0
    }
    /// Induction du coude de saturation [T] (au-delà, μr chute fortement).
    pub fn knee(self) -> f64 {
        match self {
            IronMat::Smc => 1.4,
            _ => 1.6,
        }
    }
    /// H moyen le long d'un trajet où B varie en sinus de 0 à `b_max`.
    fn h_mean_sine(self, b_max: f64) -> f64 {
        let n = 16;
        (0..n).map(|k| self.h_of_b(b_max * ((k as f64 + 0.5) / n as f64 * PI / 2.0).sin())).sum::<f64>() / n as f64
    }
}

/// Réglages de la saturation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SatSpec {
    /// Matériau du stator.
    pub stator: IronMat,
    /// Matériau de la culasse du rotor.
    pub rotor: IronMat,
    /// Coefficient de foisonnement des tôles (1 pour le massif ou le SMC).
    pub stacking: f64,
}

impl Default for SatSpec {
    fn default() -> Self {
        SatSpec { stator: IronMat::M270_35A, rotor: IronMat::Steel1010, stacking: 0.95 }
    }
}

/// Point de fonctionnement saturé.
#[derive(Clone, Debug, Default, Serialize, serde::Deserialize)]
pub struct SatPoint {
    /// Courant de phase crête [A].
    pub current: f64,
    /// Facteur de réduction du flux s (1 = pas de saturation).
    pub s: f64,
    /// Facteur de saturation k = 1 + F_fer / F_entrefer.
    pub k_sat: f64,
    /// Induction crête dans les dents, corrigée [T].
    pub b_tooth: f64,
    /// Induction crête dans la culasse du stator, corrigée [T].
    pub b_yoke: f64,
    /// Induction crête dans la culasse du rotor, corrigée [T].
    pub b_rotor: f64,
}

/// Résultat de l'analyse de saturation.
#[derive(Clone, Debug, Default, Serialize, serde::Deserialize)]
pub struct SatCurve {
    /// À vide.
    pub no_load: SatPoint,
    /// Au courant de la spécification.
    pub at_load: SatPoint,
    /// Courbe couple / courant : (courant [A], couple linéaire [N·m], couple saturé [N·m]).
    pub torque_current: Vec<[f64; 3]>,
    /// Coude de saturation du stator [T].
    pub knee: f64,
}

/// Inductions linéaires crête (dent, culasse stator, culasse rotor, entrefer)
/// pour des courants donnés, maximum sur quelques positions du rotor.
fn linear_b(model: &Model, currents_at: &(dyn Fn(f64) -> [f64; 3] + Sync), st: &SatSpec) -> (f64, f64, f64, f64) {
    let s = &model.spec;
    let q = model.q;
    let p = (s.n_poles / 2).max(1) as f64;
    let len = s.length();
    let kst = st.stacking.clamp(0.5, 1.0);
    let (w_t, w_y, w_r) = match s.topology {
        Topology::Axial => {
            let pitch = 2.0 * PI * s.axial_r_mean() / q as f64;
            ((pitch - s.slot_w.max(s.slot_open) * 1e-3).max(1e-4), s.yoke_th * 1e-3, s.rotor_yoke_th * 1e-3)
        }
        _ => (s.tooth_w * 1e-3, s.yoke_th * 1e-3, s.rotor_yoke_th * 1e-3),
    };
    let npos = 6;
    let vals: Vec<(f64, f64, f64, f64)> = crate::par::map(npos, |k| {
        let th = 2.0 * PI * k as f64 / npos as f64 / p / 2.0 + 0.011;
        let f = model.solve_i(th, &currents_at(th));
        // flux d'un seul côté du stator (le modèle additionne les entrefers identiques)
        let phi: Vec<f64> = (0..q).map(|j| model.tooth_flux(&f, j) / model.sides()).collect();
        let bt = phi.iter().fold(0.0_f64, |a, v| a.max(v.abs())) / (w_t * len * kst);
        let mut cum = Vec::with_capacity(q);
        let mut acc = 0.0;
        for v in &phi {
            acc += v;
            cum.push(acc);
        }
        let mean = cum.iter().sum::<f64>() / q as f64;
        let fy = cum.iter().fold(0.0_f64, |a, v| a.max((v - mean).abs()));
        let (br, _) = model.airgap_b(&f, model.r_mid_gap(), 720);
        let bg = br.iter().fold(0.0_f64, |a, v| a.max(v.abs()));
        (bt, fy / (w_y * len * kst), fy / (w_r * len), bg)
    });
    vals.iter().fold((0.0, 0.0, 0.0, 0.0), |a, v| (a.0.max(v.0), a.1.max(v.1), a.2.max(v.2), a.3.max(v.3)))
}

/// Résout le point saturé à partir des inductions linéaires.
fn solve_point(model: &Model, st: &SatSpec, current: f64, b: (f64, f64, f64, f64)) -> SatPoint {
    let s = &model.spec;
    let p = (s.n_poles / 2).max(1) as f64;
    let (bt, by, br, bg) = b;
    let g_eff = (s.airgap + s.magnet_th / s.mur_magnet.max(1.0)) * 1e-3;
    let f_gap = 2.0 * bg * g_eff / MU0;
    let h_t = s.slot_depth_m() + s.tip_h * 1e-3;
    let r_y = match s.topology {
        Topology::Axial => s.axial_r_mean(),
        _ => s.r_bore(),
    };
    let tau = PI * r_y / p;
    let f_iron = |x: f64| 2.0 * h_t * st.stator.h_of_b(x * bt) + tau * st.stator.h_mean_sine(x * by) + tau * st.rotor.h_mean_sine(x * br);
    let x = if f_gap <= 0.0 {
        1.0
    } else {
        // g(x) = x·F_entrefer + F_fer(x) − F_entrefer est croissante : bissection
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..60 {
            let m = 0.5 * (lo + hi);
            if m * f_gap + f_iron(m) > f_gap {
                hi = m;
            } else {
                lo = m;
            }
        }
        0.5 * (lo + hi)
    };
    SatPoint {
        current,
        s: x,
        k_sat: if f_gap > 0.0 { 1.0 + f_iron(x) / (x * f_gap).max(1e-12) } else { 1.0 },
        b_tooth: x * bt,
        b_yoke: x * by,
        b_rotor: x * br,
    }
}

/// Analyse de saturation : à vide, au courant nominal, et courbe couple /
/// courant jusqu'à trois fois le courant nominal.
pub fn analyse(model: &Model, em: &EmfCurve, avg_torque: f64) -> SatCurve {
    let s = &model.spec;
    let st = &s.saturation;
    let p = (s.n_poles / 2) as f64;
    let i0 = s.load.i_peak;
    let phase_of = |e: &Vec<f64>| {
        let (mut re, mut im) = (0.0, 0.0);
        for (k, v) in e.iter().enumerate() {
            let a = p * 2.0 * PI * k as f64 / e.len() as f64;
            re += -v * a.cos();
            im += -v * a.sin();
        }
        im.atan2(re)
    };
    let phi = [phase_of(&em.emf[0]), phase_of(&em.emf[1]), phase_of(&em.emf[2])];
    let gamma = s.load.angle_deg.to_radians();
    let at = |i: f64| {
        let f = move |th: f64| -> [f64; 3] { core::array::from_fn(|k| i * (p * th - phi[k] + gamma).cos()) };
        solve_point(model, st, i, linear_b(model, &f, st))
    };
    let no_load = at(0.0);
    let at_load = at(i0);
    let kt = if i0 > 0.0 { avg_torque / i0 } else { 0.0 };
    let torque_current = (0..=12)
        .map(|k| {
            let i = i0 * 3.0 * k as f64 / 12.0;
            let pt = if k == 0 { no_load.clone() } else { at(i) };
            [i, kt * i, kt * i * pt.s]
        })
        .collect();
    SatCurve { no_load, at_load, torque_current, knee: st.stator.knee() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::MotorSpec;
    use crate::subdomain::Resolution;

    #[test]
    fn courbes_monotones() {
        for m in ALL_MATS {
            let mut last = -1.0;
            for k in 0..60 {
                let h = m.h_of_b(k as f64 * 0.04);
                assert!(h >= last, "{m:?}");
                last = h;
            }
        }
    }

    /// Faible induction : s ≈ 1 ; plus de courant ou d'aimant : plus de saturation.
    #[test]
    fn saturation_croissante() {
        let mut s = MotorSpec::default();
        s.br_magnet = 0.3;
        let m = Model::new(&s, Resolution { nh: 120, nk: 10 }).unwrap();
        let em = crate::analysis::emf(&m, 90);
        let c = analyse(&m, &em, 0.05);
        assert!(c.no_load.s > 0.97, "{}", c.no_load.s);
        // à 3 fois le courant nominal, le flux est au plus celui de vide
        let last = c.torque_current.last().unwrap();
        assert!(last[2] <= last[1] + 1e-12 && last[2] / last[1] <= c.no_load.s + 1e-6);
        let mut s2 = MotorSpec::default();
        s2.br_magnet = 1.45;
        s2.tooth_w *= 0.4;
        let m2 = Model::new(&s2, Resolution { nh: 120, nk: 10 }).unwrap();
        let c2 = analyse(&m2, &crate::analysis::emf(&m2, 90), 0.05);
        assert!(c2.no_load.s < c.no_load.s && c2.no_load.k_sat > 1.0);
    }
}

/// Inductions crête à vide, en linéaire (dent, culasse stator, culasse
/// rotor, entrefer) [T] : estimation rapide utilisée comme contrainte par
/// l'Ajustement I.A.
pub fn no_load_inductions(model: &Model) -> (f64, f64, f64, f64) {
    linear_b(model, &|_| [0.0; 3], &model.spec.saturation)
}
