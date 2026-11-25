//! Modèle analytique GÉNÉRAL en sous-domaines pour un tronçon 2D de machine
//! à aimants permanents (méthode de séparation des variables, formulation de
//! Lubin / Zhu / Tiegna).
//!
//! Un même code traite :
//! - la géométrie **polaire** (machine radiale, rotor extérieur ou intérieur) et
//!   la géométrie **plane** (tranche d'une machine à flux axial, déroulée au
//!   rayon moyen r0 de la tranche : c'est la technique multi-tranches) ;
//! - les encoches **semi-fermées** : régions I (encoche, densité de courant J
//!   uniforme), II (isthme / ouverture), III (entrefer), IV (aimants) ;
//! - les **courants** d'encoche (fonctionnement en charge, inductances).
//!
//! Notations : coordonnée normale `y` (rayon r en polaire, cote z en plan),
//! coordonnée périodique θ ∈ [0, 2π) (en plan, x = r0·θ). Potentiel vecteur
//! A_z ; Br = (1/ρ)·∂A/∂θ et Bθ = −∂A/∂y avec ρ = y (polaire) ou r0 (plan).
//! Fer de perméabilité infinie : Hθ = 0 aux frontières fer.
//!
//! Résolution : l'entrefer et les aimants sont éliminés analytiquement
//! harmonique par harmonique (système 4×4) ; les inconnues restantes sont les
//! coefficients des encoches et des isthmes, couplés par l'entrefer. La
//! matrice ne dépend pas de la position du rotor ni des courants : elle est
//! factorisée une fois.

use crate::linalg::Lu;
use crate::magnets::{MagFourier, MU0};
use std::f64::consts::PI;

/// Type de coordonnées du tronçon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coord {
    /// Machine radiale (coordonnées polaires).
    Polar,
    /// Tranche déroulée d'une machine axiale, au rayon moyen `r0` [m].
    Planar {
        /// Rayon moyen de la tranche [m].
        r0: f64,
    },
}

/// Géométrie d'un tronçon (rayons/cotes en mètres, angles en radians).
#[derive(Clone, Debug)]
pub struct SectionGeom {
    /// Polaire ou plan.
    pub coord: Coord,
    /// Fond d'encoche (côté culasse statorique).
    pub y_sb: f64,
    /// Haut de l'encoche = bas de l'isthme.
    pub y_ib: f64,
    /// Surface du stator côté entrefer (haut de l'isthme).
    pub y_s: f64,
    /// Face des aimants côté entrefer.
    pub y_m: f64,
    /// Face des aimants côté fer rotorique.
    pub y_r: f64,
    /// Centres angulaires des encoches.
    pub slot_center: Vec<f64>,
    /// Largeur angulaire du corps d'encoche.
    pub beta_s: f64,
    /// Largeur angulaire de l'ouverture (isthme), ≤ beta_s.
    pub beta_o: f64,
    /// Perméabilité relative de recul des aimants.
    pub mur: f64,
    /// Profondeur du modèle 2D : longueur active (polaire) ou largeur de tranche (plan) [m].
    pub depth: f64,
}

/// Résolution harmonique d'un tronçon.
#[derive(Clone, Copy, Debug)]
pub struct Harmonics {
    /// Harmoniques d'entrefer.
    pub nh: usize,
    /// Harmoniques par encoche.
    pub ks: usize,
    /// Harmoniques par isthme.
    pub ko: usize,
}

/// Fonctions de base d'une région [lo, hi] pour un exposant e :
/// u1 croît vers `hi` (u1(hi) = 1), u2 croît vers `lo` (u2(lo) = 1) ;
/// pour e = 0 : u1 = 1 et u2 = ln y (polaire) ou y/r0 (plan).
#[derive(Clone, Copy)]
struct Basis {
    coord: Coord,
    lo: f64,
    hi: f64,
}

impl Basis {
    /// (u1, u1', u2, u2') en y.
    fn eval(&self, e: f64, y: f64) -> (f64, f64, f64, f64) {
        match self.coord {
            Coord::Polar => {
                if e == 0.0 {
                    (1.0, 0.0, y.ln(), 1.0 / y)
                } else {
                    let u1 = (y / self.hi).powf(e);
                    let u2 = (y / self.lo).powf(-e);
                    (u1, e / y * u1, u2, -e / y * u2)
                }
            }
            Coord::Planar { r0 } => {
                if e == 0.0 {
                    (1.0, 0.0, y / r0, 1.0 / r0)
                } else {
                    let u1 = (e * (y - self.hi) / r0).exp();
                    let u2 = (-e * (y - self.lo) / r0).exp();
                    (u1, e / r0 * u1, u2, -e / r0 * u2)
                }
            }
        }
    }
}

fn region(coord: Coord, a: f64, b: f64) -> Basis {
    Basis { coord, lo: a.min(b), hi: a.max(b) }
}

fn sinc_int(x: f64, b: f64) -> f64 {
    if (x * b).abs() < 1e-9 { b } else { (x * b).sin() / x }
}
fn cosc_int(x: f64, b: f64) -> f64 {
    if (x * b).abs() < 1e-9 { x * b * b / 2.0 } else { (1.0 - (x * b).cos()) / x }
}
/// ∫0^b cos(a u) cos(c u) du
fn icc(a: f64, c: f64, b: f64) -> f64 {
    0.5 * (sinc_int(a - c, b) + sinc_int(a + c, b))
}
/// ∫0^b cos(a u) sin(c u) du
fn ics(a: f64, c: f64, b: f64) -> f64 {
    0.5 * (cosc_int(c - a, b) + cosc_int(c + a, b))
}

#[derive(Clone, Copy)]
struct Harm {
    inv: [[f64; 4]; 4],
    alpha: f64,
}

/// Tronçon prêt à résoudre.
pub struct Section {
    /// Géométrie.
    pub g: SectionGeom,
    /// Résolution.
    pub h: Harmonics,
    gap: Basis,
    mag: Basis,
    isth: Basis,
    slot: Basis,
    harm: Vec<Harm>,
    /// ouverture : Ic/Is[(i, k, n)]
    ic: Vec<f64>,
    is_: Vec<f64>,
    /// moyenne de Gk sur la surface de l'encoche (Gk(y_ib) = 1)
    gavg: Vec<f64>,
    lu: Lu,
    nb: usize,
}

/// Solution d'un tronçon pour une position du rotor et des courants donnés.
#[derive(Clone, Debug)]
pub struct SectionField {
    /// Entrefer cos : (c, d) ; sin : (e, f) pour n = 1..=nh.
    pub c: Vec<f64>,
    /// Voir `c`.
    pub d: Vec<f64>,
    /// Voir `c`.
    pub e: Vec<f64>,
    /// Voir `c`.
    pub f: Vec<f64>,
    /// Aimants cos : (g, h) + amplitude particulière pc ; sin : (gs, hs, ps).
    pub g: Vec<f64>,
    /// Voir `g`.
    pub hh: Vec<f64>,
    /// Voir `g`.
    pub pc: Vec<f64>,
    /// Voir `g`.
    pub gs: Vec<f64>,
    /// Voir `g`.
    pub hs: Vec<f64>,
    /// Voir `g`.
    pub ps: Vec<f64>,
    /// Inconnues des encoches et isthmes (blocs de `nb`).
    pub x: Vec<f64>,
    /// Densité de courant dans chaque encoche [A/m²].
    pub j: Vec<f64>,
}

impl Section {
    /// Assemble et factorise le système du tronçon.
    pub fn new(g: SectionGeom, h: Harmonics) -> Result<Section, String> {
        let c = g.coord;
        let q = g.slot_center.len();
        if q == 0 || g.beta_o <= 0.0 || g.beta_s < g.beta_o - 1e-12 || g.beta_s * q as f64 >= 2.0 * PI {
            return Err("encoches incohérentes (ouverture ≤ largeur, somme < 360°)".into());
        }
        let gap = region(c, g.y_s, g.y_m);
        let mag = region(c, g.y_m, g.y_r);
        let isth = region(c, g.y_s, g.y_ib);
        let slot = region(c, g.y_ib, g.y_sb);
        let (nh, ks, ko) = (h.nh, h.ks, h.ko);
        let (ys, ym, yr) = (g.y_s, g.y_m, g.y_r);
        // --- entrefer + aimants : système 4×4 par harmonique (inconnues c, d, g, h)
        let mut harm = vec![Harm { inv: [[0.0; 4]; 4], alpha: 0.0 }];
        for n in 1..=nh {
            let e = n as f64;
            let (g1s, g1s_, g2s, g2s_) = gap.eval(e, ys);
            let (g1m, g1m_, g2m, g2m_) = gap.eval(e, ym);
            let (m1m, m1m_, m2m, m2m_) = mag.eval(e, ym);
            let (_, m1r_, _, m2r_) = mag.eval(e, yr);
            let mur = g.mur;
            let m = [
                [g1s_, g2s_, 0.0, 0.0],
                [g1m, g2m, -m1m, -m2m],
                [g1m_, g2m_, -m1m_ / mur, -m2m_ / mur],
                [0.0, 0.0, m1r_, m2r_],
            ];
            let lu = Lu::new(4, m.iter().flatten().copied().collect())?;
            let mut inv = [[0.0; 4]; 4];
            for jj in 0..4 {
                let mut ev = [0.0; 4];
                ev[jj] = 1.0;
                let col = lu.solve(&ev);
                for ii in 0..4 {
                    inv[ii][jj] = col[ii];
                }
            }
            let alpha = g1s * inv[0][0] + g2s * inv[1][0];
            harm.push(Harm { inv, alpha });
        }
        // --- intégrales d'ouverture
        let fo = |k: usize| k as f64 * PI / g.beta_o;
        let es = |k: usize| k as f64 * PI / g.beta_s;
        let idx3 = |i: usize, k: usize, n: usize| (i * (ko + 1) + k) * (nh + 1) + n;
        let mut ic = vec![0.0; q * (ko + 1) * (nh + 1)];
        let mut is_ = vec![0.0; q * (ko + 1) * (nh + 1)];
        for i in 0..q {
            let ta = g.slot_center[i] - g.beta_o / 2.0;
            for k in 0..=ko {
                for n in 1..=nh {
                    let nf = n as f64;
                    let cc = icc(fo(k), nf, g.beta_o);
                    let ss = ics(fo(k), nf, g.beta_o);
                    let (sa, ca) = (nf * ta).sin_cos();
                    ic[idx3(i, k, n)] = ca * cc - sa * ss;
                    is_[idx3(i, k, n)] = sa * cc + ca * ss;
                }
            }
        }
        // couplage ouverture ↔ corps : ∫ cos(F_m u) cos(E_k (u + δ)) du sur [0, βo]
        let delta = (g.beta_s - g.beta_o) / 2.0;
        let mut jm = vec![0.0; (ko + 1) * (ks + 1)];
        for m in 0..=ko {
            for k in 0..=ks {
                let (sd, cd) = (es(k) * delta).sin_cos();
                jm[m * (ks + 1) + k] = cd * icc(fo(m), es(k), g.beta_o) - sd * ics(fo(m), es(k), g.beta_o);
            }
        }
        // --- fonctions propres des encoches : G(y_ib) = 1, G'(y_sb) = 0
        let mut gd = vec![0.0; ks + 1];
        let mut gavg = vec![0.0; ks + 1];
        let (ylo, yhi) = (slot.lo, slot.hi);
        for k in 1..=ks {
            let e = es(k);
            let (_, u1b_, _, u2b_) = slot.eval(e, g.y_sb);
            let (a1, a2) = (-u2b_, u1b_);
            let gv = |y: f64| {
                let (u1, u1_, u2, u2_) = slot.eval(e, y);
                (a1 * u1 + a2 * u2, a1 * u1_ + a2 * u2_)
            };
            let g0 = gv(g.y_ib).0;
            gd[k] = gv(g.y_ib).1 / g0;
            // moyenne sur la surface de l'encoche (poids y en polaire, 1 en plan)
            let (mut num, mut den) = (0.0, 0.0);
            let np = 32;
            for p in 0..np {
                let a = ylo + (yhi - ylo) * p as f64 / np as f64;
                let hp = (yhi - ylo) / np as f64;
                for &(xg, wg) in &GL8 {
                    let y = a + (xg + 1.0) * hp / 2.0;
                    let wgt = wg * hp / 2.0 * if matches!(c, Coord::Polar) { y } else { 1.0 };
                    num += gv(y).0 / g0 * wgt;
                    den += wgt;
                }
            }
            gavg[k] = num / den;
        }
        // --- système : par encoche [a_0..a_ks | c_0..c_ko | d_0..d_ko]
        let nb = (ks + 1) + 2 * (ko + 1);
        let ns = q * nb;
        let mut mat = vec![0.0; ns * ns];
        let ia = |i: usize, k: usize| i * nb + k;
        let icx = |i: usize, k: usize| i * nb + ks + 1 + k;
        let idx = |i: usize, k: usize| i * nb + ks + 1 + ko + 1 + k;
        let wo = |m: usize| if m == 0 { g.beta_o } else { g.beta_o / 2.0 };
        let ws = |m: usize| if m == 0 { g.beta_s } else { g.beta_s / 2.0 };
        // valeurs des fonctions d'isthme aux deux bords
        let iv = |k: usize, y: f64| isth.eval(fo(k), y);
        // noyau de couplage par l'entrefer : Σn α_n/π (IcIc + IsIs)
        for i in 0..q {
            for m in 0..=ko {
                let row_e1 = (i * nb + ks + 1 + m) * ns; // ligne E1 (continuité de A en y_s)
                let row_e2 = (i * nb + ks + 1 + ko + 1 + m) * ns; // ligne E2 (continuité de A en y_ib)
                let (v1s, _, v2s, _) = iv(m, g.y_s);
                let (v1b, _, v2b, _) = iv(m, g.y_ib);
                mat[row_e1 + icx(i, m)] += wo(m) * v1s;
                mat[row_e1 + idx(i, m)] += wo(m) * v2s;
                mat[row_e2 + icx(i, m)] += wo(m) * v1b;
                mat[row_e2 + idx(i, m)] += wo(m) * v2b;
                // E2 : − ∫ A_slot(y_ib) cos(F_m ·) = − a_0 βo δm0 − Σk a_k Jm(m,k)
                for k in 0..=ks {
                    mat[row_e2 + ia(i, k)] -= if k == 0 { if m == 0 { g.beta_o } else { 0.0 } } else { jm[m * (ks + 1) + k] };
                }
                // E1 : − Σn α_n [s_n^c Ic(i,m,n) + s_n^s Is(i,m,n)]
                for jj in 0..q {
                    for k in 0..=ko {
                        let mut kern = 0.0;
                        for n in 1..=nh {
                            kern += harm[n].alpha * (ic[idx3(jj, k, n)] * ic[idx3(i, m, n)] + is_[idx3(jj, k, n)] * is_[idx3(i, m, n)]);
                        }
                        kern /= PI;
                        let (_, v1_, _, v2_) = iv(k, g.y_s);
                        mat[row_e1 + icx(jj, k)] -= kern * v1_;
                        mat[row_e1 + idx(jj, k)] -= kern * v2_;
                    }
                }
            }
            // E3 : continuité de Hθ côté encoche, projetée sur ses fonctions propres
            for mp in 0..=ks {
                let row = (i * nb + mp) * ns;
                if mp >= 1 {
                    mat[row + ia(i, mp)] += gd[mp] * ws(mp);
                }
                for k in 0..=ko {
                    let (_, v1_, _, v2_) = iv(k, g.y_ib);
                    let jv = jm[k * (ks + 1) + mp];
                    mat[row + icx(i, k)] -= v1_ * jv;
                    mat[row + idx(i, k)] -= v2_ * jv;
                }
            }
            // jauge des isthmes k = 0 : c_0 apparaît dans E1/E2 ; rien à ajouter
        }
        let lu = Lu::new(ns, mat)?;
        Ok(Section { g, h, gap, mag, isth, slot, harm, ic, is_, gavg, lu, nb })
    }

    fn idx3(&self, i: usize, k: usize, n: usize) -> usize {
        (i * (self.h.ko + 1) + k) * (self.h.nh + 1) + n
    }

    /// Rayon « métrique » ρ utilisé dans Br = (1/ρ)∂A/∂θ.
    /// Rayon (polaire) ou rayon moyen de la tranche (plane) à la cote `y`.
    pub fn rho(&self, y: f64) -> f64 {
        match self.g.coord {
            Coord::Polar => y,
            Coord::Planar { r0 } => r0,
        }
    }

    /// Solution particulière du courant d'encoche (dérivée nulle au fond) : (P, P').
    fn pj(&self, j: f64, y: f64) -> (f64, f64) {
        let yb = self.g.y_sb;
        match self.g.coord {
            Coord::Polar => (-MU0 * j / 4.0 * (y * y - 2.0 * yb * yb * y.ln()), -MU0 * j / 4.0 * (2.0 * y - 2.0 * yb * yb / y)),
            Coord::Planar { .. } => (-MU0 * j / 2.0 * (y - yb).powi(2), -MU0 * j * (y - yb)),
        }
    }

    /// Valeur moyenne de la solution particulière de courant dans l'encoche.
    fn pj_avg(&self, j: f64) -> f64 {
        let (lo, hi) = (self.slot.lo, self.slot.hi);
        let (mut num, mut den) = (0.0, 0.0);
        for &(xg, wg) in &GL8 {
            let y = lo + (xg + 1.0) * (hi - lo) / 2.0;
            let w = wg * if matches!(self.g.coord, Coord::Polar) { y } else { 1.0 };
            num += self.pj(j, y).0 * w;
            den += w;
        }
        num / den
    }

    /// Surface d'une encoche [m²] (corps).
    pub fn slot_area(&self) -> f64 {
        let (lo, hi) = (self.slot.lo, self.slot.hi);
        match self.g.coord {
            Coord::Polar => self.g.beta_s / 2.0 * (hi * hi - lo * lo),
            Coord::Planar { r0 } => self.g.beta_s * r0 * (hi - lo),
        }
    }

    /// Particulière des aimants pour l'harmonique n (source q) : (P(y_m), P'(y_m), P'(y_r), amplitude).
    fn mag_part(&self, n: usize, q: f64) -> (f64, f64, f64, f64) {
        let (ym, yr) = (self.g.y_m, self.g.y_r);
        match self.g.coord {
            Coord::Polar => {
                if n == 1 {
                    let a = -MU0 * q / 2.0;
                    (a * ym * ym.ln(), a * (ym.ln() + 1.0), a * (yr.ln() + 1.0), a)
                } else {
                    let a = MU0 * q / ((n * n) as f64 - 1.0);
                    (a * ym, a, a, a)
                }
            }
            Coord::Planar { r0 } => {
                let a = MU0 * q * r0 / (n * n) as f64;
                (a, 0.0, 0.0, a)
            }
        }
    }

    /// Résout pour une aimantation (repère stator) et des densités de courant par encoche.
    pub fn solve(&self, m: &MagFourier, j: &[f64]) -> SectionField {
        let (nh, ks, ko, nb) = (self.h.nh, self.h.ks, self.h.ko, self.nb);
        let q = self.g.slot_center.len();
        let polar = matches!(self.g.coord, Coord::Polar);
        let mut vc = vec![[0.0; 4]; nh + 1];
        let mut vs = vec![[0.0; 4]; nh + 1];
        let (mut gc, mut gs) = (vec![0.0; nh + 1], vec![0.0; nh + 1]);
        let (mut pc, mut ps) = (vec![0.0; nh + 1], vec![0.0; nh + 1]);
        for n in 1..=nh.min(m.mr_c.len().saturating_sub(1)) {
            let nf = n as f64;
            // ∇²A = −(μ0/ρ)·q ; en plan, Mθ ne crée pas de source volumique
            let (qc, qs) = if polar { (m.mt_c[n] - nf * m.mr_s[n], m.mt_s[n] + nf * m.mr_c[n]) } else { (-nf * m.mr_s[n], nf * m.mr_c[n]) };
            for (qq, mt, v, gg, pp) in [(qc, m.mt_c[n], &mut vc[n], &mut gc[n], &mut pc[n]), (qs, m.mt_s[n], &mut vs[n], &mut gs[n], &mut ps[n])] {
                let (p_m, dp_m, dp_r, amp) = self.mag_part(n, qq);
                let rhs = [0.0, p_m, (dp_m + MU0 * mt) / self.g.mur, -MU0 * mt - dp_r];
                *v = mat4(&self.harm[n].inv, &rhs);
                let (g1s, _, g2s, _) = self.gap.eval(nf, self.g.y_s);
                *gg = g1s * v[0] + g2s * v[1];
                *pp = amp;
            }
        }
        // second membre
        let ns = q * nb;
        let mut rhs = vec![0.0; ns];
        for i in 0..q {
            for mm in 0..=ko {
                let mut s = 0.0;
                for n in 1..=nh {
                    s += gc[n] * self.ic[self.idx3(i, mm, n)] + gs[n] * self.is_[self.idx3(i, mm, n)];
                }
                rhs[i * nb + ks + 1 + mm] = s;
            }
            let ji = j.get(i).copied().unwrap_or(0.0);
            if ji != 0.0 {
                let (p_b, dp_b) = self.pj(ji, self.g.y_ib);
                // E2 m = 0 : + P_J(y_ib)·βo ; E3 m' = 0 : − P_J'(y_ib)·βs
                rhs[i * nb + ks + 1 + ko + 1] += p_b * self.g.beta_o;
                rhs[i * nb] -= dp_b * self.g.beta_s;
            }
        }
        let x = self.lu.solve(&rhs);
        // dérivées en y_s → coefficients d'entrefer et d'aimants
        let mut out = SectionField {
            c: vec![0.0; nh + 1],
            d: vec![0.0; nh + 1],
            e: vec![0.0; nh + 1],
            f: vec![0.0; nh + 1],
            g: vec![0.0; nh + 1],
            hh: vec![0.0; nh + 1],
            pc,
            gs: vec![0.0; nh + 1],
            hs: vec![0.0; nh + 1],
            ps,
            x,
            j: (0..q).map(|i| j.get(i).copied().unwrap_or(0.0)).collect(),
        };
        let fo = |k: usize| k as f64 * PI / self.g.beta_o;
        let dvs: Vec<(f64, f64)> = (0..=ko).map(|k| { let (_, a, _, b) = self.isth.eval(fo(k), self.g.y_s); (a, b) }).collect();
        for n in 1..=nh {
            let (mut sc, mut ss) = (0.0, 0.0);
            for jj in 0..q {
                for k in 0..=ko {
                    let der = out.x[jj * nb + ks + 1 + k] * dvs[k].0 + out.x[jj * nb + ks + 1 + ko + 1 + k] * dvs[k].1;
                    sc += der * self.ic[self.idx3(jj, k, n)];
                    ss += der * self.is_[self.idx3(jj, k, n)];
                }
            }
            sc /= PI;
            ss /= PI;
            let inv = &self.harm[n].inv;
            out.c[n] = inv[0][0] * sc + vc[n][0];
            out.d[n] = inv[1][0] * sc + vc[n][1];
            out.g[n] = inv[2][0] * sc + vc[n][2];
            out.hh[n] = inv[3][0] * sc + vc[n][3];
            out.e[n] = inv[0][0] * ss + vs[n][0];
            out.f[n] = inv[1][0] * ss + vs[n][1];
            out.gs[n] = inv[2][0] * ss + vs[n][2];
            out.hs[n] = inv[3][0] * ss + vs[n][3];
        }
        out
    }

    /// Coefficients (Br_c, Br_s, Bθ_c, Bθ_s) dans l'entrefer à la cote y.
    pub fn gap_b_coeffs(&self, f: &SectionField, y: f64) -> [Vec<f64>; 4] {
        let nh = self.h.nh;
        let rho = self.rho(y);
        let mut o = [vec![0.0; nh + 1], vec![0.0; nh + 1], vec![0.0; nh + 1], vec![0.0; nh + 1]];
        for n in 1..=nh {
            let nf = n as f64;
            let (u1, u1_, u2, u2_) = self.gap.eval(nf, y);
            let ac = f.c[n] * u1 + f.d[n] * u2;
            let as_ = f.e[n] * u1 + f.f[n] * u2;
            o[0][n] = nf * as_ / rho;
            o[1][n] = -nf * ac / rho;
            o[2][n] = -(f.c[n] * u1_ + f.d[n] * u2_);
            o[3][n] = -(f.e[n] * u1_ + f.f[n] * u2_);
        }
        o
    }

    /// Couple exercé sur le côté « intérieur » du contour (stator si le stator
    /// est du côté des y faibles) [N·m], tenseur de Maxwell à la cote y.
    pub fn maxwell_torque(&self, f: &SectionField, y: f64) -> f64 {
        let b = self.gap_b_coeffs(f, y);
        let s: f64 = (1..=self.h.nh).map(|n| b[0][n] * b[2][n] + b[1][n] * b[3][n]).sum();
        let rho = self.rho(y);
        self.g.depth * rho * rho / MU0 * PI * s
    }

    /// Cote du milieu de l'entrefer.
    pub fn y_mid_gap(&self) -> f64 {
        0.5 * (self.g.y_s + self.g.y_m)
    }

    /// Le stator est-il du côté des y faibles (rotor extérieur / tranche axiale) ?
    pub fn stator_below(&self) -> bool {
        self.g.y_s < self.g.y_m
    }

    /// Couple sur le ROTOR [N·m].
    pub fn rotor_torque(&self, f: &SectionField) -> f64 {
        let t = self.maxwell_torque(f, self.y_mid_gap());
        if self.stator_below() { -t } else { t }
    }

    /// Br, Bθ dans l'entrefer sur `nt` points.
    pub fn gap_b(&self, f: &SectionField, y: f64, nt: usize) -> (Vec<f64>, Vec<f64>) {
        let b = self.gap_b_coeffs(f, y);
        let mut br = vec![0.0; nt];
        let mut bt = vec![0.0; nt];
        for t in 0..nt {
            let th = 2.0 * PI * t as f64 / nt as f64;
            let (s1, c1) = th.sin_cos();
            let (mut sn, mut cn) = (s1, c1);
            for n in 1..=self.h.nh {
                br[t] += b[0][n] * cn + b[1][n] * sn;
                bt[t] += b[2][n] * cn + b[3][n] * sn;
                let c2 = cn * c1 - sn * s1;
                sn = sn * c1 + cn * s1;
                cn = c2;
            }
        }
        (br, bt)
    }

    /// Force résultante sur le ROTOR [N] (composantes x, y du plan ; en tranche
    /// axiale : (force tangentielle, force d'attraction axiale)).
    pub fn rotor_force(&self, f: &SectionField) -> (f64, f64) {
        let y = self.y_mid_gap();
        let nt = 4 * self.h.nh + 64;
        let (br, bt) = self.gap_b(f, y, nt);
        let dth = 2.0 * PI / nt as f64;
        let rho = self.rho(y);
        let k = self.g.depth * rho;
        let sgn = if self.stator_below() { -1.0 } else { 1.0 };
        match self.g.coord {
            Coord::Polar => {
                let (mut fx, mut fy) = (0.0, 0.0);
                for t in 0..nt {
                    let (s, c) = (t as f64 * dth).sin_cos();
                    let pn = (br[t] * br[t] - bt[t] * bt[t]) / (2.0 * MU0);
                    let pt = br[t] * bt[t] / MU0;
                    fx += (pn * c - pt * s) * dth;
                    fy += (pn * s + pt * c) * dth;
                }
                (sgn * k * fx, sgn * k * fy)
            }
            Coord::Planar { .. } => {
                // tranche : force tangentielle (couple / r0) et attraction normale
                let (mut ft, mut fnrm) = (0.0, 0.0);
                for t in 0..nt {
                    ft += br[t] * bt[t] / MU0 * dth;
                    fnrm += (br[t] * br[t] - bt[t] * bt[t]) / (2.0 * MU0) * dth;
                }
                (sgn * k * ft, sgn * k * fnrm)
            }
        }
    }

    /// Potentiel moyen sur une encoche : `side` 0 = moitié gauche, 1 = droite, 2 = entière.
    pub fn slot_mean_a(&self, f: &SectionField, i: usize, side: usize) -> f64 {
        let ks = self.h.ks;
        let base = i * self.nb;
        let mut s = f.x[base] + self.pj_avg(f.j[i]);
        if side < 2 {
            for k in 1..=ks {
                let kb2 = k as f64 * PI / 2.0;
                let hk = (k as f64 * PI / 2.0).sin() / kb2;
                let sg = if side == 0 { 1.0 } else { -1.0 };
                s += f.x[base + k] * self.gavg[k] * sg * hk;
            }
        }
        s
    }

    /// A dans l'entrefer.
    pub fn gap_a(&self, f: &SectionField, y: f64, th: f64) -> f64 {
        let mut s = 0.0;
        for n in 1..=self.h.nh {
            let nf = n as f64;
            let (u1, _, u2, _) = self.gap.eval(nf, y);
            let (sn, cn) = (nf * th).sin_cos();
            s += (f.c[n] * u1 + f.d[n] * u2) * cn + (f.e[n] * u1 + f.f[n] * u2) * sn;
        }
        s
    }

    /// A en un point quelconque (None dans le fer).
    pub fn a_at(&self, f: &SectionField, y: f64, th: f64) -> Option<f64> {
        let (nh, ks, ko, nb) = (self.h.nh, self.h.ks, self.h.ko, self.nb);
        let inr = |b: &Basis| y >= b.lo - 1e-15 && y <= b.hi + 1e-15;
        if inr(&self.gap) {
            return Some(self.gap_a(f, y, th));
        }
        if inr(&self.mag) {
            let mut s = 0.0;
            for n in 1..=nh {
                let nf = n as f64;
                let (u1, _, u2, _) = self.mag.eval(nf, y);
                let pr = match self.g.coord {
                    Coord::Polar => if n == 1 { y * y.ln() } else { y },
                    Coord::Planar { .. } => 1.0,
                };
                let (sn, cn) = (nf * th).sin_cos();
                s += (f.g[n] * u1 + f.hh[n] * u2 + f.pc[n] * pr) * cn + (f.gs[n] * u1 + f.hs[n] * u2 + f.ps[n] * pr) * sn;
            }
            return Some(s);
        }
        let tw = th.rem_euclid(2.0 * PI);
        for i in 0..self.g.slot_center.len() {
            let d = ((tw - self.g.slot_center[i] + PI).rem_euclid(2.0 * PI)) - PI;
            if inr(&self.isth) && d.abs() <= self.g.beta_o / 2.0 {
                let u = d + self.g.beta_o / 2.0;
                let mut s = 0.0;
                for k in 0..=ko {
                    let e = k as f64 * PI / self.g.beta_o;
                    let (v1, _, v2, _) = self.isth.eval(e, y);
                    s += (f.x[i * nb + ks + 1 + k] * v1 + f.x[i * nb + ks + 1 + ko + 1 + k] * v2) * (e * u).cos();
                }
                return Some(s);
            }
            if inr(&self.slot) && d.abs() <= self.g.beta_s / 2.0 {
                let u = d + self.g.beta_s / 2.0;
                let mut s = f.x[i * nb] + self.pj(f.j[i], y).0;
                for k in 1..=ks {
                    let e = k as f64 * PI / self.g.beta_s;
                    let (_, u1b_, _, u2b_) = self.slot.eval(e, self.g.y_sb);
                    let (a1, a2) = (-u2b_, u1b_);
                    let gval = |yy: f64| { let (u1, _, u2, _) = self.slot.eval(e, yy); a1 * u1 + a2 * u2 };
                    s += f.x[i * nb + k] * gval(y) / gval(self.g.y_ib) * (e * u).cos();
                }
                return Some(s);
            }
        }
        None
    }
}

fn mat4(m: &[[f64; 4]; 4], v: &[f64; 4]) -> [f64; 4] {
    let mut o = [0.0; 4];
    for i in 0..4 {
        o[i] = m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2] + m[i][3] * v[3];
    }
    o
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::MotorSpec;
    use crate::magnets;
    use crate::legacy::{Model, Resolution};

    /// Tronçon polaire équivalent à l'ancien modèle (rotor extérieur, encoches
    /// droites) : isthme de très faible épaisseur et de même largeur.
    fn equivalent(spec: &MotorSpec, nh: usize, ks: usize) -> Section {
        let q = spec.n_slots;
        let rs = spec.r_bore();
        let beta = spec.slot_open * 1e-3 / rs;
        let eps = 1e-7;
        let g = SectionGeom {
            coord: Coord::Polar,
            y_sb: spec.r_slot_bottom(),
            y_ib: rs - eps,
            y_s: rs,
            y_m: spec.r_magnet_in(),
            y_r: spec.r_magnet_out(),
            slot_center: (0..q).map(|i| (i as f64 + 0.5) * 2.0 * PI / q as f64).collect(),
            beta_s: beta,
            beta_o: beta,
            mur: spec.mur_magnet,
            depth: spec.length(),
        };
        Section::new(g, Harmonics { nh, ks, ko: ks }).unwrap()
    }

    /// La tranche plane est la limite de la géométrie polaire à grand rayon :
    /// l'écart doit décroître comme 1/R.
    #[test]
    fn plan_limite_du_polaire() {
        let spec = MotorSpec::irregular_example();
        let pieces = magnets::pieces(&spec);
        let mf = magnets::fourier(&pieces, spec.br_magnet, 300);
        let q = 12;
        let ecart = |r0: f64| {
            // mêmes épaisseurs, mêmes angles, rayon r0
            let mk = |coord| SectionGeom {
                coord,
                y_sb: r0 - 5e-3,
                y_ib: r0 - 0.5e-3,
                y_s: r0,
                y_m: r0 + 0.2e-3,
                y_r: r0 + 1.8e-3,
                slot_center: (0..q).map(|i| (i as f64 + 0.5) * 2.0 * PI / q as f64).collect(),
                beta_s: 0.3,
                beta_o: 0.12,
                mur: 1.05,
                depth: 7e-3,
            };
            let h = Harmonics { nh: 300, ks: 16, ko: 12 };
            let pol = Section::new(mk(Coord::Polar), h).unwrap();
            let pla = Section::new(mk(Coord::Planar { r0 }), h).unwrap();
            let m = mf.rotated(0.3);
            let (tp, tl) = (pol.rotor_torque(&pol.solve(&m, &[])), pla.rotor_torque(&pla.solve(&m, &[])));
            (tp - tl).abs() / tp.abs()
        };
        let (e1, e2) = (ecart(0.1), ecart(1.0));
        assert!(e2 < 0.01, "écart à R = 1 m : {e2}");
        assert!(e2 < e1 / 5.0, "convergence : {e1} → {e2}");
    }

    #[test]
    fn retrouve_l_ancien_modele() {
        for v in ["regulier", "irregulier"] {
            let spec = MotorSpec::example(v);
            let old = Model::new(&spec, Resolution { nh: 200, nk: 12 }).unwrap();
            let sec = equivalent(&spec, 200, 12);
            let mf = magnets::fourier(&magnets::pieces(&spec), spec.br_magnet, 200);
            for th in [0.1f64, 0.7, 2.3] {
                let t_old = old.torque(&old.solve(th.to_radians()));
                let f = sec.solve(&mf.rotated(th.to_radians()), &[]);
                let t_new = sec.rotor_torque(&f);
                assert!((t_new - t_old).abs() < 1e-4 * t_old.abs().max(1e-3), "{v} θ={th}: {t_new} vs {t_old}");
            }
        }
    }
}
