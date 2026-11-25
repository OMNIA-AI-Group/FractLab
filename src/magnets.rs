//! Couronne aimantée : découpage en morceaux d'aimant et séries de Fourier
//! de l'aimantation (Mr, Mθ) sur 360° — sans hypothèse de périodicité,
//! ce qui rend visibles les répartitions apériodiques (décalées, aléatoires…).

use crate::geometry::{Magnetisation, MotorSpec};
use std::f64::consts::PI;

/// Perméabilité du vide μ0 [H/m].
pub const MU0: f64 = 4e-7 * PI;

/// Morceau d'aimant d'aimantation uniforme en norme, sur un arc [t1, t2]
/// (repère rotor, radians).
#[derive(Clone, Debug)]
pub struct Piece {
    /// Angle de début de l'arc [rad], repère rotor.
    pub t1: f64,
    /// Angle de fin de l'arc [rad], repère rotor.
    pub t2: f64,
    /// Loi d'aimantation du morceau.
    pub kind: PieceKind,
}

#[derive(Clone, Debug)]
/// Loi d'aimantation d'un morceau d'aimant.
pub enum PieceKind {
    /// Aimantation radiale : Mr = s·M, Mθ = 0.
    /// Aimantation radiale de sens `s` (+1 : vers l'extérieur, −1 : vers l'axe).
    Radial {
        /// Sens : +1 vers l'extérieur, −1 vers l'axe.
        s: f64,
    },
    /// Direction fixe ψ : Mr = M cos(θ−ψ), Mθ = M sin(ψ−θ).
    /// Aimantation de direction fixe `psi` [rad] dans le plan (parallèle, Halbach).
    Fixed {
        /// Direction d'aimantation dans le plan [rad].
        psi: f64,
    },
}

/// Découpe la couronne selon la spec (centres des aimants donnés par la variante).
pub fn pieces(spec: &MotorSpec) -> Vec<Piece> {
    let centers: Vec<f64> = spec.magnet_angles_deg().iter().map(|a| a.to_radians()).collect();
    let n = centers.len();
    let pitch = 2.0 * PI / n as f64;
    let half_arc = spec.magnet_arc_ratio * pitch / 2.0;
    let sign = |k: usize| if k.is_multiple_of(2) { 1.0 } else { -1.0 };
    let mut out = Vec::new();
    match spec.magnetisation {
        Magnetisation::Radiale => {
            for (k, &c) in centers.iter().enumerate() {
                out.push(Piece { t1: c - half_arc, t2: c + half_arc, kind: PieceKind::Radial { s: sign(k) } });
            }
        }
        Magnetisation::Parallele => {
            for (k, &c) in centers.iter().enumerate() {
                let psi = if sign(k) > 0.0 { c } else { c + PI };
                out.push(Piece { t1: c - half_arc, t2: c + half_arc, kind: PieceKind::Fixed { psi } });
            }
        }
        Magnetisation::Halbach { segments } => {
            // Pôles bornés aux milieux entre centres voisins (gère l'apériodique).
            let segs = segments.max(2) as usize;
            for k in 0..n {
                let prev = centers[(k + n - 1) % n] - if k == 0 { 2.0 * PI } else { 0.0 };
                let next = centers[(k + 1) % n] + if k == n - 1 { 2.0 * PI } else { 0.0 };
                let a = 0.5 * (prev + centers[k]);
                let b = 0.5 * (centers[k] + next);
                let w = (b - a) / segs as f64;
                for j in 0..segs {
                    let t1 = a + j as f64 * w;
                    let tc = t1 + w / 2.0;
                    // phase locale : ±π/2 aux bords du pôle
                    let u = PI * (tc - centers[k]) / (b - a);
                    // champ concentré côté intérieur (entrefer d'un rotor externe)
                    let (mr, mt) = (sign(k) * u.cos(), sign(k) * u.sin());
                    let psi = tc + mt.atan2(mr);
                    out.push(Piece { t1, t2: t1 + w, kind: PieceKind::Fixed { psi } });
                }
            }
        }
    }
    out
}

/// Coefficients de Fourier (n = 0..=nh) de Mr et Mθ [A/m], repère rotor.
/// Retourne (mr_c, mr_s, mt_c, mt_s), indice n.
pub struct MagFourier {
    /// Coefficients en cosinus de Mr [A/m], indice = rang n.
    pub mr_c: Vec<f64>,
    /// Coefficients en sinus de Mr [A/m].
    pub mr_s: Vec<f64>,
    /// Coefficients en cosinus de Mθ [A/m].
    pub mt_c: Vec<f64>,
    /// Coefficients en sinus de Mθ [A/m].
    pub mt_s: Vec<f64>,
}

const GL8: [(f64, f64); 8] = [
    (-0.9602898564975363, 0.1012285362903763),
    (-0.7966664774136267, 0.2223810344533745),
    (-0.525_532_409_916_329, 0.3137066458778873),
    (-0.1834346424956498, 0.362_683_783_378_362),
    (0.1834346424956498, 0.362_683_783_378_362),
    (0.525_532_409_916_329, 0.3137066458778873),
    (0.7966664774136267, 0.2223810344533745),
    (0.9602898564975363, 0.1012285362903763),
];

/// Coefficients de Fourier (rangs 0..=nh) de l'aimantation de la couronne,
/// pour une induction rémanente `br` [T] ; exacts pour les morceaux radiaux,
/// par quadrature de Gauss composite sinon.
pub fn fourier(pieces: &[Piece], br: f64, nh: usize) -> MagFourier {
    let m = br / MU0;
    let mut f = MagFourier {
        mr_c: vec![0.0; nh + 1],
        mr_s: vec![0.0; nh + 1],
        mt_c: vec![0.0; nh + 1],
        mt_s: vec![0.0; nh + 1],
    };
    for p in pieces {
        match p.kind {
            PieceKind::Radial { s } => {
                // exact : ∫cos nθ = (sin n t2 − sin n t1)/n
                for n in 1..=nh {
                    let nf = n as f64;
                    f.mr_c[n] += s * m / PI * ((nf * p.t2).sin() - (nf * p.t1).sin()) / nf;
                    f.mr_s[n] += s * m / PI * ((nf * p.t1).cos() - (nf * p.t2).cos()) / nf;
                }
                f.mr_c[0] += s * m * (p.t2 - p.t1) / (2.0 * PI);
            }
            PieceKind::Fixed { psi } => {
                // quadrature de Gauss composite, ≥ 2 panneaux par demi-période
                let w = p.t2 - p.t1;
                let panels = ((nh as f64 * w / PI).ceil() as usize + 2).max(4);
                let h = w / panels as f64;
                for q in 0..panels {
                    let a = p.t1 + q as f64 * h;
                    for &(x, wt) in &GL8 {
                        let th = a + (x + 1.0) * h / 2.0;
                        let ww = wt * h / 2.0;
                        let mr = m * (th - psi).cos();
                        let mt = m * (psi - th).sin();
                        f.mr_c[0] += mr * ww / (2.0 * PI);
                        f.mt_c[0] += mt * ww / (2.0 * PI);
                        // récurrence trigonométrique pour cos nθ, sin nθ
                        let (s1, c1) = th.sin_cos();
                        let (mut sn, mut cn) = (s1, c1);
                        for n in 1..=nh {
                            f.mr_c[n] += mr * cn * ww / PI;
                            f.mr_s[n] += mr * sn * ww / PI;
                            f.mt_c[n] += mt * cn * ww / PI;
                            f.mt_s[n] += mt * sn * ww / PI;
                            let c2 = cn * c1 - sn * s1;
                            sn = sn * c1 + cn * s1;
                            cn = c2;
                        }
                    }
                }
            }
        }
    }
    f
}

impl MagFourier {
    /// Coefficients dans le repère stator pour un rotor tourné de `theta_r`.
    pub fn rotated(&self, theta_r: f64) -> MagFourier {
        let rot = |c: &Vec<f64>, s: &Vec<f64>| {
            let mut oc = c.clone();
            let mut os = s.clone();
            for n in 1..c.len() {
                let (sn, cn) = (n as f64 * theta_r).sin_cos();
                oc[n] = c[n] * cn - s[n] * sn;
                os[n] = c[n] * sn + s[n] * cn;
            }
            (oc, os)
        };
        let (mr_c, mr_s) = rot(&self.mr_c, &self.mr_s);
        let (mt_c, mt_s) = rot(&self.mt_c, &self.mt_s);
        MagFourier { mr_c, mr_s, mt_c, mt_s }
    }
}
