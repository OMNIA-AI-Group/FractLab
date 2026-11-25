//! Grandeurs dépendant de la vitesse : pertes fer, frottements, échauffement,
//! excitations acoustiques — et effet de la température sur les aimants.
//!
//! Ces modèles complètent le calcul magnétostatique (validé) par des
//! formulations d'ingénierie classiques, aux coefficients modifiables :
//! - pertes fer : modèle de Bertotti (hystérésis + courants de Foucault
//!   classiques) appliqué à la forme d'onde RÉELLE de l'induction dans chaque
//!   dent et chaque portion de culasse ;
//! - roulements : couple de Palmgren (terme visqueux + terme de charge), la
//!   charge comprenant le poids du rotor, la force magnétique résultante et le
//!   balourd centrifuge ;
//! - ventilation : frottement turbulent/laminaire d'un cylindre et d'un disque
//!   tournants ;
//! - thermique : un nœud, convection forcée d'un cylindre tournant
//!   (Nu = 0,133 Re^(2/3) Pr^(1/3)) plus convection naturelle ;
//! - aimants : Br(T) = Br20·(1 + α·(T − 20 °C)) ; FEM ∝ Br, couples et forces ∝ Br².
//! - son : fréquences et amplitudes des raies d'excitation (couple de détente
//!   et force radiale), pondération A ; c'est un INDICATEUR d'excitation, pas
//!   un niveau de pression acoustique (qui dépend de la structure).

use crate::compute::Results;
use crate::geometry::MotorSpec;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Paramètres physiques complémentaires (frottement, fer, thermique, vibration).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DynParams {
    /// Coefficient de frottement de charge des roulements (≈ 0,0015 à billes).
    pub bearing_mu: f64,
    /// Diamètre moyen des roulements [mm].
    pub bearing_dm_mm: f64,
    /// Coefficient visqueux de Palmgren f0 (≈ 1,5 à 2 graisse).
    pub bearing_f0: f64,
    /// Viscosité du lubrifiant [cSt].
    pub bearing_visc_cst: f64,
    /// Épaisseur des tôles [mm].
    pub lam_thick_mm: f64,
    /// Coefficient d'hystérésis kh [W/(kg·Hz·T²)].
    pub kh: f64,
    /// Conductivité des tôles [S/m].
    pub sigma_iron: f64,
    /// Balourd résiduel d'équilibrage du rotor [mg·cm].
    pub residual_unbalance_mg_cm: f64,
    /// Température ambiante [°C].
    pub t_ambient: f64,
    /// Coefficient de température de Br [%/K] (NdFeB ≈ −0,12).
    pub alpha_br_pct: f64,
    /// Coefficient de convection naturelle [W/(m²·K)].
    pub h_natural: f64,
    /// Raideur radiale des deux roulements réunis [N/µm].
    pub bearing_stiffness_n_per_um: f64,
    /// Taux d'amortissement du mode radial (0,01 à 0,1).
    pub damping_ratio: f64,
    /// Inertie de la charge entraînée (hélice…) [kg·m²].
    pub load_inertia: f64,
}

impl Default for DynParams {
    fn default() -> Self {
        DynParams {
            bearing_mu: 0.0015,
            bearing_dm_mm: 6.5,
            bearing_f0: 1.5,
            bearing_visc_cst: 20.0,
            lam_thick_mm: 0.2,
            kh: 0.0165,
            sigma_iron: 2.2e6,
            residual_unbalance_mg_cm: 2.0,
            t_ambient: 20.0,
            alpha_br_pct: -0.12,
            h_natural: 10.0,
            bearing_stiffness_n_per_um: 10.0,
            damping_ratio: 0.05,
            load_inertia: 0.0,
        }
    }
}

const RHO_STEEL: f64 = 7650.0;
const RHO_MAGNET: f64 = 7500.0;
const RHO_AIR: f64 = 1.2;
const MU_AIR: f64 = 1.81e-5;
const K_AIR: f64 = 0.026;
const PR_AIR: f64 = 0.71;
const CP_STEEL: f64 = 460.0;
const CP_MAGNET: f64 = 440.0;

/// Masses et inertie du moteur, déduites de la géométrie.
#[derive(Clone, Debug, Default)]
pub struct Masses {
    /// Une dent du stator [kg].
    pub tooth: f64,
    /// Une portion de culasse statorique (1/Q) [kg].
    pub yoke_seg: f64,
    /// Aimants [kg].
    pub magnets: f64,
    /// Cloche du rotor [kg].
    pub rotor_yoke: f64,
    /// Rotor complet (aimants + cloche) [kg].
    pub rotor: f64,
    /// Fer du stator [kg].
    pub stator: f64,
    /// Moment d'inertie du rotor [kg·m²].
    pub inertia: f64,
}

/// Masses calculées à partir de la spécification (dimensions en mm).
pub fn masses(s: &MotorSpec) -> Masses {
    use crate::geometry::Topology;
    let rg = s.rings();
    let q = s.n_slots as f64;
    let l = s.length();
    let ring = |a: f64, b: f64| PI * (a.max(b).powi(2) - a.min(b).powi(2));
    if s.topology == Topology::Axial {
        // disques : surfaces annulaires × épaisseurs axiales
        let (ri, ro) = (s.axial.r_in * 1e-3, s.axial.r_out * 1e-3);
        let ann = PI * (ro * ro - ri * ri);
        let rm = 0.5 * (ri + ro);
        let slot_frac = (s.slot_w.max(s.slot_open) * 1e-3 * q / (2.0 * PI * rm)).min(0.95);
        let gaps = s.axial.gaps.clamp(1, 2) as f64;
        let teeth = ann * (1.0 - slot_frac) * s.slot_depth_m() * RHO_STEEL * gaps;
        let yoke = ann * s.yoke_th * 1e-3 * RHO_STEEL;
        let arc = 0.5 * (s.magnet_arc_ratio + s.axial.arc_ratio_out);
        let magnets = ann * arc * s.magnet_th * 1e-3 * RHO_MAGNET * gaps;
        let rotor_yoke = ann * s.rotor_yoke_th * 1e-3 * RHO_STEEL * gaps;
        let inertia = 0.5 * (magnets + rotor_yoke) * (ro * ro + ri * ri);
        return Masses { tooth: teeth / q, yoke_seg: yoke / q, magnets, rotor_yoke, rotor: magnets + rotor_yoke, stator: teeth + yoke, inertia };
    }
    let body_frac = 1.0 - rg.beta_body * q / (2.0 * PI);
    let tip_frac = 1.0 - rg.beta_open * q / (2.0 * PI);
    let tooth = (ring(rg.sb, rg.tip) * body_frac + ring(rg.tip, rg.bore) * tip_frac) / q * l * RHO_STEEL;
    let yoke_seg = ring(rg.yoke.0, rg.yoke.1) / q * l * RHO_STEEL;
    let magnets = ring(rg.mag.0, rg.mag.1) * s.magnet_arc_ratio * l * RHO_MAGNET;
    let r_o = rg.rotor_yoke.0.max(rg.rotor_yoke.1);
    // cloche : fond plein pour un rotor extérieur
    let cap = if s.topology == Topology::RadialOuter { PI * r_o * r_o * s.rotor_yoke_th * 1e-3 * RHO_STEEL } else { 0.0 };
    let rotor_yoke = ring(rg.rotor_yoke.0, rg.rotor_yoke.1) * l * RHO_STEEL + cap;
    let i2 = |a: f64, b: f64| a * a + b * b;
    let inertia = 0.5 * magnets * i2(rg.mag.0, rg.mag.1) + 0.5 * rotor_yoke * i2(rg.rotor_yoke.0, rg.rotor_yoke.1);
    Masses { tooth, yoke_seg, magnets, rotor_yoke, rotor: magnets + rotor_yoke, stator: q * (tooth + yoke_seg), inertia }
}

/// Induction dans chaque dent et chaque portion de culasse sur un tour.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct IronSweep {
    /// Positions du rotor [°] (tour complet, pas régulier).
    pub angles_deg: Vec<f64>,
    /// Induction moyenne dans la dent j [T] : b_tooth[j][k].
    pub b_tooth: Vec<Vec<f64>>,
    /// Induction dans la portion de culasse j [T] : b_yoke[j][k].
    pub b_yoke: Vec<Vec<f64>>,
}

/// Balayage des flux de dents sur un tour (`n` positions).
pub fn iron_sweep(model: &crate::subdomain::Model, n: usize) -> IronSweep {
    let s = &model.spec;
    let q = s.n_slots;
    let l = s.length();
    // dent réelle : en flux axial, pas d'encoche au rayon moyen moins la largeur d'encoche
    let w_t = match s.topology {
        crate::geometry::Topology::Axial => (2.0 * std::f64::consts::PI * s.axial_r_mean() / q as f64 - s.slot_w.max(s.slot_open) * 1e-3).max(1e-5),
        _ => (s.tooth_w * 1e-3).max(1e-5),
    };
    let w_y = (s.yoke_th * 1e-3).max(1e-5);
    // flux d'un seul côté du stator (le modèle additionne les entrefers identiques)
    let sides = model.sides();
    let mut bt = vec![vec![0.0; n]; q];
    let mut by = vec![vec![0.0; n]; q];
    let angles: Vec<f64> = (0..n).map(|k| 360.0 * k as f64 / n as f64).collect();
    let phis: Vec<Vec<f64>> = crate::par::map(n, |k| {
        let f = model.solve(angles[k].to_radians());
        (0..q).map(|j| model.tooth_flux(&f, j) / sides).collect()
    });
    for (k, phi) in phis.iter().enumerate() {
        for j in 0..q {
            bt[j][k] = phi[j] / (w_t * l);
        }
        // conservation du flux : la culasse entre deux dents porte la somme
        // cumulée des flux de dents (valeur moyenne retirée)
        let mut cum = vec![0.0; q];
        let mut acc = 0.0;
        for j in 0..q {
            acc += phi[j];
            cum[j] = acc;
        }
        let mean = cum.iter().sum::<f64>() / q as f64;
        for j in 0..q {
            by[j][k] = (cum[j] - mean) / (w_y * l);
        }
    }
    IronSweep { angles_deg: angles, b_tooth: bt, b_yoke: by }
}

/// Coefficients de pertes fer pré-intégrés sur la forme d'onde réelle.
#[derive(Clone, Debug, Default)]
pub struct IronCoeffs {
    /// Hystérésis : P = h · (n/60) [W par tr/s].
    pub hyst_w_per_rps: f64,
    /// Foucault : P = e · ω² [W/(rad/s)²].
    pub eddy_w_per_w2: f64,
    /// Induction crête maximale dans les dents [T].
    pub b_tooth_peak: f64,
    /// Induction crête maximale dans la culasse [T].
    pub b_yoke_peak: f64,
}

/// Intègre le modèle de Bertotti sur les formes d'onde d'induction.
pub fn iron_coeffs(sw: &IronSweep, m: &Masses, spec: &MotorSpec, p: &DynParams) -> IronCoeffs {
    let n = sw.angles_deg.len().max(2);
    let dth = 2.0 * PI / n as f64;
    let pole_pairs = (spec.n_poles / 2) as f64;
    // coefficient de Foucault classique kc = π² σ d² / (6 ρ)  [W/(kg·Hz²·T²)]
    let d = p.lam_thick_mm * 1e-3;
    let kc = PI * PI * p.sigma_iron * d * d / (6.0 * RHO_STEEL);
    let mut h = 0.0;
    let mut e = 0.0;
    let (mut bt_pk, mut by_pk) = (0.0f64, 0.0f64);
    for (set, mass, pk) in [(&sw.b_tooth, m.tooth, &mut bt_pk), (&sw.b_yoke, m.yoke_seg, &mut by_pk)] {
        for b in set.iter() {
            let bmax = b.iter().fold(0.0f64, |a, v| a.max(v.abs()));
            *pk = pk.max(bmax);
            // hystérésis : p cycles par tour mécanique, induction crête bmax
            h += p.kh * pole_pairs * bmax * bmax * mass;
            // Foucault : P = kc/(2π²) · <(dB/dt)²>, dB/dt = ω · dB/dθ
            let mut d2 = 0.0;
            for k in 0..n {
                let db = (b[(k + 1) % n] - b[(k + n - 1) % n]) / (2.0 * dth);
                d2 += db * db;
            }
            e += kc / (2.0 * PI * PI) * (d2 / n as f64) * mass;
        }
    }
    IronCoeffs { hyst_w_per_rps: h, eddy_w_per_w2: e, b_tooth_peak: bt_pk, b_yoke_peak: by_pk }
}

/// Raie d'excitation acoustique.
#[derive(Clone, Debug)]
pub struct Tone {
    /// Fréquence [Hz].
    pub freq_hz: f64,
    /// Amplitude de l'excitation (N·m pour le couple, N pour la force).
    pub amplitude: f64,
    /// Niveau relatif pondéré A [dB] (0 dB = raie la plus audible).
    pub level_a_db: f64,
    /// Origine : « couple » ou « force ».
    pub source: &'static str,
}

/// État du moteur à une vitesse donnée.
#[derive(Clone, Debug, Default)]
pub struct DynState {
    /// Vitesse [tr/min].
    pub rpm: f64,
    /// Fréquence électrique [Hz].
    pub f_elec_hz: f64,
    /// Pertes fer par hystérésis [W].
    pub p_hyst: f64,
    /// Pertes fer par courants de Foucault [W].
    pub p_eddy: f64,
    /// Pertes des roulements [W].
    pub p_bearing: f64,
    /// Pertes par ventilation [W].
    pub p_windage: f64,
    /// Pertes totales à vide [W].
    pub p_total: f64,
    /// Couple résistant total [N·m].
    pub drag_torque: f64,
    /// Charge radiale moyenne sur les roulements [N].
    pub bearing_load: f64,
    /// Force centrifuge due au balourd résiduel [N].
    pub unbalance_force: f64,
    /// Coefficient de convection [W/(m²·K)].
    pub h_conv: f64,
    /// Température en régime établi [°C].
    pub t_steady: f64,
    /// Constante de temps thermique [s].
    pub tau_s: f64,
    /// Rapport Br(T)/Br(20 °C).
    pub br_ratio: f64,
    /// Raies d'excitation acoustique, triées par niveau pondéré A.
    pub tones: Vec<Tone>,
    /// Indice global d'excitation pondéré A [dB rel. 1 mN·m].
    pub la_index_db: f64,
}

/// Pondération A (CEI 61672) [dB].
pub fn a_weight_db(f: f64) -> f64 {
    let f2 = f * f;
    let ra = 12194.0f64.powi(2) * f2 * f2 / ((f2 + 20.6f64.powi(2)) * ((f2 + 107.7f64.powi(2)) * (f2 + 737.9f64.powi(2))).sqrt() * (f2 + 12194.0f64.powi(2)));
    20.0 * ra.max(1e-30).log10() + 2.0
}

/// Évalue toutes les grandeurs à la vitesse `rpm` (régime établi thermique).
pub fn evaluate(res: &Results, iron: &IronCoeffs, m: &Masses, p: &DynParams, rpm: f64) -> DynState {
    let s = &res.spec;
    let w = rpm.abs() * 2.0 * PI / 60.0;
    let rps = rpm.abs() / 60.0;
    let rg = s.rings();
    let r_o = if s.topology == crate::geometry::Topology::Axial { s.axial.r_out * 1e-3 } else { rg.rotor_yoke.0.max(rg.rotor_yoke.1) };
    let l = if s.topology == crate::geometry::Topology::Axial { s.rotor_yoke_th * 1e-3 } else { s.length() };
    // charge des roulements : poids + force magnétique moyenne + balourd
    let u_kgm = p.residual_unbalance_mg_cm * 1e-8;
    let f_unb = u_kgm * w * w;
    let f_mag = res.forces.mean_n;
    let load = m.rotor * 9.81 + f_mag + f_unb;
    // Palmgren : M = M0 + M1 par roulement (2 roulements), en N·mm
    let dm = p.bearing_dm_mm;
    let vn = p.bearing_visc_cst * rpm.abs();
    let m0 = if vn >= 2000.0 { 1e-7 * p.bearing_f0 * vn.powf(2.0 / 3.0) * dm.powi(3) } else { 160e-7 * p.bearing_f0 * dm.powi(3) };
    let m1 = p.bearing_mu * (load / 2.0) * dm / 2.0;
    let t_bearing = 2.0 * (m0 + m1) * 1e-3;
    // ventilation : cylindre extérieur + disque du fond de cloche
    let re_c = RHO_AIR * w * r_o * r_o / MU_AIR;
    // laminaire : couple de Rayleigh 4π μ ω r² L ; turbulent : Cf = 0,0325 Re^−0,2
    let t_lam = 4.0 * PI * MU_AIR * w * r_o * r_o * l;
    let t_turb = 0.0325 * re_c.max(1.0).powf(-0.2) * PI * RHO_AIR * w * w * r_o.powi(4) * l;
    let t_cyl = t_lam.max(t_turb);
    let cm = if re_c < 3e5 { 3.87 / re_c.max(1.0).sqrt() } else { 0.146 * re_c.powf(-0.2) };
    let t_disc = 0.5 * cm * RHO_AIR * w * w * r_o.powi(5);
    let t_wind = t_cyl + t_disc;
    // pertes fer (dépendent de Br(T) : B ∝ Br) — itération thermique
    let area = 2.0 * PI * r_o * l + PI * r_o * r_o;
    let d = 2.0 * r_o;
    let re_d = w * d * d / (MU_AIR / RHO_AIR) / 2.0;
    let nu = 0.133 * re_d.max(0.0).powf(2.0 / 3.0) * PR_AIR.powf(1.0 / 3.0);
    let h_conv = p.h_natural + nu * K_AIR / d;
    let cap = m.stator * CP_STEEL + m.magnets * CP_MAGNET + m.rotor_yoke * CP_STEEL;
    let mut temp = p.t_ambient;
    let (mut ph, mut pe, mut ratio) = (0.0, 0.0, 1.0);
    for _ in 0..30 {
        ratio = (1.0 + p.alpha_br_pct / 100.0 * (temp - 20.0)).max(0.0);
        ph = iron.hyst_w_per_rps * rps * ratio * ratio;
        pe = iron.eddy_w_per_w2 * w * w * ratio * ratio;
        let total = ph + pe + (t_bearing + t_wind) * w;
        let t_new = p.t_ambient + total / (h_conv * area);
        if (t_new - temp).abs() < 1e-6 {
            temp = t_new;
            break;
        }
        temp = 0.5 * (temp + t_new);
    }
    let p_b = t_bearing * w;
    let p_w = t_wind * w;
    // raies acoustiques : couple de détente (spectre sur la fenêtre) et force
    let mut tones = Vec::new();
    let span = res.cog.metrics.span_deg;
    for (k, a) in res.cog.spectrum.iter().enumerate().skip(1) {
        let order = k as f64 * 360.0 / span;
        tones.push(Tone { freq_hz: order * rps, amplitude: a * ratio * ratio, level_a_db: 0.0, source: "couple" });
    }
    let fmag: Vec<f64> = res.forces.fx.iter().zip(&res.forces.fy).map(|(x, y)| x.hypot(*y)).collect();
    let fs = crate::analysis::spectrum(&fmag);
    for (k, a) in fs.iter().enumerate().skip(1) {
        tones.push(Tone { freq_hz: k as f64 * rps, amplitude: a * ratio * ratio, level_a_db: 0.0, source: "force" });
    }
    tones.retain(|t| t.amplitude > 0.0 && t.freq_hz > 0.0);
    // niveau pondéré A relatif ; référence 1 mN·m (couple) ou 1 mN (force)
    for t in tones.iter_mut() {
        t.level_a_db = 20.0 * (t.amplitude / 1e-3).max(1e-30).log10() + a_weight_db(t.freq_hz);
    }
    tones.sort_by(|a, b| b.level_a_db.partial_cmp(&a.level_a_db).unwrap());
    let la = 10.0 * tones.iter().filter(|t| t.freq_hz >= 20.0 && t.freq_hz <= 20000.0).map(|t| 10f64.powf(t.level_a_db / 10.0)).sum::<f64>().max(1e-30).log10();
    tones.truncate(8);
    DynState {
        rpm,
        f_elec_hz: rps * (s.n_poles / 2) as f64,
        p_hyst: ph,
        p_eddy: pe,
        p_bearing: p_b,
        p_windage: p_w,
        p_total: ph + pe + p_b + p_w,
        drag_torque: t_bearing + t_wind + if w > 0.0 { (ph + pe) / w } else { 0.0 },
        bearing_load: load,
        unbalance_force: f_unb,
        h_conv,
        t_steady: temp,
        tau_s: cap / (h_conv * area),
        br_ratio: ratio,
        tones,
        la_index_db: la,
    }
}

/// Vibration du rotor à une vitesse donnée.
///
/// Rotor rigide (masse m) sur roulements de raideur k et d'amortissement
/// visqueux ζ, excité par la force magnétique tournante (série de Fourier de
/// Fx + i·Fy sur un tour, ordres positifs et négatifs) et par le balourd
/// résiduel. Torsion : rotor libre d'inertie J (+ charge), excité par le
/// couple de détente.
#[derive(Clone, Debug, Default)]
pub struct Vibration {
    /// Ordres mécaniques k et déplacements complexes X_k [m].
    pub modes: Vec<(i32, f64, f64)>,
    /// Déplacement crête de l'orbite [µm].
    pub disp_peak_um: f64,
    /// Vitesse vibratoire efficace par axe [mm/s] (critère ISO 10816).
    pub vel_rms_mm_s: f64,
    /// Accélération crête [g].
    pub acc_peak_g: f64,
    /// Zone ISO 10816-1 classe I : 'A', 'B', 'C' ou 'D'.
    pub iso_zone: char,
    /// Vitesse critique radiale [tr/min].
    pub critical_rpm: f64,
    /// Ondulation de vitesse crête-à-crête due au couple de détente [tr/min].
    pub speed_ripple_rpm: f64,
    /// Accélération angulaire crête [rad/s²].
    pub ang_acc_peak: f64,
    /// Harmoniques de force les plus fortes : (ordre k, amplitude [N],
    /// vitesse à laquelle cet ordre excite la résonance des roulements [tr/min]).
    pub resonances: Vec<(i32, f64, f64)>,
}

impl Vibration {
    /// Position (x, y) du centre du rotor [m] pour la position θ [°].
    pub fn disp_at(&self, theta_deg: f64) -> (f64, f64) {
        let th = theta_deg.to_radians();
        let (mut x, mut y) = (0.0, 0.0);
        for &(k, re, im) in &self.modes {
            let (sn, cs) = (k as f64 * th).sin_cos();
            x += re * cs - im * sn;
            y += re * sn + im * cs;
        }
        (x, y)
    }
}

/// Calcule la vibration radiale et de torsion à `rpm`, avec le facteur
/// `br_ratio` = Br(T)/Br(20 °C) (forces et couples ∝ Br²).
pub fn vibration(res: &Results, m: &Masses, p: &DynParams, rpm: f64, br_ratio: f64) -> Vibration {
    let w = rpm.abs() * 2.0 * PI / 60.0;
    let kb = p.bearing_stiffness_n_per_um.max(1e-6) * 1e6;
    let mass = m.rotor.max(1e-6);
    let c = 2.0 * p.damping_ratio * (kb * mass).sqrt();
    let n = res.forces.fx.len();
    let g2 = br_ratio * br_ratio;
    // série de Fourier complexe de F = Fx + i·Fy sur un tour : F = Σ C_k e^{ikθ}
    let mut modes = Vec::new();
    let mut amps: Vec<(i32, f64)> = Vec::new();
    let kmax = (n / 2) as i32 - 1;
    for k in -kmax..=kmax {
        let (mut re, mut im) = (0.0, 0.0);
        for j in 0..n {
            let a = -(k as f64) * 2.0 * PI * j as f64 / n as f64;
            let (sa, ca) = a.sin_cos();
            let (fx, fy) = (res.forces.fx[j] * g2, res.forces.fy[j] * g2);
            re += fx * ca - fy * sa;
            im += fx * sa + fy * ca;
        }
        re /= n as f64;
        im /= n as f64;
        if k == 1 {
            // balourd : force centrifuge tournant à 1×, U·ω²
            re += p.residual_unbalance_mg_cm * 1e-8 * w * w;
        }
        if re.hypot(im) < 1e-12 {
            continue;
        }
        if k != 0 {
            amps.push((k, re.hypot(im)));
        }
        // X_k = C_k / (k_s − m (kω)² + i c kω)
        let wk = k as f64 * w;
        let (dr, di) = (kb - mass * wk * wk, c * wk);
        let den = dr * dr + di * di;
        modes.push((k, (re * dr + im * di) / den, (im * dr - re * di) / den));
    }
    let v2: f64 = modes.iter().map(|&(k, re, im)| (k as f64 * w).powi(2) * (re * re + im * im)).sum();
    let vel_rms = (v2 / 2.0).sqrt(); // par axe
    let mut vib = Vibration { modes, ..Default::default() };
    let (mut dpk, mut apk) = (0.0f64, 0.0f64);
    for j in 0..360 {
        let (x, y) = vib.disp_at(j as f64);
        dpk = dpk.max(x.hypot(y));
        let th = (j as f64).to_radians();
        let (mut ax, mut ay) = (0.0, 0.0);
        for &(k, re, im) in &vib.modes {
            let f = -(k as f64 * w).powi(2);
            let (sn, cs) = (k as f64 * th).sin_cos();
            ax += f * (re * cs - im * sn);
            ay += f * (re * sn + im * cs);
        }
        apk = apk.max(ax.hypot(ay));
    }
    vib.disp_peak_um = dpk * 1e6;
    vib.vel_rms_mm_s = vel_rms * 1e3;
    vib.acc_peak_g = apk / 9.81;
    vib.iso_zone = match vib.vel_rms_mm_s {
        v if v <= 0.71 => 'A',
        v if v <= 1.8 => 'B',
        v if v <= 4.5 => 'C',
        _ => 'D',
    };
    vib.critical_rpm = (kb / mass).sqrt() * 60.0 / (2.0 * PI);
    amps.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    vib.resonances = amps.iter().take(4).map(|&(k, a)| (k, a, vib.critical_rpm / (k.unsigned_abs() as f64))).collect();
    // torsion : ondulation de vitesse Δω = ∫ T/J dt, sur la fenêtre de cogging
    let j_tot = (m.inertia + p.load_inertia).max(1e-12);
    if w > 0.0 {
        let tq = &res.cog.torque_nm;
        let nn = tq.len().max(1);
        let dt = res.cog.metrics.span_deg.to_radians() / nn as f64 / w;
        let mean = tq.iter().sum::<f64>() / nn as f64;
        let (mut om, mut lo, mut hi) = (0.0f64, 0.0f64, 0.0f64);
        for v in tq {
            om += (v - mean) * g2 / j_tot * dt;
            lo = lo.min(om);
            hi = hi.max(om);
        }
        vib.speed_ripple_rpm = (hi - lo) * 60.0 / (2.0 * PI);
        vib.ang_acc_peak = tq.iter().fold(0.0f64, |a, v| a.max((v - mean).abs())) * g2 / j_tot;
    }
    vib
}

/// Interpolation périodique d'un signal échantillonné sur `span` degrés.
pub fn interp_periodic(angles: &[f64], v: &[f64], span: f64, theta_deg: f64) -> f64 {
    let n = v.len();
    if n == 0 {
        return 0.0;
    }
    let x = theta_deg.rem_euclid(span) / span * n as f64;
    let i = x.floor() as usize % n;
    let f = x - x.floor();
    let _ = angles;
    v[i] * (1.0 - f) + v[(i + 1) % n] * f
}

/// Signal audio (WAV mono 16 bits, 44,1 kHz) de l'excitation magnétique
/// synthétisée à `rpm` : couple de détente + force radiale, normalisés.
pub fn synth_wav(res: &Results, rpm: f64, seconds: f64) -> Vec<u8> {
    let sr = 44100.0;
    let n = (seconds * sr) as usize;
    let span = res.cog.metrics.span_deg;
    let fmag: Vec<f64> = res.forces.fx.iter().zip(&res.forces.fy).map(|(x, y)| x.hypot(*y)).collect();
    let mean_f = fmag.iter().sum::<f64>() / fmag.len().max(1) as f64;
    let deg_per_s = rpm.abs() * 6.0;
    let mut sig: Vec<f64> = (0..n)
        .map(|k| {
            let th = deg_per_s * k as f64 / sr;
            let c = interp_periodic(&res.cog.angles_deg, &res.cog.torque_nm, span, th);
            let f = interp_periodic(&res.forces.angles_deg, &fmag, 360.0, th) - mean_f;
            c * 1e3 + 0.3 * f
        })
        .collect();
    let mx = sig.iter().fold(0.0f64, |a, v| a.max(v.abs())).max(1e-12);
    // fondu d'entrée/sortie de 20 ms
    let fade = (0.02 * sr) as usize;
    for k in 0..n {
        let g = (k.min(n - 1 - k) as f64 / fade as f64).min(1.0);
        sig[k] = sig[k] / mx * 0.8 * g;
    }
    let mut out = Vec::with_capacity(44 + 2 * n);
    let data_len = (2 * n) as u32;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&44100u32.to_le_bytes());
    out.extend_from_slice(&(44100u32 * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for v in sig {
        out.extend_from_slice(&((v * 32767.0) as i16).to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::{self, Msg, Settings};

    fn res(v: &str) -> Results {
        let job = compute::run_main(MotorSpec::example(v), Settings { n_cog: 240, n_emf: 360, ..Default::default() });
        match job.rx.recv().unwrap() {
            Msg::Main(Ok(r)) => r,
            _ => panic!(),
        }
    }

    #[test]
    fn ponderation_a_normalisee() {
        assert!(a_weight_db(1000.0).abs() < 0.1);
        assert!((a_weight_db(100.0) + 19.1).abs() < 0.3);
    }

    #[test]
    fn pertes_croissent_avec_la_vitesse() {
        let r = res("regulier");
        let m = masses(&r.spec);
        let sw = iron_sweep(&r.model, 180);
        let p = DynParams::default();
        let ic = iron_coeffs(&sw, &m, &r.spec, &p);
        assert!(ic.b_tooth_peak > 0.5 && ic.b_tooth_peak < 3.0, "B dent {}", ic.b_tooth_peak);
        let a = evaluate(&r, &ic, &m, &p, 5000.0);
        let b = evaluate(&r, &ic, &m, &p, 20000.0);
        assert!(b.p_total > a.p_total && b.t_steady > a.t_steady);
        // hystérésis ∝ n, Foucault ∝ n² (à température égale, ratio proche)
        assert!(b.p_eddy / a.p_eddy > 10.0);
        assert!(b.br_ratio < 1.0);
        // masse plausible pour un petit moteur de drone (rotor ≈ 10 à 25 g)
        assert!(m.rotor > 0.005 && m.rotor < 0.04, "rotor {} kg", m.rotor);
    }

    #[test]
    fn vibration_coherente() {
        let p = DynParams::default();
        // rotor régulier : force magnétique nulle par symétrie → seule reste la réponse au balourd (1×)
        let r = res("regulier");
        let m = masses(&r.spec);
        let v = vibration(&r, &m, &p, 20000.0, 1.0);
        let w = 20000.0 * 2.0 * PI / 60.0;
        let kb = p.bearing_stiffness_n_per_um * 1e6;
        let attendu = p.residual_unbalance_mg_cm * 1e-8 * w * w / (kb - m.rotor * w * w).abs();
        assert!((v.disp_peak_um * 1e-6 / attendu - 1.0).abs() < 0.05, "{} vs {}", v.disp_peak_um, attendu * 1e6);
        assert!(v.critical_rpm > 50000.0);
        // rotor irrégulier : la force magnétique tournante domine, vibration plus forte
        let r2 = res("irregulier");
        let v2 = vibration(&r2, &m, &p, 20000.0, 1.0);
        assert!(v2.vel_rms_mm_s > v.vel_rms_mm_s);
        // inertie de charge : l'ondulation de vitesse diminue
        let mut p2 = p.clone();
        p2.load_inertia = 1e-5;
        assert!(vibration(&r2, &m, &p2, 20000.0, 1.0).speed_ripple_rpm < v2.speed_ripple_rpm);
    }

    #[test]
    fn wav_valide() {
        let r = res("irregulier");
        let w = synth_wav(&r, 10000.0, 0.2);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(w.len(), 44 + 2 * 8820);
    }
}
