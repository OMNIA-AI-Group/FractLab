//! Analyses : balayages en position, spectres et métriques du couple de détente.

use crate::geometry::{self, MotorSpec};
use crate::subdomain::{Field, Model};
use crate::winding::{self, Coil};
use serde::Serialize;
use std::f64::consts::PI;

/// Amplitudes |X_k| (k = 0..n/2) d'un signal échantillonné régulièrement,
/// normalisées en amplitude crête (DFT directe, n ≤ quelques milliers).
pub fn spectrum(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let m = x.iter().sum::<f64>() / n as f64;
    crate::par::map(n / 2 + 1, |k| {
            let (mut re, mut im) = (0.0, 0.0);
            for (t, v) in x.iter().enumerate() {
                let a = 2.0 * PI * (k * t) as f64 / n as f64;
                re += (v - m) * a.cos();
                im -= (v - m) * a.sin();
            }
            let s = if k == 0 || 2 * k == n { 1.0 } else { 2.0 };
            s * (re * re + im * im).sqrt() / n as f64
        })
}

#[derive(Clone, Debug, Serialize, Default, serde::Deserialize)]
/// Métriques du couple de détente.
pub struct CoggingMetrics {
    /// Nom de la variante.
    pub variant: String,
    /// Fenêtre de balayage [°].
    pub span_deg: f64,
    /// Couple de détente crête-à-crête [mN·m].
    pub cogging_pp_mnm: f64,
    /// Couple de détente efficace (RMS, valeur moyenne retirée) [mN·m].
    pub cogging_rms_mnm: f64,
    /// harmonique dominante (amplitude, mN·m) et son ordre par tour
    pub dominant_mnm: f64,
    /// Ordre de l'harmonique dominante, en raies par tour mécanique.
    pub dominant_order_per_rev: f64,
    /// pic tonal : dominante / plancher médian [dB] 
    pub tonal_peak_db: f64,
    /// Entropie de Shannon du spectre normalisé (étalement).
    pub spectral_entropy: f64,
    /// part de l'énergie spectrale portée par la seule dominante [%]
    /// (métrique robuste proposée : le pic tonal en dB dépend du plancher
    /// numérique quand le signal est sans bruit)
    pub dominant_share_pct: f64,
    /// Nombre de positions calculées.
    pub n_points: usize,
}

/// Courbe de couple de détente et son analyse.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CoggingCurve {
    /// Positions du rotor [°].
    pub angles_deg: Vec<f64>,
    /// Couple sur le rotor [N·m].
    pub torque_nm: Vec<f64>,
    /// Amplitudes du spectre [N·m], indice = rang sur la fenêtre.
    pub spectrum: Vec<f64>,
    /// Métriques associées.
    pub metrics: CoggingMetrics,
}

/// Balayage du couple de détente sur `span_deg` (période ou fenêtre commune).
pub fn cogging(model: &Model, span_deg: f64, n: usize) -> CoggingCurve {
    let angles: Vec<f64> = (0..n).map(|i| span_deg * i as f64 / n as f64).collect();
    let torque: Vec<f64> = crate::par::map(n, |i| model.torque(&model.solve(angles[i].to_radians())));
    let sp = spectrum(&torque);
    let (mn, mx) = torque.iter().fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    let mean = torque.iter().sum::<f64>() / n as f64;
    let rms = (torque.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
    let mags = &sp[1..];
    let (kdom, dom) = mags.iter().enumerate().fold((0, 0.0), |(ki, m), (k, &v)| if v > m { (k + 1, v) } else { (ki, m) });
    let mut sorted = mags.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let floor = sorted[sorted.len() / 2].max(1e-15);
    let tot: f64 = mags.iter().sum();
    let entropy = if tot > 0.0 {
        -mags.iter().map(|v| v / tot).filter(|p| *p > 0.0).map(|p| p * p.ln()).sum::<f64>()
    } else {
        0.0
    };
    CoggingCurve {
        metrics: CoggingMetrics {
            variant: model.spec.name.clone(),
            span_deg,
            cogging_pp_mnm: (mx - mn) * 1e3,
            cogging_rms_mnm: rms * 1e3,
            dominant_mnm: dom * 1e3,
            dominant_order_per_rev: kdom as f64 * 360.0 / span_deg,
            tonal_peak_db: 20.0 * (dom.max(1e-15) / floor).log10(),
            spectral_entropy: entropy,
            dominant_share_pct: {
                let e: f64 = mags.iter().map(|v| v * v).sum();
                if e > 0.0 { 100.0 * dom * dom / e } else { 0.0 }
            },
            n_points: n,
        },
        angles_deg: angles,
        torque_nm: torque,
        spectrum: sp,
    }
}

/// FEM à vide sur un tour mécanique.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EmfCurve {
    /// Positions du rotor [°].
    pub angles_deg: Vec<f64>,
    /// flux de phase [Wb] (spires comprises)
    pub flux: [Vec<f64>; 3],
    /// FEM de phase [V] à la vitesse de la spec
    pub emf: [Vec<f64>; 3],
    /// Bobinage utilisé.
    pub coils: Vec<Coil>,
    /// spectre de la FEM de la phase A par ordre mécanique (par tour)
    pub spectrum_a: Vec<f64>,
    /// FEM de phase crête maximale [V].
    pub emf_peak_v: f64,
    /// Taux de distorsion harmonique de la FEM de la phase A [%].
    pub thd_pct: f64,
    /// énergie hors fondamental hors multiples de p (interharmoniques) [%]
    pub interharm_pct: f64,
    /// constante de FEM ligne-ligne crête [V/krpm] et KV estimé [tr/min/V]
    pub ke_ll_v_per_krpm: f64,
    /// KV estimé [tr/min/V] (modèle 2D).
    pub kv_estime: f64,
    /// Écart relatif entre les crêtes des trois phases [%].
    pub unbalance_pct: f64,
}

/// FEM à vide sur un tour mécanique complet (n positions).
pub fn emf(model: &Model, n: usize) -> EmfCurve {
    let spec = &model.spec;
    let coils = winding::layout(spec);
    let angles: Vec<f64> = (0..n).map(|i| 360.0 * i as f64 / n as f64).collect();
    let mut flux = [vec![0.0; n], vec![0.0; n], vec![0.0; n]];
    let per_pos: Vec<[f64; 3]> = crate::par::map(n, |t| {
        let f = model.solve(angles[t].to_radians());
        let mut v = [0.0; 3];
        for c in &coils {
            v[c.phase] += c.sign * spec.turns_per_coil * model.tooth_flux(&f, c.tooth);
        }
        v
    });
    for (t, v) in per_pos.iter().enumerate() {
        for ph in 0..3 {
            flux[ph][t] = v[ph];
        }
    }
    let w = spec.rpm * 2.0 * PI / 60.0;
    let dth = 2.0 * PI / n as f64;
    let emf: [Vec<f64>; 3] = core::array::from_fn(|ph| {
        (0..n)
            .map(|t| -w * (flux[ph][(t + 1) % n] - flux[ph][(t + n - 1) % n]) / (2.0 * dth))
            .collect()
    });
    let sp = spectrum(&emf[0]);
    let p = spec.n_poles / 2;
    let fund = sp.get(p).copied().unwrap_or(0.0).max(1e-15);
    let harm: f64 = (2..).map(|h| h * p).take_while(|&k| k < sp.len()).map(|k| sp[k] * sp[k]).sum();
    let all: f64 = sp[1..].iter().map(|v| v * v).sum();
    let inter = (all - fund * fund - harm).max(0.0);
    let peaks: Vec<f64> = emf.iter().map(|e| e.iter().fold(0.0f64, |m, v| m.max(v.abs()))).collect();
    let pk = peaks.iter().cloned().fold(0.0, f64::max);
    let pmin = peaks.iter().cloned().fold(f64::MAX, f64::min);
    let ll: Vec<f64> = (0..n).map(|t| emf[0][t] - emf[1][t]).collect();
    let ll_pk = ll.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    let ke = ll_pk / (spec.rpm / 1000.0);
    EmfCurve {
        angles_deg: angles,
        flux,
        emf,
        coils,
        emf_peak_v: pk,
        thd_pct: 100.0 * harm.sqrt() / fund,
        interharm_pct: 100.0 * inter.sqrt() / fund,
        ke_ll_v_per_krpm: ke,
        kv_estime: if ke > 0.0 { 1000.0 / ke } else { 0.0 },
        unbalance_pct: if pk > 0.0 { 100.0 * (pk - pmin) / pk } else { 0.0 },
        spectrum_a: sp,
    }
}

/// Force magnétique résultante sur le rotor en fonction de sa position.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ForceCurve {
    /// Positions du rotor [°].
    pub angles_deg: Vec<f64>,
    /// Composante x [N].
    pub fx: Vec<f64>,
    /// Composante y [N].
    pub fy: Vec<f64>,
    /// Module maximal [N].
    pub max_n: f64,
    /// Module moyen [N].
    pub mean_n: f64,
}

/// Balayage de la force résultante sur `span_deg` degrés.
pub fn forces(model: &Model, span_deg: f64, n: usize) -> ForceCurve {
    let angles: Vec<f64> = (0..n).map(|i| span_deg * i as f64 / n as f64).collect();
    let xy: Vec<(f64, f64)> = crate::par::map(n, |i| model.force(&model.solve(angles[i].to_radians())));
    // en dessous du nanonewton, ce n'est que du bruit d'arrondi (rotor symétrique) : zéro exact
    let clean = |v: f64| if v.abs() < 1e-9 { 0.0 } else { v };
    let (fx, fy): (Vec<f64>, Vec<f64>) = xy.into_iter().map(|(x, y)| (clean(x), clean(y))).unzip();
    let mags: Vec<f64> = fx.iter().zip(&fy).map(|(x, y)| x.hypot(*y)).collect();
    ForceCurve {
        max_n: mags.iter().cloned().fold(0.0, f64::max),
        mean_n: mags.iter().sum::<f64>() / n as f64,
        angles_deg: angles,
        fx,
        fy,
    }
}

/// Induction au milieu de l'entrefer pour une position du rotor.
#[derive(Clone, Debug)]
pub struct GapField {
    /// Angle stator [°].
    pub theta_deg: Vec<f64>,
    /// Induction radiale [T].
    pub br: Vec<f64>,
    /// Induction tangentielle [T].
    pub bt: Vec<f64>,
    /// Spectre de Br par rang (par tour) [T].
    pub spectrum_br: Vec<f64>,
}

/// Induction d'entrefer sur `n` points réguliers.
pub fn gap_field(model: &Model, fld: &Field, n: usize) -> GapField {
    let (br, bt) = model.airgap_b(fld, model.r_mid_gap(), n);
    GapField {
        theta_deg: (0..n).map(|i| 360.0 * i as f64 / n as f64).collect(),
        spectrum_br: spectrum(&br),
        br,
        bt,
    }
}

/// Flux axial : induction d'entrefer de la tranche `i` sur `n` points
/// (`br` = composante normale, c'est-à-dire AXIALE ; `bt` = tangentielle).
pub fn gap_field_slice(model: &Model, fld: &Field, i: usize, n: usize) -> GapField {
    let s = &model.sections()[i];
    let (bn, bt) = s.gap_b(&fld.secs[i], s.y_mid_gap(), n);
    GapField { theta_deg: (0..n).map(|k| 360.0 * k as f64 / n as f64).collect(), spectrum_br: spectrum(&bn), br: bn, bt }
}

/// Flux axial : induction axiale d'entrefer sur toute la face du disque,
/// tranche par tranche : (rayon bas, rayon haut [m], Bz(θ) sur `nt` points).
pub fn disc_gap_map(model: &Model, fld: &Field, nt: usize) -> Vec<(f64, f64, Vec<f64>)> {
    (0..model.n_sections())
        .map(|i| {
            let (r0, dr) = model.slice_radius(i);
            let s = &model.sections()[i];
            (r0 - dr / 2.0, r0 + dr / 2.0, s.gap_b(&fld.secs[i], s.y_mid_gap(), nt).0)
        })
        .collect()
}

/// Flux axial : carte A et |B| de la tranche `i` développée, sur une grille
/// (cote axiale y, angle θ) : `r` contient les cotes y [m] (de la culasse du
/// stator vers le rotor), `th` les angles. Bz = (1/r0)·∂A/∂θ, Bθ = −∂A/∂y.
pub fn slice_field_map(model: &Model, fld: &Field, i: usize, n_slot: usize, n_gap: usize, n_mag: usize, nt: usize) -> FieldMap {
    let s = &model.sections()[i];
    let g = &s.g;
    let r0 = s.rho(s.y_mid_gap());
    let mut ys = Vec::new();
    for (a, b, n) in [(g.y_sb, g.y_s, n_slot), (g.y_s, g.y_m, n_gap), (g.y_m, g.y_r, n_mag)] {
        let (lo, hi) = (a.min(b), a.max(b));
        for k in 0..n {
            ys.push(lo + (hi - lo) * (k as f64 + 0.5) / n as f64);
        }
    }
    ys.sort_by(|a, b| a.total_cmp(b));
    let nr = ys.len();
    let th: Vec<f64> = (0..nt).map(|j| 2.0 * PI * j as f64 / nt as f64).collect();
    let rows: Vec<Vec<Option<f64>>> = crate::par::map(nr, |k| th.iter().map(|&t| s.a_at(&fld.secs[i], ys[k], t)).collect());
    let a: Vec<Option<f64>> = rows.concat();
    let dth = 2.0 * PI / nt as f64;
    let mut b = vec![None; nr * nt];
    for k in 0..nr {
        for j in 0..nt {
            let get = |kk: usize, jj: usize| a[kk * nt + (jj % nt)];
            if let (Some(_), Some(ap), Some(am)) = (get(k, j), get(k, j + 1), get(k, j + nt - 1)) {
                let bz = (ap - am) / (2.0 * dth * r0);
                let (lo, hi) = (k.saturating_sub(1), (k + 1).min(nr - 1));
                if let (Some(a1), Some(a0)) = (get(hi, j), get(lo, j)) {
                    let bt = -(a1 - a0) / (ys[hi] - ys[lo]);
                    b[k * nt + j] = Some(bz.hypot(bt));
                }
            }
        }
    }
    FieldMap { r: ys, th, a, b, nr, nt }
}

/// Flux axial : effets d'un défaut d'alignement du rotor.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AxialDefects {
    /// Attraction axiale d'un disque, rotor centré [N].
    pub attraction_n: f64,
    /// Force axiale nette sur l'arbre pour le décalage axial donné [N].
    pub offset_force_n: f64,
    /// Raideur axiale magnétique [N/µm] (négative : le rotor est attiré vers le côté le plus proche).
    pub offset_stiffness_n_per_um: f64,
    /// Moment de basculement pour l'inclinaison donnée [N·m].
    pub tilt_moment_nm: f64,
    /// Raideur angulaire magnétique [N·m/mrad] (négative : déstabilisante).
    pub tilt_stiffness_nm_per_mrad: f64,
    /// Inclinaison correspondante [mrad].
    pub tilt_mrad: f64,
}

/// Flux axial : décalage axial du rotor (`offset` = fraction de l'entrefer)
/// et inclinaison du disque (`tilt` = variation d'entrefer au rayon
/// extérieur, en fraction de l'entrefer), par la méthode de perméance
/// appliquée tranche par tranche : l'induction locale est divisée par
/// (1 ± δ/g'), avec g' = g + h_aimant/μr. Pression magnétique Bn²/2μ0,
/// moyennée sur `n` positions du rotor. Avec deux entrefers, les deux disques
/// rotors (solidaires de l'arbre) voient des variations opposées.
pub fn axial_defects(model: &Model, offset: f64, tilt: f64, n: usize) -> AxialDefects {
    let s = &model.spec;
    if s.topology != crate::geometry::Topology::Axial {
        return AxialDefects::default();
    }
    let g = s.airgap * 1e-3;
    let geff = (s.airgap + s.magnet_th / s.mur_magnet.max(1.0)) * 1e-3;
    let two = s.axial.gaps >= 2;
    let r_out = s.axial.r_out * 1e-3;
    let alpha = tilt * g / r_out; // rad
    let nt = 720;
    let dth = 2.0 * PI / nt as f64;
    let mu2 = 2.0 * crate::magnets::MU0;
    let per = s.cogging_period_deg();
    let parts: Vec<(f64, f64, f64)> = crate::par::map(n.max(1), |ip| {
        let f = model.solve((per * ip as f64 / n.max(1) as f64).to_radians());
        let (mut f0, mut fnet, mut mom) = (0.0, 0.0, 0.0);
        for i in 0..model.n_sections() {
            let (r0, dr) = model.slice_radius(i);
            let sec = &model.sections()[i];
            let bn = sec.gap_b(&f.secs[i], sec.y_mid_gap(), nt).0;
            let k_off = (offset * g / geff).clamp(-0.95, 0.95);
            for (t, b) in bn.iter().enumerate() {
                let th = t as f64 * dth;
                let da = dr * r0 * dth;
                let p0 = b * b / mu2;
                f0 += p0 * da;
                // décalage axial : côté + plus proche, côté − plus loin (ou rien)
                let p_plus = p0 / (1.0 - k_off).powi(2);
                let p_minus = if two { p0 / (1.0 + k_off).powi(2) } else { p0 };
                fnet += (p_plus - p_minus) * da;
                // inclinaison : entrefer + r·α·sinθ côté +, opposé côté −
                let kt = (alpha * r0 * th.sin() / geff).clamp(-0.95, 0.95);
                let q_plus = p0 / (1.0 + kt).powi(2);
                let q_minus = if two { p0 / (1.0 - kt).powi(2) } else { 0.0 };
                mom += (q_minus - q_plus) * r0 * th.sin() * da;
            }
        }
        (f0, fnet, mom)
    });
    let m = parts.len() as f64;
    let (f0, fnet, mom) = parts.iter().fold((0.0, 0.0, 0.0), |a, v| (a.0 + v.0 / m, a.1 + v.1 / m, a.2 + v.2 / m));
    AxialDefects {
        attraction_n: f0,
        offset_force_n: fnet,
        offset_stiffness_n_per_um: if offset.abs() > 0.0 { -fnet / (offset * g * 1e6) } else { 0.0 },
        tilt_moment_nm: mom,
        tilt_stiffness_nm_per_mrad: if alpha.abs() > 0.0 { -mom / (alpha * 1e3) } else { 0.0 },
        tilt_mrad: alpha * 1e3,
    }
}

/// Carte |B| et A sur une grille polaire (nr × nt) couvrant encoches → aimants.
pub struct FieldMap {
    /// Rayons de la grille [m].
    pub r: Vec<f64>,
    /// Angles de la grille [rad].
    pub th: Vec<f64>,
    /// Potentiel vecteur [Wb/m] (None dans le fer).
    pub a: Vec<Option<f64>>,
    /// Module de l'induction [T] (None dans le fer).
    pub b: Vec<Option<f64>>,
    /// Nombre de rayons.
    pub nr: usize,
    /// Nombre d'angles.
    pub nt: usize,
}

/// Rayons de carte resserrés là où la physique se joue (entrefer, ouvertures).
pub fn map_radii(model: &Model, n_slot: usize, n_gap: usize, n_mag: usize) -> Vec<f64> {
    let (rsb, rs, rm, rr) = model.radii();
    let mut r = Vec::new();
    for (a, b, n) in [(rsb, rs, n_slot), (rs, rm, n_gap), (rm, rr, n_mag)] {
        for i in 0..n {
            r.push(a + (b - a) * (i as f64 + 0.5) / n as f64);
        }
    }
    r
}

/// Carte A et |B| sur la grille (rayons `r`, `nt` angles).
pub fn field_map(model: &Model, fld: &Field, r: Vec<f64>, nt: usize) -> FieldMap {
    let nr = r.len();
    let th: Vec<f64> = (0..nt).map(|j| 2.0 * PI * j as f64 / nt as f64).collect();
    let mut a = vec![None; nr * nt];
    for i in 0..nr {
        for j in 0..nt {
            a[i * nt + j] = model.a_at(fld, r[i], th[j]);
        }
    }
    let dth = 2.0 * PI / nt as f64;
    let mut b = vec![None; nr * nt];
    for i in 0..nr {
        for j in 0..nt {
            let get = |ii: usize, jj: usize| a[ii * nt + (jj % nt)];
            if let (Some(_), Some(ap), Some(am)) = (get(i, j), get(i, j + 1), get(i, j + nt - 1)) {
                let brv = (ap - am) / (2.0 * dth * r[i]);
                let (lo, hi) = (i.saturating_sub(1), (i + 1).min(nr - 1));
                if let (Some(a1), Some(a0)) = (get(hi, j), get(lo, j)) {
                    let btv = -(a1 - a0) / (r[hi] - r[lo]);
                    b[i * nt + j] = Some(brv.hypot(btv));
                }
            }
        }
    }
    FieldMap { r, th, a, b, nr, nt }
}

/// Synthèse des critères pour une répartition.
#[derive(Clone, Debug, Serialize, Default, serde::Deserialize)]
pub struct Report {
    /// Nom de la variante.
    pub variant: String,
    /// Balourd résiduel [mg·cm].
    pub balourd_mg_cm: f64,
    /// Autocorrélation circulaire maximale des écarts (None si rotor régulier).
    pub autocorr_max: Option<f64>,
    /// Métriques du cogging.
    pub cogging: CoggingMetrics,
    /// Force radiale maximale [N].
    pub force_max_n: f64,
    /// THD de la FEM [%].
    pub emf_thd_pct: f64,
    /// Interharmoniques de la FEM [%].
    pub emf_interharm_pct: f64,
    /// Constante de FEM ligne-ligne crête [V/krpm].
    pub ke_ll_v_per_krpm: f64,
    /// KV estimé [tr/min/V] (modèle 2D).
    pub kv_estime: f64,
    /// Notation du bobinage.
    pub winding: String,
    /// écart libre minimal entre aimants voisins [°] (négatif = chevauchement)
    pub min_magnet_gap_deg: f64,
}

/// Assemble le rapport à partir de courbes déjà calculées.
pub fn report_from(spec: &MotorSpec, cg: &CoggingCurve, em: &EmfCurve, fc: &ForceCurve) -> Report {
    Report {
        variant: spec.name.clone(),
        balourd_mg_cm: geometry::balourd_mg_cm(spec),
        autocorr_max: geometry::autocorr_max(spec),
        cogging: cg.metrics.clone(),
        force_max_n: fc.max_n,
        emf_thd_pct: em.thd_pct,
        emf_interharm_pct: em.interharm_pct,
        ke_ll_v_per_krpm: em.ke_ll_v_per_krpm,
        kv_estime: em.kv_estime,
        winding: winding::notation(&em.coils),
        min_magnet_gap_deg: geometry::magnet_gaps_deg(spec).into_iter().fold(f64::MAX, f64::min),
    }
}

/// Calcule toutes les courbes puis le rapport.
pub fn report(spec: &MotorSpec, model: &Model, span_deg: f64, n_cog: usize, n_emf: usize) -> Report {
    let cg = cogging(model, span_deg, n_cog);
    let fc = forces(model, 360.0, (n_cog / 2).max(16));
    let em = emf(model, n_emf);
    report_from(spec, &cg, &em, &fc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subdomain::Resolution;

    #[test]
    fn continuite_entrefer_aimant() {
        let s = MotorSpec::irregular_example();
        let m = Model::new(&s, Resolution { nh: 200, nk: 12 }).unwrap();
        let f = m.solve(0.2);
        let (_, _, rm, _) = m.radii();
        for k in 0..12 {
            let th = k as f64 * 0.5;
            let a1 = m.a_at(&f, rm * (1.0 - 1e-9), th).unwrap();
            let a2 = m.a_at(&f, rm * (1.0 + 1e-9), th).unwrap();
            assert!((a1 - a2).abs() < 1e-6 * a1.abs().max(1e-6), "{a1} {a2}");
        }
    }

    #[test]
    fn fem_equilibree_b0() {
        let s = MotorSpec::default();
        let m = Model::new(&s, Resolution { nh: 200, nk: 12 }).unwrap();
        let e = emf(&m, 360);
        assert!(e.unbalance_pct < 1.0, "déséquilibre {}", e.unbalance_pct);
        assert!(e.emf_peak_v > 0.1);
    }
}

/// Fonctionnement en charge (courants sinusoïdaux alignés sur la FEM).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct LoadCurve {
    /// Positions du rotor [°].
    pub angles_deg: Vec<f64>,
    /// Couple en charge sur le rotor [N·m] (Maxwell).
    pub torque: Vec<f64>,
    /// Couple estimé par l'énergie, Σ i·dλ/dθ + couple de détente [N·m] (contrôle).
    pub torque_energy: Vec<f64>,
    /// Courants de phase [A].
    pub currents: [Vec<f64>; 3],
    /// Couple moyen [N·m].
    pub avg_torque: f64,
    /// Ondulation crête-à-crête rapportée au couple moyen [%].
    pub ripple_pct: f64,
    /// Décomposition de l'ondulation, crête-à-crête rapportée au couple moyen [%] :
    /// couple de détente, couple mutuel (Σ i·dλ/dθ), reste (réluctance et
    /// interaction des harmoniques : couple de Maxwell − détente − mutuel).
    #[serde(default)]
    pub ripple_parts_pct: [f64; 3],
    /// Couple mutuel Σ i·dλ/dθ [N·m] (même positions que `torque`).
    #[serde(default)]
    pub torque_mutual: Vec<f64>,
    /// Couple de détente interpolé [N·m] (même positions que `torque`).
    #[serde(default)]
    pub torque_cogging: Vec<f64>,
    /// Constante de couple [N·m/A crête].
    pub kt: f64,
    /// Couple moyen en fonction de l'angle du courant : (angle [°él], couple [N·m]).
    pub angle_sweep: Vec<[f64; 2]>,
    /// Matrice d'inductances [H] (hors têtes de bobines).
    pub inductances: [[f64; 3]; 3],
    /// Inductance synchrone L − M [H], têtes de bobines comprises.
    pub ls: f64,
    /// Inductance de fuite des têtes de bobines par phase [H] (comprise dans `ls`).
    pub l_end: f64,
    /// Résistance de phase à 20 °C [Ω].
    pub r_phase: f64,
    /// Pertes Joule [W].
    pub p_joule: f64,
    /// Puissance mécanique à la vitesse de la spécification [W].
    pub p_mech: f64,
}

/// Longueur d'une spire, pas des dents : (longueur active, pas d'encoche, ouverture de la bobine en pas).
fn turn_geometry(model: &Model) -> (f64, f64, impl Fn(&crate::geometry::CoilDef) -> f64) {
    use crate::geometry::Topology;
    let s = &model.spec;
    let q = model.q as f64;
    let (active, pitch) = match s.topology {
        Topology::Axial => ((s.axial.r_out - s.axial.r_in) * 1e-3, 2.0 * PI * s.axial_r_mean() / q),
        _ => (s.length(), 2.0 * PI * s.r_bore() / q),
    };
    let span = move |c: &crate::geometry::CoilDef| {
        let d = (c.go as i64 - c.ret as i64).unsigned_abs() as f64;
        d.min(q - d).max(1.0)
    };
    (active, pitch, span)
}

/// Inductance de fuite des têtes de bobines d'une phase [H] :
/// L = Σ_bobines μ0 · N² · l_têtes · λ / a², avec l_têtes la longueur des deux
/// têtes d'une spire (demi-cercles de diamètre l'ouverture de la bobine,
/// comme pour la résistance) et a le nombre de voies en parallèle. Les
/// mutuelles entre têtes sont négligées.
pub fn end_winding_inductance(model: &Model) -> f64 {
    let s = &model.spec;
    let (_, pitch, span) = turn_geometry(model);
    let paths = s.load.parallel_paths.max(1.0);
    let mu0 = 4e-7 * PI;
    model.coils.iter().filter(|c| c.phase == 0).map(|c| mu0 * c.turns * c.turns * 2.0 * (PI / 2.0 * span(c) * pitch) * s.load.end_lambda.max(0.0)).sum::<f64>() / (paths * paths)
}

/// Résistance de phase estimée (cuivre à 20 °C) [Ω].
pub fn phase_resistance(model: &Model) -> f64 {
    let s = &model.spec;
    // longueur moyenne d'une spire : deux côtés actifs + deux têtes de bobine
    let (active, pitch, span) = turn_geometry(model);
    let mut conductors = vec![0.0; model.q];
    for c in &model.coils {
        conductors[c.go] += c.turns;
        conductors[c.ret] += c.turns;
    }
    let nmax = conductors.iter().cloned().fold(0.0, f64::max).max(1.0);
    let wire = s.load.fill_factor * model.slot_area() / nmax;
    let paths = s.load.parallel_paths.max(1.0);
    let len: f64 = model.coils.iter().filter(|c| c.phase == 0).map(|c| c.turns * 2.0 * (active + PI / 2.0 * span(c) * pitch)).sum();
    1.72e-8 * len / wire.max(1e-12) / (paths * paths)
}

/// Loi des courants de phase alignés sur la FEM : (θ rotor [rad], angle du
/// courant γ [rad]) → courants [A] (crête `load.i_peak`).
pub fn current_law(model: &Model, em: &EmfCurve) -> impl Fn(f64, f64) -> [f64; 3] + Sync + use<> {
    let s = &model.spec;
    let p = (s.n_poles / 2) as f64;
    let i0 = s.load.i_peak;
    // phase électrique de −e (le courant en phase avec −e = dλ/dθ donne un couple moteur)
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
    move |th: f64, gamma: f64| -> [f64; 3] { core::array::from_fn(|k| i0 * (p * th - phi[k] + gamma).cos()) }
}

/// Couple moyen [N·m] et ondulation crête-à-crête [%] en charge, version
/// allégée (n positions sur un tour, angle du courant de la spécification) :
/// sert à l'Ajustement I.A.
pub fn ripple_quick(model: &Model, em: &EmfCurve, n: usize) -> (f64, f64) {
    if model.spec.load.i_peak <= 0.0 {
        return (0.0, 0.0);
    }
    let law = current_law(model, em);
    let g = model.spec.load.angle_deg.to_radians();
    let tq: Vec<f64> = crate::par::map(n, |k| {
        let th = 2.0 * PI * k as f64 / n as f64;
        model.torque(&model.solve_i(th, &law(th, g)))
    });
    let avg = tq.iter().sum::<f64>() / n as f64;
    let (mn, mx) = tq.iter().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    (avg, if avg.abs() > 1e-12 { 100.0 * (mx - mn) / avg.abs() } else { 0.0 })
}

fn pp(v: &[f64]) -> f64 {
    let (mn, mx) = v.iter().fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    if v.is_empty() { 0.0 } else { mx - mn }
}

/// Analyse en charge à partir de la FEM à vide (pour aligner les courants).
pub fn load(model: &Model, em: &EmfCurve, cog: &CoggingCurve, n: usize) -> LoadCurve {
    let s = &model.spec;
    let p = (s.n_poles / 2) as f64;
    let i0 = s.load.i_peak;
    let currents_at = current_law(model, em);
    let gamma0 = s.load.angle_deg.to_radians();
    let angles: Vec<f64> = (0..n).map(|k| 360.0 * k as f64 / n as f64).collect();
    let mut torque = Vec::with_capacity(n);
    let mut te = Vec::with_capacity(n);
    let mut tmut = Vec::with_capacity(n);
    let mut tcog = Vec::with_capacity(n);
    let mut cur = [vec![0.0; n], vec![0.0; n], vec![0.0; n]];
    let ne = em.flux[0].len();
    let dth = 2.0 * PI / ne as f64;
    let tq: Vec<f64> = crate::par::map(n, |k| {
        let th = angles[k].to_radians();
        model.torque(&model.solve_i(th, &currents_at(th, gamma0)))
    });
    for (k, a) in angles.iter().enumerate() {
        let th = a.to_radians();
        let i = currents_at(th, gamma0);
        for ph in 0..3 {
            cur[ph][k] = i[ph];
        }
        torque.push(tq[k]);
        // contrôle énergétique : Σ i·dλ/dθ + couple de détente (inductances constantes)
        let idx = ((a / 360.0) * ne as f64).round() as usize % ne;
        let dl: f64 = (0..3).map(|ph| i[ph] * (em.flux[ph][(idx + 1) % ne] - em.flux[ph][(idx + ne - 1) % ne]) / (2.0 * dth)).sum();
        let tc = crate::dynamics::interp_periodic(&cog.angles_deg, &cog.torque_nm, cog.metrics.span_deg, *a);
        te.push(dl + tc);
        tmut.push(dl);
        tcog.push(tc);
    }
    let avg = torque.iter().sum::<f64>() / n as f64;
    let rest: Vec<f64> = (0..n).map(|k| torque[k] - tmut[k] - tcog[k]).collect();
    let rel = |v: &[f64]| if avg.abs() > 1e-12 { 100.0 * pp(v) / avg.abs() } else { 0.0 };
    let ripple_parts_pct = [rel(&tcog), rel(&tmut), rel(&rest)];
    let (mn, mx) = torque.iter().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    // couple moyen selon l'angle du courant (12 positions par angle)
    let m = 12;
    let all: Vec<f64> = crate::par::map(13 * m, |idx| {
        let (k, j) = (idx / m, idx % m);
        let g = -90.0 + 15.0 * k as f64;
        let th = 2.0 * PI * j as f64 / m as f64 / p.max(1.0) + 0.013;
        model.torque(&model.solve_i(th, &currents_at(th, g.to_radians())))
    });
    let sweep: Vec<[f64; 2]> = (0..=12).map(|k| [-90.0 + 15.0 * k as f64, all[k * m..(k + 1) * m].iter().sum::<f64>() / m as f64]).collect();
    let mut lm = [[0.0; 3]; 3];
    for ph in 0..3 {
        let mut i = [0.0; 3];
        i[ph] = 1.0;
        let lam = model.phase_flux(&model.solve_armature(&i));
        for k in 0..3 {
            lm[k][ph] = lam[k];
        }
    }
    let r = phase_resistance(model);
    let w = s.rpm * 2.0 * PI / 60.0;
    LoadCurve {
        angles_deg: angles,
        torque,
        torque_energy: te,
        currents: cur,
        avg_torque: avg,
        ripple_pct: if avg.abs() > 1e-12 { 100.0 * (mx - mn) / avg.abs() } else { 0.0 },
        ripple_parts_pct,
        torque_mutual: tmut,
        torque_cogging: tcog,
        kt: if i0 > 0.0 { avg / i0 } else { 0.0 },
        angle_sweep: sweep,
        inductances: lm,
        ls: lm[0][0] - lm[0][1] + end_winding_inductance(model),
        l_end: end_winding_inductance(model),
        r_phase: r,
        p_joule: 1.5 * r * i0 * i0,
        p_mech: avg * w,
    }
}

#[cfg(test)]
mod tests_charge {
    use super::*;
    use crate::geometry::MotorSpec;
    use crate::subdomain::Resolution;

    /// Le couple de Maxwell en charge doit égaler Σ i·dλ/dθ + couple de détente
    /// (bilan énergétique, inductances constantes), et être maximal à angle nul.
    #[test]
    fn couple_en_charge_coherent() {
        let s = MotorSpec::default();
        let m = Model::new(&s, Resolution { nh: 200, nk: 12 }).unwrap();
        let em = emf(&m, 360);
        let cg = cogging(&m, s.cogging_period_regular_deg(), 60);
        let l = load(&m, &em, &cg, 72);
        assert!(l.avg_torque > 0.0, "couple moyen {}", l.avg_torque);
        let te = l.torque_energy.iter().sum::<f64>() / l.torque_energy.len() as f64;
        assert!((te - l.avg_torque).abs() < 0.03 * l.avg_torque, "Maxwell {} vs énergie {}", l.avg_torque, te);
        let best = l.angle_sweep.iter().fold((0.0, f64::MIN), |b, v| if v[1] > b.1 { (v[0], v[1]) } else { b });
        assert!(best.0.abs() <= 15.0, "angle optimal {}", best.0);
        assert!(l.ls > 0.0 && l.r_phase > 0.0);
    }
}

/// Force magnétique déséquilibrée due à une EXCENTRICITÉ du rotor (méthode
/// de perméance, petite excentricité) : l'induction radiale d'entrefer est
/// modulée par 1 / (1 − ε·(g/g_eff)·cos(θ − θe)), avec θe fixe (statique) ou
/// tournant avec le rotor (dynamique) ; g_eff = g + h_aimant/μr.
pub fn forces_eccentric(model: &Model, eps: f64, dynamic: bool, n: usize) -> ForceCurve {
    let s = &model.spec;
    let g = s.airgap;
    let geff = g + s.magnet_th / s.mur_magnet.max(1.0);
    let k = (eps * g / geff).clamp(0.0, 0.95);
    let r = model.r_mid_gap();
    let nt = 720;
    let dth = 2.0 * PI / nt as f64;
    let angles: Vec<f64> = (0..n).map(|i| 360.0 * i as f64 / n as f64).collect();
    let polar = s.topology != crate::geometry::Topology::Axial;
    let len = s.length();
    let sgn = if s.topology == crate::geometry::Topology::RadialInner { 1.0 } else { -1.0 };
    let xy: Vec<(f64, f64)> = crate::par::map(n, |ia| {
        let th_r = angles[ia].to_radians();
        let (br, bt) = model.airgap_b(&model.solve(th_r), r, nt);
        let th_e = if dynamic { th_r } else { 0.0 };
        let (mut x, mut y) = (0.0, 0.0);
        if polar {
            for t in 0..nt {
                let th = t as f64 * dth;
                let b = br[t] / (1.0 - k * (th - th_e).cos());
                let pn = (b * b - bt[t] * bt[t]) / (2.0 * crate::magnets::MU0);
                let pt = b * bt[t] / crate::magnets::MU0;
                let (sn, cs) = th.sin_cos();
                x += (pn * cs - pt * sn) * dth;
                y += (pn * sn + pt * cs) * dth;
            }
        }
        (sgn * len * r * x, sgn * len * r * y)
    });
    let (fx, fy): (Vec<f64>, Vec<f64>) = xy.into_iter().unzip();
    let mags: Vec<f64> = fx.iter().zip(&fy).map(|(x, y)| x.hypot(*y)).collect();
    ForceCurve { max_n: mags.iter().cloned().fold(0.0, f64::max), mean_n: mags.iter().sum::<f64>() / n.max(1) as f64, angles_deg: angles, fx, fy }
}

#[cfg(test)]
mod tests_exc {
    use super::*;
    use crate::geometry::MotorSpec;
    use crate::subdomain::Resolution;

    /// Rotor régulier symétrique : force nulle sans excentricité, force croissante avec ε
    /// et dirigée vers l'entrefer le plus petit (attraction).
    #[test]
    fn excentricite_cree_une_attraction() {
        let s = MotorSpec::default();
        let m = Model::new(&s, Resolution { nh: 200, nk: 12 }).unwrap();
        let f0 = forces_eccentric(&m, 0.0, false, 12);
        let f1 = forces_eccentric(&m, 0.1, false, 12);
        let f2 = forces_eccentric(&m, 0.2, false, 12);
        assert!(f0.max_n < 1e-3, "{}", f0.max_n);
        assert!(f2.mean_n > 1.8 * f1.mean_n && f1.mean_n > 0.1, "{} {}", f1.mean_n, f2.mean_n);
    }
}

#[cfg(test)]
mod tests_axial_defects {
    use super::*;
    use crate::geometry::{MotorSpec, Topology};
    use crate::subdomain::Resolution;

    /// Flux axial à deux entrefers : centré, aucune force nette ; décalé ou
    /// incliné, le rotor est attiré du côté le plus proche (raideurs
    /// négatives), effet croissant avec le défaut ; l'attraction d'un disque
    /// est proche de celle du tenseur de Maxwell.
    #[test]
    fn defauts_d_alignement_axial() {
        let mut a = MotorSpec::default();
        a.topology = Topology::Axial;
        a.axial.r_in = 20.0;
        a.axial.r_out = 40.0;
        a.axial.slices = 3;
        a.axial.gaps = 2;
        a.n_slots = 12;
        a.slot_depth = 12.0;
        a.slot_w = 6.0;
        a.slot_open = 6.0;
        a.airgap = 1.0;
        a.magnet_th = 4.0;
        let m = Model::new(&a, Resolution { nh: 120, nk: 10 }).unwrap();
        let d0 = axial_defects(&m, 0.0, 0.0, 4);
        assert!(d0.offset_force_n.abs() < 1e-9 * d0.attraction_n && d0.tilt_moment_nm.abs() < 1e-9 * d0.attraction_n);
        let d1 = axial_defects(&m, 0.1, 0.1, 4);
        let d2 = axial_defects(&m, 0.2, 0.2, 4);
        assert!(d1.offset_force_n > 0.0 && d2.offset_force_n > 1.9 * d1.offset_force_n);
        assert!(d1.offset_stiffness_n_per_um < 0.0 && d1.tilt_stiffness_nm_per_mrad < 0.0);
        let maxwell = m.axial_attraction(&m.solve(0.0)).abs();
        assert!((d0.attraction_n / maxwell - 1.0).abs() < 0.15, "{} / {}", d0.attraction_n, maxwell);
    }
}

