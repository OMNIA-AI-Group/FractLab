//! Solveur NUMÉRIQUE indépendant (volumes finis sur grille polaire) servant
//! à valider le modèle analytique en sous-domaines.
//!
//! Même géométrie idéalisée que `subdomain.rs` (encoches droites de largeur
//! `slot_open`, fer rotor et culasse stator en frontière Hθ = 0) mais
//! méthode différente : les dents sont du FER DE PERMÉABILITÉ FINIE
//! (μr = 1e5) maillé, l'aimantation est moyennée par cellule (rotation
//! continue du rotor), et le couple est intégré par le tenseur de Maxwell
//! moyenné sur toutes les couches de l'entrefer (méthode d'Arkkio).
//! Résolution : gradient conjugué préconditionné par une factorisation de
//! Cholesky en bande du système sans le raccord périodique.

use crate::geometry::MotorSpec;
use crate::magnets::{self, Piece, PieceKind, MU0};
use std::f64::consts::PI;

/// Grille polaire du solveur de contrôle.
pub struct FdGrid {
    /// Nombre de cellules angulaires.
    pub nt: usize,
    /// Rayons des centres de cellules [m].
    pub rc: Vec<f64>,    // centres radiaux
    /// Rayons des faces radiales [m] (nr + 1 valeurs).
    pub rf: Vec<f64>,    // faces radiales (nr + 1)
    /// Zone de chaque couche : 0 encoches/dents, 1 entrefer, 2 aimants.
    pub zone: Vec<u8>,   // par couche : 0 encoche/dent, 1 entrefer, 2 aimant
    /// Pas angulaire [rad].
    pub dth: f64,
    /// Géométrie plane (tranche de machine axiale, circonférence développée) :
    /// rayon moyen r0 qui remplace le rayon dans les longueurs d'arc, les
    /// aires et le bras de levier ; `None` = géométrie polaire.
    pub planar_r0: Option<f64>,
}

impl FdGrid {
    /// Rayon métrique : r en polaire, r0 en géométrie plane.
    fn arc(&self, r: f64) -> f64 {
        self.planar_r0.unwrap_or(r)
    }
    /// Aire d'une cellule de la couche i [m²].
    fn area(&self, i: usize) -> f64 {
        match self.planar_r0 {
            Some(r0) => (self.rf[i + 1] - self.rf[i]) * r0 * self.dth,
            None => 0.5 * (self.rf[i + 1].powi(2) - self.rf[i].powi(2)) * self.dth,
        }
    }
}

/// Solveur volumes finis prêt à l'emploi (préconditionneur factorisé).
pub struct FdModel {
    /// Spécification, avec l'ouverture d'encoche arrondie à la grille.
    pub spec: MotorSpec,
    /// Grille de calcul.
    pub g: FdGrid,
    pieces: Vec<Piece>,
    iron: Vec<bool>, // (j*nr + i) : cellule de fer
    chol: BandChol,
    /// Perméabilité relative du fer maillé.
    pub mu_iron: f64,
    mur: f64,
    br: f64,
    length: f64,
    /// stator du côté des rayons faibles (rotor extérieur) ?
    stator_below: bool,
    /// densité de courant par cellule [A/m²]
    cell_j: Vec<f64>,
    /// encoche de chaque cellule du corps d'encoche (−1 ailleurs)
    slot_of: Vec<i32>,
}

fn graded(a: f64, b: f64, n: usize, ratio: f64, fine_at_b: bool) -> Vec<f64> {
    // n cellules de a à b, progression géométrique (taille finale/initiale = ratio)
    let q = ratio.powf(1.0 / (n.max(2) - 1) as f64);
    let mut w: Vec<f64> = (0..n).map(|k| q.powi(k as i32)).collect();
    if fine_at_b {
        w.reverse();
    }
    let s: f64 = w.iter().sum();
    let mut f = vec![a];
    let mut x = a;
    for wi in w {
        x += (b - a) * wi / s;
        f.push(x);
    }
    *f.last_mut().unwrap() = b;
    f
}

impl FdModel {
    /// Solveur pour une géométrie générale de tronçon polaire (rotor intérieur
    /// ou extérieur, encoches semi-fermées, courants). Les bords angulaires
    /// des encoches et ouvertures sont arrondis à la grille : la géométrie
    /// effectivement maillée est renvoyée (à réutiliser pour le modèle
    /// analytique comparé).
    pub fn general(
        geom: &crate::section::SectionGeom,
        pieces: Vec<Piece>,
        br: f64,
        nt: usize,
        scale: usize,
        j_slots: &[f64],
    ) -> (FdModel, crate::section::SectionGeom) {
        let mut ge = geom.clone();
        let dth = 2.0 * PI / nt as f64;
        let snap = |x: f64| (x / dth).round() * dth;
        // largeurs : nombre pair de cellules ; centres sur des faces de cellules
        ge.beta_s = 2.0 * snap(ge.beta_s / 2.0).max(dth);
        ge.beta_o = 2.0 * snap(ge.beta_o / 2.0).max(dth);
        ge.slot_center = ge.slot_center.iter().map(|&c| snap(c)).collect();
        let s = scale.max(1);
        let below = ge.y_s < ge.y_m;
        // couches, dans l'ordre des y croissants
        let mut layers: Vec<(f64, f64, usize, u8, bool)> = vec![
            (ge.y_sb, ge.y_ib, 12 * s, 0, below),
            (ge.y_ib, ge.y_s, 6 * s, 3, below),
            (ge.y_s, ge.y_m, 6 * s, 1, false),
            (ge.y_m, ge.y_r, 10 * s, 2, !below),
        ];
        for l in layers.iter_mut() {
            if l.0 > l.1 {
                std::mem::swap(&mut l.0, &mut l.1);
            }
        }
        layers.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut rf = vec![layers[0].0];
        let mut zone_of_layer = Vec::new();
        for &(a, b, n, z, fine_b) in &layers {
            if b - a < 1e-12 {
                continue;
            }
            let ratio = if z == 1 { 1.0 } else { 4.0 };
            let f = graded(a, b, n, ratio, fine_b);
            rf.extend_from_slice(&f[1..]);
            zone_of_layer.extend(std::iter::repeat_n(z, n));
        }
        let nr = rf.len() - 1;
        let rc: Vec<f64> = (0..nr).map(|i| 0.5 * (rf[i] + rf[i + 1])).collect();
        // zones : 0 corps d'encoche, 3 isthme → « stator » ; 1 entrefer ; 2 aimant
        let zone: Vec<u8> = zone_of_layer.iter().map(|&z| if z == 3 { 0 } else { z }).collect();
        let mut iron = vec![false; nt * nr];
        let mut cell_j = vec![0.0; nt * nr];
        let mut slot_of = vec![-1i32; nt * nr];
        for j in 0..nt {
            let th = (j as f64 + 0.5) * dth;
            for i in 0..nr {
                let z = zone_of_layer[i];
                if z != 0 && z != 3 {
                    continue;
                }
                let w = if z == 0 { ge.beta_s } else { ge.beta_o };
                let mut inside = None;
                for (k, &c) in ge.slot_center.iter().enumerate() {
                    let d = ((th - c + PI).rem_euclid(2.0 * PI)) - PI;
                    if d.abs() < w / 2.0 {
                        inside = Some(k);
                    }
                }
                match inside {
                    None => iron[j * nr + i] = true,
                    Some(k) if z == 0 => {
                        cell_j[j * nr + i] = j_slots.get(k).copied().unwrap_or(0.0);
                        slot_of[j * nr + i] = k as i32;
                    }
                    _ => {}
                }
            }
        }
        let planar_r0 = match ge.coord {
            crate::section::Coord::Planar { r0 } => Some(r0),
            crate::section::Coord::Polar => None,
        };
        let g = FdGrid { nt, rc, rf, zone, dth, planar_r0 };
        let mut m = FdModel {
            spec: MotorSpec::default(),
            g,
            pieces,
            iron,
            chol: BandChol { n: 0, bw: 0, l: vec![] },
            mu_iron: 1e5,
            mur: ge.mur,
            br,
            length: ge.depth,
            stator_below: below,
            cell_j,
            slot_of,
        };
        let (diag, off_r, off_t) = m.coeffs();
        m.chol = BandChol::factor_nonperiodic(m.g.nt, m.g.rc.len(), &diag, &off_r, &off_t);
        (m, ge)
    }

    /// `nt` doit être multiple de 2Q ; l'ouverture d'encoche est arrondie à
    /// un nombre pair de cellules (renvoie la spec ajustée).
    pub fn new(spec_in: &MotorSpec, nt: usize, scale: usize) -> FdModel {
        let mut spec = spec_in.clone();
        let q = spec.n_slots;
        assert!(nt.is_multiple_of(2 * q));
        let dth = 2.0 * PI / nt as f64;
        let rs = spec.r_bore();
        let half_cells = ((spec.slot_open * 1e-3 / rs) / 2.0 / dth).round().max(1.0) as usize;
        spec.slot_open = 2.0 * half_cells as f64 * dth * rs * 1e3;
        let (rsb, rm, rr) = (spec.r_slot_bottom(), spec.r_magnet_in(), spec.r_magnet_out());
        let s = scale.max(1);
        let mut rf = graded(rsb, rs, 16 * s, 8.0, true);
        let gap = graded(rs, rm, 6 * s, 1.0, false);
        rf.extend_from_slice(&gap[1..]);
        let mag = graded(rm, rr, 12 * s, 4.0, false);
        rf.extend_from_slice(&mag[1..]);
        let nr = rf.len() - 1;
        let rc: Vec<f64> = (0..nr).map(|i| 0.5 * (rf[i] + rf[i + 1])).collect();
        let zone: Vec<u8> = rc.iter().map(|&r| if r < rs { 0 } else if r < rm { 1 } else { 2 }).collect();
        let mut iron = vec![false; nt * nr];
        let per_slot = nt / q;
        for j in 0..nt {
            // encoche i centrée sur la face (i + 1/2)·per_slot (comme subdomain.rs)
            let jr = (j + nt - per_slot / 2) % per_slot;
            let in_slot = jr < half_cells || jr >= per_slot - half_cells;
            for i in 0..nr {
                if zone[i] == 0 && !in_slot {
                    iron[j * nr + i] = true;
                }
            }
        }
        let g = FdGrid { nt, rc, rf, zone, dth, planar_r0: None };
        let mu_iron = 1e5;
        let pieces = magnets::pieces(&spec);
        let nr0 = g.rc.len();
        let mut m = FdModel {
            mur: spec.mur_magnet,
            br: spec.br_magnet,
            length: spec.length(),
            stator_below: true,
            cell_j: vec![0.0; nt * nr0],
            slot_of: vec![-1; nt * nr0],
            spec,
            g,
            pieces,
            iron,
            chol: BandChol { n: 0, bw: 0, l: vec![] },
            mu_iron,
        };
        let (diag, off_r, off_t) = m.coeffs();
        m.chol = BandChol::factor_nonperiodic(m.g.nt, m.g.rc.len(), &diag, &off_r, &off_t);
        m
    }

    fn nu(&self, j: usize, i: usize) -> f64 {
        let nr = self.g.rc.len();
        if self.iron[j * nr + i] {
            1.0 / (MU0 * self.mu_iron)
        } else if self.g.zone[i] == 2 {
            1.0 / (MU0 * self.mur)
        } else {
            1.0 / MU0
        }
    }

    /// Coefficients : diagonale, couplage radial (i,i+1), couplage angulaire (j,j+1 périodique).
    fn coeffs(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let nt = self.g.nt;
        let nr = self.g.rc.len();
        let mut diag = vec![0.0; nt * nr];
        let mut off_r = vec![0.0; nt * nr];
        let mut off_t = vec![0.0; nt * nr];
        for j in 0..nt {
            for i in 0..nr {
                let p = j * nr + i;
                if i + 1 < nr {
                    let (dp, dq) = (self.g.rf[i + 1] - self.g.rc[i], self.g.rc[i + 1] - self.g.rf[i + 1]);
                    let nuf = (dp + dq) / (dp / self.nu(j, i) + dq / self.nu(j, i + 1));
                    let c = self.g.arc(self.g.rf[i + 1]) * self.g.dth * nuf / (dp + dq);
                    off_r[p] = -c;
                    diag[p] += c;
                    diag[p + 1] += c;
                }
                let jn = (j + 1) % nt;
                let nuf = 2.0 / (1.0 / self.nu(j, i) + 1.0 / self.nu(jn, i));
                let dr = self.g.rf[i + 1] - self.g.rf[i];
                let c = nuf * dr / (self.g.arc(self.g.rc[i]) * self.g.dth);
                off_t[p] = -c;
                diag[p] += c;
                diag[jn * nr + i] += c;
            }
        }
        (diag, off_r, off_t)
    }

    /// Aimantation moyenne (μ0·Mr, μ0·Mθ) [T] de chaque colonne angulaire.
    fn cell_mag(&self, theta_r: f64) -> (Vec<f64>, Vec<f64>) {
        let nt = self.g.nt;
        let dth = self.g.dth;
        let br = self.br;
        let mut mr = vec![0.0; nt];
        let mut mt = vec![0.0; nt];
        for p in &self.pieces {
            let (t1, t2) = (p.t1 + theta_r, p.t2 + theta_r);
            let j0 = (t1 / dth).floor() as i64;
            let j1 = (t2 / dth).ceil() as i64;
            for jj in j0..j1 {
                let a = (jj as f64 * dth).max(t1);
                let b = ((jj + 1) as f64 * dth).min(t2);
                if b <= a {
                    continue;
                }
                let j = jj.rem_euclid(nt as i64) as usize;
                match p.kind {
                    PieceKind::Radial { s } => mr[j] += s * br * (b - a) / dth,
                    PieceKind::Fixed { psi } => {
                        let psi = psi + theta_r;
                        mr[j] += br * ((b - psi).sin() - (a - psi).sin()) / dth;
                        mt[j] += br * ((psi - b).cos() - (psi - a).cos()) / dth;
                    }
                }
            }
        }
        (mr, mt)
    }

    /// Potentiel vecteur A [Wb/m] par cellule pour la position `theta_r` [rad].
    pub fn solve(&self, theta_r: f64) -> Vec<f64> {
        let nt = self.g.nt;
        let nr = self.g.rc.len();
        let (mr, mt) = self.cell_mag(theta_r);
        // μ0 M par cellule (nul hors aimants) ; M'_f = ν_f (μ0Mp dp + μ0Mq dq)/(dp+dq)
        let m0 = |j: usize, i: usize, comp: &Vec<f64>| if self.g.zone[i] == 2 { comp[j] } else { 0.0 };
        let mut b = vec![0.0; nt * nr];
        for j in 0..nt {
            for i in 0..nr {
                let p = j * nr + i;
                if i + 1 < nr {
                    let (dp, dq) = (self.g.rf[i + 1] - self.g.rc[i], self.g.rc[i + 1] - self.g.rf[i + 1]);
                    let (np, nq) = (self.nu(j, i), self.nu(j, i + 1));
                    let nuf = (dp + dq) / (dp / np + dq / nq);
                    let mf = nuf * (m0(j, i, &mt) * dp + m0(j, i + 1, &mt) * dq) / (dp + dq);
                    let sf = self.g.arc(self.g.rf[i + 1]) * mf * self.g.dth;
                    b[p] += sf;
                    b[p + 1] -= sf;
                }
                let jn = (j + 1) % nt;
                let (np, nq) = (self.nu(j, i), self.nu(jn, i));
                let nuf = 2.0 / (1.0 / np + 1.0 / nq);
                let mf = nuf * 0.5 * (m0(j, i, &mr) + m0(jn, i, &mr));
                let dr = self.g.rf[i + 1] - self.g.rf[i];
                let sf = -mf * dr;
                b[p] += sf;
                b[jn * nr + i] -= sf;
            }
        }
        // courants d'encoche : ∫ J dS par cellule
        for j in 0..nt {
            for i in 0..nr {
                let jv = self.cell_j[j * nr + i];
                if jv != 0.0 {
                    b[j * nr + i] += jv * self.g.area(i);
                }
            }
        }
        let (diag, off_r, off_t) = self.coeffs();
        pcg(nt, nr, &diag, &off_r, &off_t, &b, &self.chol)
    }

    /// Potentiel moyen [Wb/m] sur le corps de l'encoche `k` (moyenne pondérée
    /// par l'aire des cellules), pour le calcul des flux de bobines.
    pub fn slot_mean_a(&self, a: &[f64], k: usize) -> f64 {
        let nr = self.g.rc.len();
        let (mut num, mut den) = (0.0, 0.0);
        for (p, &s) in self.slot_of.iter().enumerate() {
            if s == k as i32 {
                let w = self.g.area(p % nr);
                num += a[p] * w;
                den += w;
            }
        }
        if den > 0.0 { num / den } else { 0.0 }
    }

    /// Couple sur le rotor [N·m], moyenne sur les faces internes de l'entrefer.
    pub fn torque(&self, a: &[f64]) -> f64 {
        let nt = self.g.nt;
        let nr = self.g.rc.len();
        let l = self.length;
        let sgn = if self.stator_below { -1.0 } else { 1.0 };
        let mut acc = 0.0;
        let mut cnt = 0.0;
        for i in 0..nr - 1 {
            if !(self.g.zone[i] == 1 && self.g.zone[i + 1] == 1) {
                continue;
            }
            let r = self.g.arc(self.g.rf[i + 1]);
            let dr = self.g.rc[i + 1] - self.g.rc[i];
            let mut s = 0.0;
            for j in 0..nt {
                let jp = (j + 1) % nt;
                let jm = (j + nt - 1) % nt;
                let am = |jj: usize| 0.5 * (a[jj * nr + i] + a[jj * nr + i + 1]);
                let br = (am(jp) - am(jm)) / (2.0 * self.g.dth * r);
                let bt = -(a[j * nr + i + 1] - a[j * nr + i]) / dr;
                s += br * bt * self.g.dth;
            }
            acc += sgn * (l * r * r / MU0) * s;
            cnt += 1.0;
        }
        acc / cnt
    }
}

/// Cholesky en bande (ordre θ-majeur : index = j·nr + i, demi-largeur = nr).
pub struct BandChol {
    n: usize,
    bw: usize,
    l: Vec<f64>, // l[p*(bw+1) + k] = L[p][p-k]
}

impl BandChol {
    fn factor_nonperiodic(nt: usize, nr: usize, diag: &[f64], off_r: &[f64], off_t: &[f64]) -> BandChol {
        let n = nt * nr;
        let bw = nr;
        let w = bw + 1;
        let mut l = vec![0.0; n * w];
        // remplir A (partie basse) : A[p][p-1] = off_r[p-1] si même colonne ; A[p][p-nr] = off_t[p-nr]
        for p in 0..n {
            l[p * w] = diag[p];
            let i = p % nr;
            if i > 0 {
                l[p * w + 1] = off_r[p - 1];
            }
            if p >= nr {
                l[p * w + nr] = off_t[p - nr];
            }
        }
        for p in 0..n {
            let kmax = bw.min(p);
            // L[p][p-k] pour k = kmax..1
            for k in (1..=kmax).rev() {
                let c = p - k;
                let mut s = l[p * w + k];
                let lim = bw.min(c).min(bw - k);
                for t in 1..=lim {
                    // L[p][c-t] * L[c][c-t]
                    s -= l[p * w + k + t] * l[c * w + t];
                }
                l[p * w + k] = s / l[c * w];
            }
            let mut s = l[p * w];
            for k in 1..=kmax {
                s -= l[p * w + k] * l[p * w + k];
            }
            l[p * w] = s.sqrt();
        }
        BandChol { n, bw, l }
    }

    fn solve(&self, b: &[f64]) -> Vec<f64> {
        let (n, bw, w) = (self.n, self.bw, self.bw + 1);
        let mut y = b.to_vec();
        for p in 0..n {
            let mut s = y[p];
            for k in 1..=bw.min(p) {
                s -= self.l[p * w + k] * y[p - k];
            }
            y[p] = s / self.l[p * w];
        }
        for p in (0..n).rev() {
            let mut s = y[p];
            for k in 1..=bw.min(n - 1 - p) {
                s -= self.l[(p + k) * w + k] * y[p + k];
            }
            y[p] = s / self.l[p * w];
        }
        y
    }
}

fn matvec(nt: usize, nr: usize, diag: &[f64], off_r: &[f64], off_t: &[f64], x: &[f64]) -> Vec<f64> {
    let n = nt * nr;
    let mut y: Vec<f64> = (0..n).map(|p| diag[p] * x[p]).collect();
    for j in 0..nt {
        for i in 0..nr {
            let p = j * nr + i;
            if i + 1 < nr {
                y[p] += off_r[p] * x[p + 1];
                y[p + 1] += off_r[p] * x[p];
            }
            let q = ((j + 1) % nt) * nr + i;
            y[p] += off_t[p] * x[q];
            y[q] += off_t[p] * x[p];
        }
    }
    y
}

fn pcg(nt: usize, nr: usize, diag: &[f64], off_r: &[f64], off_t: &[f64], b: &[f64], m: &BandChol) -> Vec<f64> {
    // Le système est singulier à une constante près (Neumann partout) :
    // on projette b et les itérés sur l'orthogonal des constantes.
    let n = nt * nr;
    let mean = |v: &mut Vec<f64>| {
        let s = v.iter().sum::<f64>() / n as f64;
        v.iter_mut().for_each(|x| *x -= s);
    };
    let mut bb = b.to_vec();
    mean(&mut bb);
    let mut x = vec![0.0; n];
    let mut r = bb.clone();
    let mut z = m.solve(&r);
    mean(&mut z);
    let mut p = z.clone();
    let mut rz: f64 = r.iter().zip(&z).map(|(a, b)| a * b).sum();
    let bn = bb.iter().map(|v| v * v).sum::<f64>().sqrt();
    if bn == 0.0 {
        return x;
    }
    for _it in 0..500 {
        let ap = matvec(nt, nr, diag, off_r, off_t, &p);
        let pap: f64 = p.iter().zip(&ap).map(|(a, b)| a * b).sum();
        // convergence exacte (direction nulle) : arrêt, sinon 0/0 donnerait NaN
        if !(pap > 0.0) || !pap.is_finite() {
            break;
        }
        let al = rz / pap;
        for k in 0..n {
            x[k] += al * p[k];
            r[k] -= al * ap[k];
        }
        let rn = r.iter().map(|v| v * v).sum::<f64>().sqrt();
        if rn < 1e-12 * bn {
            break;
        }
        z = m.solve(&r);
        mean(&mut z);
        let rz2: f64 = r.iter().zip(&z).map(|(a, b)| a * b).sum();
        if rz2 == 0.0 || !rz2.is_finite() {
            break;
        }
        let be = rz2 / rz;
        rz = rz2;
        for k in 0..n {
            p[k] = z[k] + be * p[k];
        }
    }
    x
}
