//! Modèle de MACHINE complet, construit sur les tronçons de `section.rs` :
//! - flux radial à rotor extérieur ou intérieur : un tronçon polaire ;
//! - flux axial : N tranches planes (technique multi-tranches de SIMUMSAP),
//!   chacune au rayon moyen de sa tranche, avec l'arc des aimants et
//!   l'ouverture angulaire des encoches propres à ce rayon ; 1 ou 2 entrefers.
//!
//! Le modèle gère les encoches semi-fermées (becs de dents), un bobinage
//! général (liste de bobines) et les courants de phase (fonctionnement en
//! charge, inductances).

use crate::geometry::{CoilDef, MotorSpec, Topology};
use crate::magnets::{self, MagFourier};
use crate::section::{Coord, Harmonics, Section, SectionField, SectionGeom};
use std::f64::consts::PI;

/// Plafond du nombre d'harmoniques adapté automatiquement.
pub const MAX_HARMONICS: usize = 2400;

/// Résolution harmonique.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Resolution {
    /// Harmoniques d'entrefer.
    pub nh: usize,
    /// Harmoniques par encoche (et par isthme).
    pub nk: usize,
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution { nh: 400, nk: 24 }
    }
}

/// Machine modélisée.
pub struct Model {
    /// Spécification.
    pub spec: MotorSpec,
    /// Résolution.
    pub res: Resolution,
    /// Nombre d'encoches.
    pub q: usize,
    /// Ouverture angulaire des encoches (tronçon de référence) [rad].
    pub beta: f64,
    /// Bord gauche de l'ouverture de chaque encoche (tronçon de référence).
    pub slot_start: Vec<f64>,
    /// Bobinage effectif.
    pub coils: Vec<CoilDef>,
    secs: Vec<Section>,
    mags: Vec<MagFourier>,
    /// facteur multiplicatif (nombre d'entrefers identiques en axial)
    mult: f64,
    ref_idx: usize,
}

/// Solution de la machine pour une position et des courants.
pub struct Field {
    /// Position du rotor [rad].
    pub theta_r: f64,
    /// Solutions par tronçon.
    pub secs: Vec<SectionField>,
}

/// Bobinage effectif : celui de la spécification, sinon bobinage concentré
/// sur dents déduit de l'étoile des encoches.
pub fn effective_coils(spec: &MotorSpec) -> Vec<CoilDef> {
    if !spec.coils.is_empty() {
        return spec.coils.clone();
    }
    let q = spec.n_slots;
    crate::winding::layout(spec)
        .into_iter()
        .map(|c| {
            // bobine autour de la dent j : moitié droite de l'encoche j−1, moitié gauche de l'encoche j
            let (a, b) = ((c.tooth + q - 1) % q, c.tooth);
            let (go, ret, gs, rs) = if c.sign > 0.0 { (a, b, 1, 0) } else { (b, a, 0, 1) };
            CoilDef { phase: c.phase, go, ret, turns: spec.turns_per_coil, go_side: gs, ret_side: rs }
        })
        .collect()
}

impl Model {
    /// Construit la machine (factorisation des systèmes de chaque tronçon).
    pub fn new(spec: &MotorSpec, res: Resolution) -> Result<Model, String> {
        let q = spec.n_slots;
        if q < 2 || spec.n_poles < 2 {
            return Err("nombre d'encoches ou de pôles insuffisant".into());
        }
        let h = Harmonics { nh: res.nh, ks: res.nk, ko: res.nk };
        // Nombre d'harmoniques adapté à la géométrie : au moins deux fois le
        // rapport circonférence / ouverture d'encoche, sinon le couple de
        // détente n'est pas convergé (grandes machines axiales à ouvertures
        // étroites : erreur d'un facteur 2,5 avec 400 harmoniques). Les
        // petites machines radiales gardent la valeur choisie.
        let adapt = |r: f64, open: f64| -> Harmonics {
            let need = (2.0 * 2.0 * PI * r / open.max(1e-6)).ceil() as usize;
            Harmonics { nh: res.nh.max(need).min(MAX_HARMONICS), ..h }
        };
        let centers: Vec<f64> = (0..q).map(|i| (i as f64 + 0.5) * 2.0 * PI / q as f64).collect();
        let open = spec.slot_open * 1e-3;
        let body = (spec.slot_w.max(spec.slot_open)) * 1e-3;
        let tip = spec.tip_h * 1e-3;
        let mut secs = Vec::new();
        let mut mags = Vec::new();
        let mut mult = 1.0;
        let dir = spec.rotor_dir();
        match spec.topology {
            Topology::RadialOuter | Topology::RadialInner => {
                let ys = spec.r_bore();
                let (ym, yr, ysb) = (spec.r_magnet_in(), spec.r_magnet_out(), spec.r_slot_bottom());
                if spec.slot_depth_m() <= tip + 1e-6 {
                    return Err("profondeur d'encoche inférieure à l'épaisseur des becs".into());
                }
                let yib = if tip > 0.0 { ys - dir * tip } else { ys - dir * 1e-7 };
                let beta_o = open / ys;
                let beta_s = if tip > 0.0 { body / yib } else { beta_o };
                let g = SectionGeom {
                    coord: Coord::Polar,
                    y_sb: ysb,
                    y_ib: yib,
                    y_s: ys,
                    y_m: ym,
                    y_r: yr,
                    slot_center: centers.clone(),
                    beta_s: beta_s.max(beta_o),
                    beta_o,
                    mur: spec.mur_magnet,
                    depth: spec.length(),
                };
                let hh = adapt(ys, open);
                secs.push(Section::new(g, hh)?);
                mags.push(magnets::fourier(&magnets::pieces(spec), spec.br_magnet, hh.nh));
            }
            Topology::Axial => {
                let ax = &spec.axial;
                if ax.r_out <= ax.r_in {
                    return Err("rayon extérieur ≤ rayon intérieur".into());
                }
                let n = ax.slices.max(1);
                let dr = (ax.r_out - ax.r_in) * 1e-3 / n as f64;
                mult = ax.gaps.clamp(1, 2) as f64;
                // tranches indépendantes : construites (et factorisées) en parallèle
                let built: Vec<Result<(Section, MagFourier), String>> = crate::par::map(n, |k| {
                    let r0 = ax.r_in * 1e-3 + (k as f64 + 0.5) * dr;
                    // cotes planes placées autour de r0 (uniquement pour l'affichage)
                    let ys = r0;
                    let ym = ys + spec.airgap * 1e-3;
                    let yr = ym + spec.magnet_th * 1e-3;
                    let ysb = ys - spec.slot_depth_m();
                    let yib = if tip > 0.0 { ys - tip } else { ys - 1e-7 };
                    let beta_o = open / r0;
                    let beta_s = if tip > 0.0 { body / r0 } else { beta_o };
                    let g = SectionGeom {
                        coord: Coord::Planar { r0 },
                        y_sb: ysb,
                        y_ib: yib,
                        y_s: ys,
                        y_m: ym,
                        y_r: yr,
                        slot_center: centers.clone(),
                        beta_s: beta_s.max(beta_o),
                        beta_o,
                        mur: spec.mur_magnet,
                        depth: dr,
                    };
                    let hh = adapt(r0 + dr / 2.0, open);
                    let sec = Section::new(g, hh)?;
                    // arc des aimants interpolé entre rayon intérieur et extérieur
                    let t = (k as f64 + 0.5) / n as f64;
                    let mut sk = spec.clone();
                    sk.magnet_arc_ratio = spec.magnet_arc_ratio + t * (ax.arc_ratio_out - spec.magnet_arc_ratio);
                    Ok((sec, magnets::fourier(&magnets::pieces(&sk), spec.br_magnet, hh.nh)))
                });
                for b in built {
                    let (sec, mf) = b?;
                    secs.push(sec);
                    mags.push(mf);
                }
            }
        }
        let ref_idx = secs.len() / 2;
        let beta = secs[ref_idx].g.beta_o;
        Ok(Model {
            spec: spec.clone(),
            res,
            q,
            beta,
            slot_start: centers.iter().map(|c| c - beta / 2.0).collect(),
            coils: effective_coils(spec),
            secs,
            mags,
            mult,
            ref_idx,
        })
    }

    /// Nombre de tronçons (tranches en axial).
    pub fn n_sections(&self) -> usize {
        self.secs.len()
    }

    /// Tous les tronçons (une tranche par rayon en flux axial).
    pub fn sections(&self) -> &[Section] {
        &self.secs
    }

    /// Indice du tronçon de référence (tranche du milieu en flux axial).
    pub fn reference_index(&self) -> usize {
        self.ref_idx
    }

    /// Flux axial : rayon moyen et largeur radiale de la tranche `i` [m]
    /// (rayon d'alésage et longueur active pour une machine radiale).
    pub fn slice_radius(&self, i: usize) -> (f64, f64) {
        let s = &self.secs[i];
        (s.rho(s.y_mid_gap()), s.g.depth)
    }

    /// Flux axial : rapport d'arc des aimants de la tranche `i` (interpolé
    /// entre le rayon intérieur et le rayon extérieur).
    pub fn slice_arc_ratio(&self, i: usize) -> f64 {
        let ax = &self.spec.axial;
        if self.spec.topology != Topology::Axial {
            return self.spec.magnet_arc_ratio;
        }
        let t = (i as f64 + 0.5) / self.secs.len().max(1) as f64;
        self.spec.magnet_arc_ratio + t * (ax.arc_ratio_out - self.spec.magnet_arc_ratio)
    }

    /// Nombre d'harmoniques effectivement utilisé (le plus grand des tronçons).
    pub fn harmonics_used(&self) -> usize {
        self.secs.iter().map(|s| s.h.nh).max().unwrap_or(self.res.nh)
    }

    /// Tronçon de référence (milieu).
    pub fn reference(&self) -> &Section {
        &self.secs[self.ref_idx]
    }

    /// Densités de courant par encoche pour des courants de phase donnés [A].
    pub fn slot_currents(&self, sec: &Section, i_phase: &[f64]) -> Vec<f64> {
        let paths = self.spec.load.parallel_paths.max(1.0);
        let mut at = vec![0.0; self.q];
        for c in &self.coils {
            let i = i_phase.get(c.phase).copied().unwrap_or(0.0) / paths * c.turns;
            at[c.go] += i;
            at[c.ret] -= i;
        }
        let a = sec.slot_area();
        at.iter().map(|x| x / a).collect()
    }

    /// Solution à vide (sans courant).
    pub fn solve(&self, theta_r: f64) -> Field {
        self.solve_i(theta_r, &[])
    }

    /// Solution avec courants de phase [A].
    pub fn solve_i(&self, theta_r: f64, i_phase: &[f64]) -> Field {
        let secs = self
            .secs
            .iter()
            .zip(&self.mags)
            .map(|(s, m)| {
                let j = if i_phase.iter().any(|x| *x != 0.0) { self.slot_currents(s, i_phase) } else { Vec::new() };
                s.solve(&m.rotated(theta_r), &j)
            })
            .collect();
        Field { theta_r, secs }
    }

    /// Solution sans aimants (réaction d'induit seule).
    pub fn solve_armature(&self, i_phase: &[f64]) -> Field {
        let zero = MagFourier { mr_c: vec![0.0; 1], mr_s: vec![0.0; 1], mt_c: vec![0.0; 1], mt_s: vec![0.0; 1] };
        let secs = self.secs.iter().map(|s| s.solve(&zero, &self.slot_currents(s, i_phase))).collect();
        Field { theta_r: 0.0, secs }
    }

    /// Couple sur le rotor [N·m] (somme des tranches × entrefers).
    pub fn torque(&self, fld: &Field) -> f64 {
        self.mult * self.secs.iter().zip(&fld.secs).map(|(s, f)| s.rotor_torque(f)).sum::<f64>()
    }

    /// Couple calculé à une cote donnée de l'entrefer (tronçon de référence, contrôle).
    pub fn torque_at(&self, fld: &Field, y: f64) -> f64 {
        let s = self.reference();
        let t = s.maxwell_torque(&fld.secs[self.ref_idx], y);
        self.mult * if s.stator_below() { -t } else { t } * self.secs.len() as f64
    }

    /// Force résultante sur le rotor [N]. Radial : (Fx, Fy) dans le plan.
    /// Axial : (force tangentielle totale, attraction axiale d'UN disque rotorique).
    pub fn force(&self, fld: &Field) -> (f64, f64) {
        let mut fx = 0.0;
        let mut fy = 0.0;
        for (s, f) in self.secs.iter().zip(&fld.secs) {
            let (a, b) = s.rotor_force(f);
            fx += a;
            fy += b;
        }
        if self.spec.topology != Topology::Axial {
            return (fx, fy);
        }
        // Flux axial : la force utile pour la vibration est la force
        // TRANSVERSE (dans le plan du disque) que crée une répartition
        // d'aimants non périodique : somme, sur le tour, des contraintes de
        // cisaillement Bn·Bt/μ0 projetées dans le plan. La contrainte normale
        // est axiale (attraction, voir `axial_attraction`).
        let (mut tx, mut ty) = (0.0, 0.0);
        for (s, f) in self.secs.iter().zip(&fld.secs) {
            let y = s.y_mid_gap();
            // produit de deux séries de rang ≤ nh : intégrale exacte (trapèzes) dès nt > 2·nh + 1
            let nt = 2 * s.h.nh + 8;
            let (bn, bt) = s.gap_b(f, y, nt);
            let dth = 2.0 * std::f64::consts::PI / nt as f64;
            let area = s.g.depth * s.rho(y) * dth;
            for k in 0..nt {
                let th = k as f64 * dth;
                let ft = bn[k] * bt[k] / crate::magnets::MU0 * area;
                tx -= ft * th.sin();
                ty += ft * th.cos();
            }
        }
        (tx * self.mult, ty * self.mult)
    }

    /// Flux axial : attraction axiale d'un disque rotor vers le stator [N]
    /// (zéro pour une machine radiale).
    pub fn axial_attraction(&self, fld: &Field) -> f64 {
        if self.spec.topology != Topology::Axial {
            return 0.0;
        }
        self.secs.iter().zip(&fld.secs).map(|(s, f)| s.rotor_force(f).1).sum()
    }

    /// Nombre de côtés actifs identiques (2 pour une machine axiale à deux
    /// entrefers, 1 sinon) : les flux de dents en font la somme.
    pub fn sides(&self) -> f64 {
        self.mult
    }

    /// Cote du milieu de l'entrefer (tronçon de référence).
    pub fn r_mid_gap(&self) -> f64 {
        self.reference().y_mid_gap()
    }

    /// Cotes caractéristiques du tronçon de référence : (fond d'encoche, alésage, aimants côté entrefer, aimants côté fer).
    pub fn radii(&self) -> (f64, f64, f64, f64) {
        let g = &self.reference().g;
        (g.y_sb, g.y_s, g.y_m, g.y_r)
    }

    /// Induction d'entrefer (tronçon de référence) sur `nt` points.
    pub fn airgap_b(&self, fld: &Field, y: f64, nt: usize) -> (Vec<f64>, Vec<f64>) {
        self.reference().gap_b(&fld.secs[self.ref_idx], y, nt)
    }

    /// Flux de la bobine autour de la dent j, par spire [Wb].
    pub fn tooth_flux(&self, fld: &Field, j: usize) -> f64 {
        let q = self.q;
        let (l, r) = ((j + q - 1) % q, j);
        self.mult * self.secs.iter().zip(&fld.secs).map(|(s, f)| s.g.depth * (s.slot_mean_a(f, l, 1) - s.slot_mean_a(f, r, 0))).sum::<f64>()
    }

    /// Flux totalisés des trois phases [Wb] (spires et voies parallèles comprises).
    pub fn phase_flux(&self, fld: &Field) -> [f64; 3] {
        let paths = self.spec.load.parallel_paths.max(1.0);
        let mut out = [0.0; 3];
        for c in &self.coils {
            let mut lam = 0.0;
            for (s, f) in self.secs.iter().zip(&fld.secs) {
                lam += s.g.depth * (s.slot_mean_a(f, c.go, c.go_side as usize) - s.slot_mean_a(f, c.ret, c.ret_side as usize));
            }
            if c.phase < 3 {
                out[c.phase] += self.mult * c.turns * lam / paths;
            }
        }
        out
    }

    /// A dans l'entrefer (tronçon de référence).
    pub fn airgap_a(&self, fld: &Field, y: f64, th: f64) -> f64 {
        self.reference().gap_a(&fld.secs[self.ref_idx], y, th)
    }

    /// A en un point (tronçon de référence) ; None dans le fer.
    pub fn a_at(&self, fld: &Field, y: f64, th: f64) -> Option<f64> {
        self.reference().a_at(&fld.secs[self.ref_idx], y, th)
    }

    /// Surface d'une encoche du tronçon de référence [m²].
    pub fn slot_area(&self) -> f64 {
        self.reference().slot_area()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::MotorSpec;

    fn small() -> Resolution {
        Resolution { nh: 200, nk: 12 }
    }

    #[test]
    fn identique_a_l_ancien_modele_par_defaut() {
        for v in ["regulier", "irregulier"] {
            let s = MotorSpec::example(v);
            let old = crate::legacy::Model::new(&s, crate::legacy::Resolution { nh: 200, nk: 12 }).unwrap();
            let new = Model::new(&s, small()).unwrap();
            for th in [0.2f64, 1.7] {
                let (a, b) = (old.torque(&old.solve(th.to_radians())), new.torque(&new.solve(th.to_radians())));
                assert!((a - b).abs() < 1e-4 * a.abs().max(1e-3), "{v}: {a} vs {b}");
                let (fa, fb) = (old.tooth_flux(&old.solve(0.3), 2), new.tooth_flux(&new.solve(0.3), 2));
                assert!((fa - fb).abs() < 1e-3 * fa.abs(), "flux {fa} vs {fb}");
            }
        }
    }

    #[test]
    fn inductance_positive_et_symetrique() {
        let s = MotorSpec::default();
        let m = Model::new(&s, small()).unwrap();
        let f = m.solve_armature(&[1.0, 0.0, 0.0]);
        let lam = m.phase_flux(&f);
        assert!(lam[0] > 0.0, "L propre {}", lam[0]);
        let g = m.solve_armature(&[0.0, 1.0, 0.0]);
        let lb = m.phase_flux(&g);
        // réciprocité : M_AB = M_BA
        assert!((lam[1] - lb[0]).abs() < 1e-3 * lam[0].abs(), "{} vs {}", lam[1], lb[0]);
    }

    #[test]
    fn rotor_interieur_et_axial_calculent() {
        let mut s = MotorSpec::default();
        s.topology = Topology::RadialInner;
        s.stator_or = 9.8;
        s.stator_ir = 16.0;
        s.yoke_th = 1.5;
        s.tip_h = 0.5;
        s.slot_w = 3.5;
        let m = Model::new(&s, small()).unwrap();
        assert!(m.torque(&m.solve(0.02)).is_finite());
        let mut a = MotorSpec::default();
        a.topology = Topology::Axial;
        a.n_slots = 12;
        a.slot_depth = 12.0;
        a.tip_h = 1.0;
        a.slot_w = 8.0;
        a.slot_open = 3.0;
        a.airgap = 1.5;
        a.magnet_th = 6.5;
        let m = Model::new(&a, small()).unwrap();
        let t = m.torque(&m.solve(0.02));
        assert!(t.is_finite());
        let fz = m.axial_attraction(&m.solve(0.0));
        assert!(fz.abs() > 1.0, "attraction axiale {fz}");
        // force transverse : nulle par symétrie pour un rotor régulier,
        // non nulle pour une répartition apériodique ; toujours petite devant l'attraction
        let (fx, fy) = m.force(&m.solve(0.0));
        let mut v = a.clone();
        v.custom_angles_deg = MotorSpec::irregular_example().custom_angles_deg.filter(|c| c.len() == v.n_poles).or_else(|| {
            let p = v.pole_pitch_deg();
            Some((0..v.n_poles).map(|k| k as f64 * p + 2.0 * (4.0 * std::f64::consts::PI * k as f64 / v.n_poles as f64).sin()).collect())
        });
        let mv = Model::new(&v, small()).unwrap();
        let (vx, vy) = mv.force(&mv.solve(0.0));
        assert!(fx.hypot(fy) < 1e-6 * fz.abs().max(1.0), "régulier : {fx} {fy}");
        assert!(vx.hypot(vy) > 1e3 * fx.hypot(fy).max(1e-12) && vx.hypot(vy) < 0.2 * fz.abs(), "irrégulier : {} / {fz}", vx.hypot(vy));
    }
}
