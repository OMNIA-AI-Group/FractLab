//! Modèle analytique en sous-domaines, coordonnées POLAIRES exactes,
//! machine à flux radial à ROTOR EXTERNE (stator encoché à l'intérieur).
//!
//! Régions (de l'axe vers l'extérieur) :
//!   encoches i  : Rsb ≤ r ≤ Rs, θ ∈ [θi − β/2, θi + β/2]   (Laplace)
//!   entrefer    : Rs  ≤ r ≤ Rm                              (Laplace)
//!   aimants     : Rm  ≤ r ≤ Rr                              (Poisson, M(θ))
//!   fer stator et fer rotor : perméabilité infinie (Hθ = 0, ou Hr = 0 sur
//!   les flancs d'encoche).
//! Potentiel vecteur A_z ; B = rot A ; séries de Fourier sur 360° complets
//! (aucune hypothèse de périodicité : rotors apériodiques admis).
//!
//! Hypothèses de cette version (v0.1) : fer linéaire infiniment perméable,
//! encoches droites de largeur = ouverture `slot_open` (pas de becs de dents),
//! fonctionnement à vide. Validation : voir `fdcheck.rs` (solveur numérique
//! indépendant) et les tests de cohérence en bas de fichier.

use crate::geometry::MotorSpec;
use crate::linalg::Lu;
use crate::magnets::{self, MagFourier, MU0};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
/// Nombre d'harmoniques retenus dans les séries de Fourier.
pub struct Resolution {
    /// harmoniques d'espace dans l'entrefer / les aimants
    pub nh: usize,
    /// harmoniques par encoche
    pub nk: usize,
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution { nh: 400, nk: 24 }
    }
}

/// Solution d'une harmonique n : coefficients (c, d, g, h) = s·u + v.
#[derive(Clone, Copy)]
struct Harm {
    inv: [[f64; 4]; 4], // inverse du système 4×4
    rho: f64,           // (Rs/Rm)^n
    alpha: f64,         // A_II(Rs) par unité de dérivée s
}

/// Modèle analytique prêt à résoudre : matrices factorisées une fois pour toutes,
/// puis `solve` pour chaque position du rotor.
pub struct Model {
    /// Spécification du moteur modélisé.
    pub spec: MotorSpec,
    /// Résolution harmonique utilisée.
    pub res: Resolution,
    rs: f64,
    rm: f64,
    rr: f64,
    rsb: f64,
    mur: f64,
    /// Nombre d'encoches.
    pub q: usize,
    /// Ouverture angulaire d'une encoche [rad].
    pub beta: f64,
    /// bord gauche (angle de départ) de chaque encoche
    pub slot_start: Vec<f64>,
    harm: Vec<Harm>,
    lam: Vec<f64>,    // λk = dérivée normalisée en Rs
    ic: Vec<f64>,     // Ic[(i, k, n)]
    is_: Vec<f64>,    // Is[(i, k, n)]
    lu: Lu,
    mag: MagFourier, // aimantation, repère rotor
    gbar: Vec<f64>,  // moyenne radiale normalisée de Gk dans l'encoche
}

/// Champ résolu pour une position du rotor.
pub struct Field {
    /// Position du rotor [rad].
    pub theta_r: f64,
    /// entrefer : cos (c, d), sin (e, f) pour n = 1..=nh (indice 0 inutilisé)
    pub c: Vec<f64>,
    /// Entrefer : coefficients cosinus en (r/Rs)^−n.
    pub d: Vec<f64>,
    /// Entrefer : coefficients sinus en (r/Rm)^n.
    pub e: Vec<f64>,
    /// Entrefer : coefficients sinus en (r/Rs)^−n.
    pub f: Vec<f64>,
    /// encoches : a[i*(K+1)+k]
    pub a: Vec<f64>,
    /// aimants : cos (g, h, particulière), sin (gs, hs, particulière)
    pub g: Vec<f64>,
    /// Aimants : coefficients cosinus en (r/Rm)^−n.
    pub h: Vec<f64>,
    /// Aimants : amplitude cosinus de la solution particulière.
    pub pc: Vec<f64>,
    /// Aimants : coefficients sinus en (r/Rr)^n.
    pub gs: Vec<f64>,
    /// Aimants : coefficients sinus en (r/Rm)^−n.
    pub hs: Vec<f64>,
    /// Aimants : amplitude sinus de la solution particulière.
    pub ps: Vec<f64>,
}

fn sinc_int(x: f64, b: f64) -> f64 {
    // ∫0^b cos(x u) du = sin(x b)/x
    if (x * b).abs() < 1e-8 { b } else { (x * b).sin() / x }
}
fn cosc_int(x: f64, b: f64) -> f64 {
    // ∫0^b sin(x u) du = (1 − cos(x b))/x
    if (x * b).abs() < 1e-8 { x * b * b / 2.0 } else { (1.0 - (x * b).cos()) / x }
}

impl Model {
    /// Construit et factorise le modèle (indépendant de la position du rotor).
    pub fn new(spec: &MotorSpec, res: Resolution) -> Result<Model, String> {
        let rs = spec.r_bore();
        let rm = spec.r_magnet_in();
        let rr = spec.r_magnet_out();
        let rsb = spec.r_slot_bottom();
        if !(rsb < rs && rs < rm && rm < rr) {
            return Err("rayons incohérents (fond d'encoche < alésage < aimants)".into());
        }
        let q = spec.n_slots;
        let beta = spec.slot_open * 1e-3 / rs;
        if beta <= 0.0 || beta >= 2.0 * PI / q as f64 {
            return Err("ouverture d'encoche hors bornes".into());
        }
        let mur = spec.mur_magnet;
        let nh = res.nh;
        let nk = res.nk;
        // Encoches centrées en (i + 1/2)·2π/Q : la dent 0 est centrée en θ = 0.
        let slot_start: Vec<f64> =
            (0..q).map(|i| (i as f64 + 0.5) * 2.0 * PI / q as f64 - beta / 2.0).collect();

        // --- harmoniques entrefer + aimants ---
        let mut harm = Vec::with_capacity(nh + 1);
        harm.push(Harm { inv: [[0.0; 4]; 4], rho: 0.0, alpha: 0.0 });
        for n in 1..=nh {
            let nf = n as f64;
            let rho = (rs / rm).powf(nf);
            let sig = (rm / rr).powf(nf);
            // inconnues (c, d, g, h)
            let m = [
                [nf / rs * rho, -nf / rs, 0.0, 0.0],
                [1.0, rho, -sig, -1.0],
                [nf / rm, -nf / rm * rho, -nf / (rm * mur) * sig, nf / (rm * mur)],
                [0.0, 0.0, nf / rr, -nf / rr * sig],
            ];
            let flat: Vec<f64> = m.iter().flatten().copied().collect();
            let lu = Lu::new(4, flat)?;
            let mut inv = [[0.0; 4]; 4];
            for j in 0..4 {
                let mut e = [0.0; 4];
                e[j] = 1.0;
                let col = lu.solve(&e);
                for i in 0..4 {
                    inv[i][j] = col[i];
                }
            }
            let alpha = rho * inv[0][0] + inv[1][0];
            harm.push(Harm { inv, rho, alpha });
        }

        // --- encoches ---
        let ek = |k: usize| k as f64 * PI / beta;
        let lam: Vec<f64> = (0..=nk)
            .map(|k| {
                if k == 0 {
                    return 0.0;
                }
                let e = ek(k);
                let tau = (rsb / rs).powf(2.0 * e);
                e / rs * (1.0 - tau) / (1.0 + tau)
            })
            .collect();
        // moyenne radiale (pondérée r dr) de Gk(r)/Gk(Rs), Gauss 8 × 8 panneaux
        let gbar: Vec<f64> = (0..=nk)
            .map(|k| {
                if k == 0 {
                    return 1.0;
                }
                let e = ek(k);
                let g = |r: f64| (r / rs).powf(e) + (rsb / rs).powf(e) * (rsb / r).powf(e);
                let g_rs = 1.0 + (rsb / rs).powf(2.0 * e);
                let panels = 16;
                let h = (rs - rsb) / panels as f64;
                let mut num = 0.0;
                for p in 0..panels {
                    let a = rsb + p as f64 * h;
                    for &(x, w) in &GL8 {
                        let r = a + (x + 1.0) * h / 2.0;
                        num += g(r) * r * w * h / 2.0;
                    }
                }
                num / ((rs * rs - rsb * rsb) / 2.0) / g_rs
            })
            .collect();

        let idx3 = |i: usize, k: usize, n: usize| (i * (nk + 1) + k) * (nh + 1) + n;
        let mut ic = vec![0.0; q * (nk + 1) * (nh + 1)];
        let mut is_ = vec![0.0; q * (nk + 1) * (nh + 1)];
        for i in 0..q {
            let ta = slot_start[i];
            for k in 0..=nk {
                let e = ek(k);
                for n in 1..=nh {
                    let nf = n as f64;
                    // C = ∫0^β cos(Eu) cos(nu), S = ∫0^β cos(Eu) sin(nu)
                    let cc = 0.5 * (sinc_int(e - nf, beta) + sinc_int(e + nf, beta));
                    let ss = 0.5 * (cosc_int(nf - e, beta) + cosc_int(nf + e, beta));
                    let (sa, ca) = (nf * ta).sin_cos();
                    ic[idx3(i, k, n)] = ca * cc - sa * ss;
                    is_[idx3(i, k, n)] = sa * cc + ca * ss;
                }
            }
        }

        // --- système des encoches (indépendant de la position rotor) ---
        let ns = q * (nk + 1);
        let mut mat = vec![0.0; ns * ns];
        for i in 0..q {
            for m in 0..=nk {
                let row = i * (nk + 1) + m;
                let wm = if m == 0 { beta } else { beta / 2.0 };
                mat[row * ns + row] += wm;
                for j in 0..q {
                    for k in 1..=nk {
                        let col = j * (nk + 1) + k;
                        let mut sum = 0.0;
                        for n in 1..=nh {
                            sum += harm[n].alpha
                                * (ic[idx3(j, k, n)] * ic[idx3(i, m, n)]
                                    + is_[idx3(j, k, n)] * is_[idx3(i, m, n)]);
                        }
                        mat[row * ns + col] -= sum * lam[k] / PI;
                    }
                }
            }
        }
        let lu = Lu::new(ns, mat)?;
        let mag = magnets::fourier(&magnets::pieces(spec), spec.br_magnet, nh);
        Ok(Model {
            spec: spec.clone(),
            res,
            rs,
            rm,
            rr,
            rsb,
            mur,
            q,
            beta,
            slot_start,
            harm,
            lam,
            ic,
            is_,
            lu,
            mag,
            gbar,
        })
    }

    fn idx3(&self, i: usize, k: usize, n: usize) -> usize {
        (i * (self.res.nk + 1) + k) * (self.res.nh + 1) + n
    }

    /// Second membre « aimants » d'une harmonique (partie cos ou sin).
    fn source_rhs(&self, n: usize, q_src: f64, mt: f64) -> [f64; 4] {
        let nf = n as f64;
        // particulière : P = a r (n ≠ 1) ; P = a r ln r (n = 1)
        let (p_rm, dp_rm, dp_rr) = if n == 1 {
            let a = -MU0 * q_src / 2.0;
            (a * self.rm * self.rm.ln(), a * (self.rm.ln() + 1.0), a * (self.rr.ln() + 1.0))
        } else {
            let a = MU0 * q_src / (nf * nf - 1.0);
            (a * self.rm, a, a)
        };
        [0.0, p_rm, (dp_rm + MU0 * mt) / self.mur, -MU0 * mt - dp_rr]
    }

    /// Résout le champ pour une position du rotor `theta_r` [rad].
    pub fn solve(&self, theta_r: f64) -> Field {
        let nh = self.res.nh;
        let nk = self.res.nk;
        let q = self.q;
        let m = self.mag.rotated(theta_r);
        // parties « source » des coefficients (c,d,g,h) et γ
        let mut vc = vec![[0.0; 4]; nh + 1];
        let mut vs = vec![[0.0; 4]; nh + 1];
        let mut gc = vec![0.0; nh + 1];
        let mut gs = vec![0.0; nh + 1];
        let mut pc = vec![0.0; nh + 1];
        let mut ps = vec![0.0; nh + 1];
        for n in 1..=nh {
            let nf = n as f64;
            let h = &self.harm[n];
            // ∇²A = −(μ0/r)(Mθ − ∂Mr/∂θ)
            let qc = m.mt_c[n] - nf * m.mr_s[n];
            let qs = m.mt_s[n] + nf * m.mr_c[n];
            let rc = self.source_rhs(n, qc, m.mt_c[n]);
            let rsn = self.source_rhs(n, qs, m.mt_s[n]);
            pc[n] = particular_amp(n, qc);
            ps[n] = particular_amp(n, qs);
            vc[n] = mat4(&h.inv, &rc);
            vs[n] = mat4(&h.inv, &rsn);
            gc[n] = h.rho * vc[n][0] + vc[n][1];
            gs[n] = h.rho * vs[n][0] + vs[n][1];
        }
        // second membre des encoches
        let ns = q * (nk + 1);
        let mut rhs = vec![0.0; ns];
        for i in 0..q {
            for mm in 0..=nk {
                let mut s = 0.0;
                for n in 1..=nh {
                    s += gc[n] * self.ic[self.idx3(i, mm, n)] + gs[n] * self.is_[self.idx3(i, mm, n)];
                }
                rhs[i * (nk + 1) + mm] = s;
            }
        }
        let a = self.lu.solve(&rhs);
        // dérivées en Rs → coefficients complets de l'entrefer
        let (mut c, mut d, mut e, mut f) =
            (vec![0.0; nh + 1], vec![0.0; nh + 1], vec![0.0; nh + 1], vec![0.0; nh + 1]);
        let (mut g, mut hh, mut g2, mut h2) =
            (vec![0.0; nh + 1], vec![0.0; nh + 1], vec![0.0; nh + 1], vec![0.0; nh + 1]);
        for n in 1..=nh {
            let (mut sc, mut ss) = (0.0, 0.0);
            for j in 0..q {
                for k in 1..=nk {
                    let ajk = a[j * (nk + 1) + k] * self.lam[k];
                    sc += ajk * self.ic[self.idx3(j, k, n)];
                    ss += ajk * self.is_[self.idx3(j, k, n)];
                }
            }
            sc /= PI;
            ss /= PI;
            let h = &self.harm[n];
            c[n] = h.inv[0][0] * sc + vc[n][0];
            d[n] = h.inv[1][0] * sc + vc[n][1];
            e[n] = h.inv[0][0] * ss + vs[n][0];
            f[n] = h.inv[1][0] * ss + vs[n][1];
            g[n] = h.inv[2][0] * sc + vc[n][2];
            hh[n] = h.inv[3][0] * sc + vc[n][3];
            g2[n] = h.inv[2][0] * ss + vs[n][2];
            h2[n] = h.inv[3][0] * ss + vs[n][3];
        }
        Field { theta_r, c, d, e, f, a, g, h: hh, pc, gs: g2, hs: h2, ps }
    }

    /// Coefficients de Fourier de (Br, Bθ) dans l'entrefer au rayon r.
    /// Retourne (br_c, br_s, bt_c, bt_s), indice n.
    pub fn airgap_b_coeffs(&self, fld: &Field, r: f64) -> [Vec<f64>; 4] {
        let nh = self.res.nh;
        let mut out = [vec![0.0; nh + 1], vec![0.0; nh + 1], vec![0.0; nh + 1], vec![0.0; nh + 1]];
        for n in 1..=nh {
            let nf = n as f64;
            let p1 = (r / self.rm).powf(nf);
            let p2 = (r / self.rs).powf(-nf);
            let ac = fld.c[n] * p1 + fld.d[n] * p2;
            let as_ = fld.e[n] * p1 + fld.f[n] * p2;
            let dac = nf / r * (fld.c[n] * p1 - fld.d[n] * p2);
            let das = nf / r * (fld.e[n] * p1 - fld.f[n] * p2);
            out[0][n] = nf * as_ / r; // Br cos
            out[1][n] = -nf * ac / r; // Br sin
            out[2][n] = -dac; // Bθ cos
            out[3][n] = -das; // Bθ sin
        }
        out
    }

    /// Rayon au milieu de l'entrefer [m].
    pub fn r_mid_gap(&self) -> f64 {
        0.5 * (self.rs + self.rm)
    }
    /// Rayons caractéristiques (fond d'encoche, alésage, aimants int., aimants ext.) [m].
    pub fn radii(&self) -> (f64, f64, f64, f64) {
        (self.rsb, self.rs, self.rm, self.rr)
    }

    /// Couple électromagnétique sur le ROTOR [N·m] (tenseur de Maxwell, rayon r).
    pub fn torque_at(&self, fld: &Field, r: f64) -> f64 {
        let b = self.airgap_b_coeffs(fld, r);
        let mut s = 0.0;
        for n in 1..=self.res.nh {
            s += b[0][n] * b[2][n] + b[1][n] * b[3][n];
        }
        // couple sur le stator (contour entourant le stator), rotor = opposé
        -(self.spec.length() * r * r / MU0) * PI * s
    }
    /// Couple sur le rotor [N·m] au milieu de l'entrefer.
    pub fn torque(&self, fld: &Field) -> f64 {
        self.torque_at(fld, self.r_mid_gap())
    }

    /// Induction (Br, Bθ) dans l'entrefer sur `nt` points réguliers en θ.
    pub fn airgap_b(&self, fld: &Field, r: f64, nt: usize) -> (Vec<f64>, Vec<f64>) {
        let b = self.airgap_b_coeffs(fld, r);
        let mut br = vec![0.0; nt];
        let mut bt = vec![0.0; nt];
        for t in 0..nt {
            let th = 2.0 * PI * t as f64 / nt as f64;
            let (s1, c1) = th.sin_cos();
            let (mut sn, mut cn) = (s1, c1);
            let (mut x, mut y) = (0.0, 0.0);
            for n in 1..=self.res.nh {
                x += b[0][n] * cn + b[1][n] * sn;
                y += b[2][n] * cn + b[3][n] * sn;
                let c2 = cn * c1 - sn * s1;
                sn = sn * c1 + cn * s1;
                cn = c2;
            }
            br[t] = x;
            bt[t] = y;
        }
        (br, bt)
    }

    /// Force magnétique résultante sur le ROTOR [N] (Fx, Fy), tenseur de Maxwell.
    pub fn force(&self, fld: &Field) -> (f64, f64) {
        let r = self.r_mid_gap();
        let nt = 4 * self.res.nh + 64;
        let (br, bt) = self.airgap_b(fld, r, nt);
        let (mut fx, mut fy) = (0.0, 0.0);
        let dth = 2.0 * PI / nt as f64;
        for t in 0..nt {
            let th = t as f64 * dth;
            let (s, c) = th.sin_cos();
            let pn = (br[t] * br[t] - bt[t] * bt[t]) / (2.0 * MU0);
            let pt = br[t] * bt[t] / MU0;
            fx += (pn * c - pt * s) * dth;
            fy += (pn * s + pt * c) * dth;
        }
        let k = self.spec.length() * r;
        (-k * fx, -k * fy)
    }

    /// Potentiel vecteur moyen sur la moitié gauche (0) ou droite (1) de l'encoche i.
    pub fn slot_half_mean_a(&self, fld: &Field, i: usize, side: usize) -> f64 {
        let nk = self.res.nk;
        let mut s = fld.a[i * (nk + 1)];
        for k in 1..=nk {
            let ekb2 = k as f64 * PI / 2.0; // Ek·β/2
            let h = (k as f64 * PI / 2.0).sin() / ekb2;
            let sgn = if side == 0 { 1.0 } else { -1.0 };
            s += fld.a[i * (nk + 1) + k] * self.gbar[k] * sgn * h;
        }
        s
    }

    /// Flux par spire [Wb] de la bobine enroulée autour de la dent j
    /// (côtés : moitié droite de l'encoche j−1, moitié gauche de l'encoche j).
    pub fn tooth_flux(&self, fld: &Field, j: usize) -> f64 {
        let q = self.q;
        let left_slot = (j + q - 1) % q;
        let right_slot = j;
        self.spec.length()
            * (self.slot_half_mean_a(fld, left_slot, 1) - self.slot_half_mean_a(fld, right_slot, 0))
    }

    /// Valeur de A dans l'entrefer (r, θ) — utile aux contrôles.
    pub fn airgap_a(&self, fld: &Field, r: f64, th: f64) -> f64 {
        let mut s = 0.0;
        for n in 1..=self.res.nh {
            let nf = n as f64;
            let p1 = (r / self.rm).powf(nf);
            let p2 = (r / self.rs).powf(-nf);
            let (sn, cn) = (nf * th).sin_cos();
            s += (fld.c[n] * p1 + fld.d[n] * p2) * cn + (fld.e[n] * p1 + fld.f[n] * p2) * sn;
        }
        s
    }
}

fn particular_amp(n: usize, q_src: f64) -> f64 {
    if n == 1 { -MU0 * q_src / 2.0 } else { MU0 * q_src / ((n * n) as f64 - 1.0) }
}

impl Model {
    /// Potentiel vecteur A_z (r, θ) dans n'importe quelle région calculée ;
    /// `None` dans le fer (dents) ou hors domaine.
    pub fn a_at(&self, fld: &Field, r: f64, th: f64) -> Option<f64> {
        let nh = self.res.nh;
        if r >= self.rs && r <= self.rm {
            return Some(self.airgap_a(fld, r, th));
        }
        if r > self.rm && r <= self.rr {
            let mut s = 0.0;
            for n in 1..=nh {
                let nf = n as f64;
                let p1 = (r / self.rr).powf(nf);
                let p2 = (r / self.rm).powf(-nf);
                let pr = if n == 1 { r * r.ln() } else { r };
                let (sn, cn) = (nf * th).sin_cos();
                s += (fld.g[n] * p1 + fld.h[n] * p2 + fld.pc[n] * pr) * cn
                    + (fld.gs[n] * p1 + fld.hs[n] * p2 + fld.ps[n] * pr) * sn;
            }
            return Some(s);
        }
        if r >= self.rsb && r < self.rs {
            let tw = th.rem_euclid(2.0 * PI);
            for i in 0..self.q {
                let u = (tw - self.slot_start[i]).rem_euclid(2.0 * PI);
                if u <= self.beta {
                    let nk = self.res.nk;
                    let mut s = fld.a[i * (nk + 1)];
                    for k in 1..=nk {
                        let e = k as f64 * PI / self.beta;
                        let gk = (r / self.rs).powf(e) + (self.rsb / self.rs).powf(e) * (self.rsb / r).powf(e);
                        let g0 = 1.0 + (self.rsb / self.rs).powf(2.0 * e);
                        s += fld.a[i * (nk + 1) + k] * gk / g0 * (e * u).cos();
                    }
                    return Some(s);
                }
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

    fn small() -> Resolution {
        Resolution { nh: 200, nk: 12 }
    }

    /// Le couple calculé par le tenseur de Maxwell ne doit pas dépendre du
    /// rayon d'intégration dans l'entrefer (contrôle de la solution).
    #[test]
    fn couple_independant_du_rayon() {
        let s = MotorSpec::default();
        let m = Model::new(&s, small()).unwrap();
        let f = m.solve(1.0f64.to_radians());
        let (_, rs, rm, _) = m.radii();
        let t1 = m.torque_at(&f, rs + 0.2 * (rm - rs));
        let t2 = m.torque_at(&f, rs + 0.8 * (rm - rs));
        assert!((t1 - t2).abs() < 1e-9 * t1.abs().max(1e-6), "{t1} vs {t2}");
    }

    /// Continuité de A entre entrefer et encoche sur l'ouverture.
    #[test]
    fn continuite_a_sur_ouverture() {
        let s = MotorSpec::irregular_example();
        let m = Model::new(&s, small()).unwrap();
        let f = m.solve(0.3);
        let (_, rs, _, _) = m.radii();
        let nk = m.res.nk;
        let mut err = 0.0f64;
        let mut amp = 0.0f64;
        for i in [0, 5] {
            for t in 1..10 {
                let u = m.beta * t as f64 / 10.0;
                let th = m.slot_start[i] + u;
                let ag = m.airgap_a(&f, rs, th);
                let mut sl = f.a[i * (nk + 1)];
                for k in 1..=nk {
                    sl += f.a[i * (nk + 1) + k] * (k as f64 * PI / m.beta * u).cos();
                }
                err = err.max((ag - sl).abs());
                amp = amp.max(ag.abs());
            }
        }
        assert!(err < 2e-2 * amp, "err {err} amp {amp}");
    }

    /// Rotor régulier : couple de détente périodique de 360°/ppcm(Q,2p)
    /// et de valeur moyenne nulle.
    #[test]
    fn cogging_periodique_b0() {
        let s = MotorSpec::default();
        let m = Model::new(&s, small()).unwrap();
        let p = s.cogging_period_regular_deg().to_radians();
        let t0 = m.torque(&m.solve(0.37 * p));
        let t1 = m.torque(&m.solve(1.37 * p));
        assert!((t0 - t1).abs() < 1e-6 * t0.abs().max(1e-6), "{t0} vs {t1}");
    }

    /// Rotor régulier symétrique : force radiale résultante nulle.
    #[test]
    fn force_nulle_b0() {
        let s = MotorSpec::default();
        let m = Model::new(&s, small()).unwrap();
        let (fx, fy) = m.force(&m.solve(0.1));
        assert!(fx.hypot(fy) < 1e-6, "{fx} {fy}");
    }
}
