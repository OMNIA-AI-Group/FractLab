//! Simulation DYNAMIQUE temporelle de la machine dans son environnement :
//! alimentation (réseau, onduleur, commutation 120°, redresseur à diodes),
//! charge mécanique (constante, quadratique, convoyeur, éolienne) et défauts
//! (phase ouverte, court-circuit entre spires).
//!
//! Modèle électrique en phases, étoile à neutre isolé, inductance synchrone
//! constante L_s = L − M et résistance R issues de l'analyse en charge ;
//! flux d'aimants λ_k(θ) et couple de détente issus du modèle de champ.
//! Équations : L_s di_k/dt = v_k − v_n − R i_k − ω dλ_k/dθ, avec v_n tel que
//! Σ i_k = 0 ; couple Te = Σ i_k dλ_k/dθ + T_détente(θ) ;
//! J dω/dt = Te + T_entraînement − T_charge − T_frottements. Intégration RK4.

use crate::compute::Results;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Alimentation électrique.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Supply {
    /// Réseau triphasé sinusoïdal : tension entre phases efficace [V], fréquence [Hz].
    Grid {
        /// Tension composée efficace [V].
        v_ll: f64,
        /// Fréquence [Hz].
        f_hz: f64,
    },
    /// Onduleur MLI sinusoïdal autopiloté (synchronisé sur la position du rotor).
    InverterSine {
        /// Tension du bus continu [V].
        vdc: f64,
        /// Indice de modulation (0 … 1).
        m: f64,
        /// Avance angulaire [° électriques].
        advance_deg: f64,
    },
    /// Commutation 120° (contrôleur de moteur de drone / BLDC).
    SixStep {
        /// Tension du bus continu [V].
        vdc: f64,
        /// Rapport cyclique (0 … 1).
        duty: f64,
        /// Avance angulaire [° électriques].
        advance_deg: f64,
    },
    /// Génératrice débitant sur un redresseur à diodes et un bus continu.
    DiodeRectifier {
        /// Capacité du bus [F].
        c_bus: f64,
        /// Résistance de charge [Ω].
        r_load: f64,
    },
}

/// Charge mécanique.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Load {
    /// Aucune (rotor libre).
    Free,
    /// Couple résistant constant [N·m].
    Constant(f64),
    /// Ventilateur / pompe / hélice : T = k·ω² [N·m/(rad/s)²].
    Quadratic(f64),
    /// Convoyeur : couple constant + frottement visqueux.
    Conveyor {
        /// Couple constant [N·m].
        t0: f64,
        /// Coefficient visqueux [N·m·s/rad].
        visc: f64,
    },
    /// Éolienne entraînant la machine (génératrice).
    WindTurbine {
        /// Rayon des pales [m].
        radius: f64,
        /// Vitesse du vent [m/s].
        wind: f64,
    },
}

/// Défaut simulé.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Fault {
    /// Machine saine.
    None,
    /// Phase ouverte (0 = A, 1 = B, 2 = C).
    OpenPhase(usize),
    /// Court-circuit entre spires de la phase A.
    InterTurn {
        /// Fraction de spires en court-circuit (0 … 1).
        mu: f64,
        /// Résistance du défaut [Ω].
        r_f: f64,
    },
}

/// Paramètres de simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimSetup {
    /// Alimentation.
    pub supply: Supply,
    /// Charge.
    pub load: Load,
    /// Défaut.
    pub fault: Fault,
    /// Durée simulée [s].
    pub duration: f64,
    /// Vitesse initiale [tr/min].
    pub rpm0: f64,
    /// Instant d'apparition du défaut [s].
    pub fault_t: f64,
}

impl Default for SimSetup {
    fn default() -> Self {
        SimSetup {
            supply: Supply::SixStep { vdc: 16.0, duty: 0.6, advance_deg: 0.0 },
            load: Load::Quadratic(2e-8),
            fault: Fault::None,
            duration: 0.5,
            rpm0: 0.0,
            fault_t: 0.25,
        }
    }
}

/// Résultats échantillonnés.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SimResult {
    /// Temps [s].
    pub t: Vec<f64>,
    /// Vitesse [tr/min].
    pub rpm: Vec<f64>,
    /// Couple électromagnétique [N·m].
    pub te: Vec<f64>,
    /// Courants de phase [A].
    pub i: [Vec<f64>; 3],
    /// Tension de la phase A [V].
    pub va: Vec<f64>,
    /// Tension du bus continu [V] (redresseur).
    pub vdc: Vec<f64>,
    /// Courant de défaut [A].
    pub i_fault: Vec<f64>,
    /// Puissance électrique absorbée (> 0 moteur) [W].
    pub p_elec: Vec<f64>,
    /// Pas de calcul [s].
    pub dt: f64,
}

struct Plant {
    p: f64,
    r: f64,
    ls: f64,
    j: f64,
    flux: [Vec<f64>; 3],
    cog: Vec<f64>,
    cog_span: f64,
    phi: [f64; 3],
    fric: Vec<(f64, f64)>,
}

impl Plant {
    fn dflux(&self, k: usize, th: f64) -> f64 {
        let n = self.flux[k].len();
        let x = th.rem_euclid(2.0 * PI) / (2.0 * PI) * n as f64;
        let i = x.floor() as usize % n;
        let f = x - x.floor();
        let d = |j: usize| (self.flux[k][(j + 1) % n] - self.flux[k][(j + n - 1) % n]) / (2.0 * 2.0 * PI / n as f64);
        d(i) * (1.0 - f) + d((i + 1) % n) * f
    }
    fn cogging(&self, th: f64) -> f64 {
        let n = self.cog.len();
        let x = th.to_degrees().rem_euclid(self.cog_span) / self.cog_span * n as f64;
        let i = x.floor() as usize % n;
        let f = x - x.floor();
        self.cog[i] * (1.0 - f) + self.cog[(i + 1) % n] * f
    }
    fn friction(&self, w: f64) -> f64 {
        let rpm = w.abs() * 30.0 / PI;
        let t = match self.fric.iter().position(|(r, _)| *r >= rpm) {
            Some(0) => self.fric[0].1 * rpm / self.fric[0].0.max(1.0),
            Some(k) => {
                let (a, b) = (self.fric[k - 1], self.fric[k]);
                a.1 + (b.1 - a.1) * (rpm - a.0) / (b.0 - a.0)
            }
            None => self.fric.last().map(|x| x.1).unwrap_or(0.0),
        };
        t * w.signum()
    }
}

/// Simule ; `x` = [iA, iB, iC, θ, ω, Vdc, i_f].
pub fn simulate(res: &Results, st: &SimSetup) -> SimResult {
    let s = &res.spec;
    let pp = (s.n_poles / 2) as f64;
    let p_dyn = &s.dynamics;
    let m = crate::dynamics::masses(s);
    let ic = crate::dynamics::iron_coeffs(&res.iron, &m, s, p_dyn);
    let fric: Vec<(f64, f64)> = (0..30)
        .map(|k| {
            let rpm = 50.0 * (1200.0f64).powf(k as f64 / 29.0);
            let e = crate::dynamics::evaluate(res, &ic, &m, p_dyn, rpm);
            (rpm, e.drag_torque)
        })
        .collect();
    // phases électriques de −e (comme l'analyse en charge)
    let phase_of = |e: &Vec<f64>| {
        let (mut re, mut im) = (0.0, 0.0);
        for (k, v) in e.iter().enumerate() {
            let a = pp * 2.0 * PI * k as f64 / e.len() as f64;
            re += -v * a.cos();
            im += -v * a.sin();
        }
        im.atan2(re)
    };
    let plant = Plant {
        p: pp,
        r: res.load.r_phase.max(1e-6),
        ls: res.load.ls.max(1e-9),
        j: (m.inertia + p_dyn.load_inertia).max(1e-9),
        flux: res.emf.flux.clone(),
        cog: res.cog.torque_nm.clone(),
        cog_span: res.cog.metrics.span_deg,
        phi: [phase_of(&res.emf.emf[0]), phase_of(&res.emf.emf[1]), phase_of(&res.emf.emf[2])],
        fric,
    };
    // pas : 1/200 de la période électrique attendue (bornée), ou constante L/R
    let w_guess = match st.supply {
        Supply::Grid { f_hz, .. } => 2.0 * PI * f_hz / pp,
        _ => (st.rpm0.max(3000.0)) * PI / 30.0 * 4.0,
    };
    let dt = (2.0 * PI / (pp * w_guess.max(1.0)) / 200.0).min(plant.ls / plant.r / 20.0).clamp(1e-7, 1e-4);
    let nsteps = ((st.duration / dt).ceil() as usize).clamp(10, 3_000_000);
    let every = (nsteps / 4000).max(1);
    let mut x = [0.0, 0.0, 0.0, 0.0, st.rpm0 * PI / 30.0, 0.0, 0.0];
    let mut out = SimResult { dt, ..Default::default() };
    let deriv = |t: f64, x: &[f64; 7]| -> ([f64; 7], f64, f64, f64) {
        let th = x[3];
        let w = x[4];
        let fault_on = t >= st.fault_t;
        let dl: [f64; 3] = core::array::from_fn(|k| plant.dflux(k, th));
        let e: [f64; 3] = core::array::from_fn(|k| w * dl[k]);
        let i = [x[0], x[1], x[2]];
        // tensions de phase imposées (par rapport au milieu du bus)
        let v: [f64; 3] = match st.supply {
            Supply::Grid { v_ll, f_hz } => {
                let vp = v_ll / 3f64.sqrt() * 2f64.sqrt();
                core::array::from_fn(|k| vp * (2.0 * PI * f_hz * t - 2.0 * PI * k as f64 / 3.0).cos())
            }
            Supply::InverterSine { vdc, m, advance_deg } => {
                core::array::from_fn(|k| m.clamp(0.0, 1.15) * vdc / 2.0 * (plant.p * th - plant.phi[k] + advance_deg.to_radians()).cos())
            }
            Supply::SixStep { vdc, duty, advance_deg } => {
                // la phase dont l'angle est dans ±60° de la crête est à +, celle à ±180° est à −, la troisième flotte
                core::array::from_fn(|k| {
                    let a = (plant.p * th - plant.phi[k] + advance_deg.to_radians()).rem_euclid(2.0 * PI);
                    let a = if a > PI { a - 2.0 * PI } else { a };
                    if a.abs() < PI / 3.0 {
                        duty * vdc / 2.0
                    } else if a.abs() > 2.0 * PI / 3.0 {
                        -duty * vdc / 2.0
                    } else {
                        0.0
                    }
                })
            }
            Supply::DiodeRectifier { .. } => core::array::from_fn(|k| -x[5] / 2.0 * (i[k] / 0.05).tanh()),
        };
        let mut di = [0.0; 3];
        let open = match st.fault {
            Fault::OpenPhase(k) if fault_on => Some(k.min(2)),
            _ => None,
        };
        match open {
            Some(k) => {
                let (a, b) = ((k + 1) % 3, (k + 2) % 3);
                let d = ((v[a] - v[b]) - 2.0 * plant.r * i[a] - (e[a] - e[b])) / (2.0 * plant.ls);
                di[a] = d;
                di[b] = -d;
                di[k] = -i[k] / 1e-5; // courant forcé à zéro
            }
            None => {
                let vn = (0..3).map(|k| v[k] - e[k] - plant.r * i[k]).sum::<f64>() / 3.0;
                for k in 0..3 {
                    di[k] = (v[k] - vn - plant.r * i[k] - e[k]) / plant.ls;
                }
            }
        }
        // court-circuit entre spires (modèle simplifié)
        let (mut dif, mut tf) = (0.0, 0.0);
        if let Fault::InterTurn { mu, r_f } = st.fault
            && fault_on {
                let lf = (mu * mu * plant.ls).max(1e-9);
                dif = (mu * e[0] - (mu * plant.r + r_f) * x[6]) / lf;
                tf = -mu * dl[0] * x[6];
            }
        let te = (0..3).map(|k| i[k] * dl[k]).sum::<f64>() + plant.cogging(th) + tf;
        let t_load = match st.load {
            Load::Free => 0.0,
            Load::Constant(c) => c * w.signum(),
            Load::Quadratic(k) => k * w * w.abs(),
            Load::Conveyor { t0, visc } => t0 * w.signum() + visc * w,
            Load::WindTurbine { radius, wind } => {
                let lam = (w.abs() * radius / wind.max(0.1)).max(0.1);
                let inv = 1.0 / lam - 0.035;
                let cp = (0.5176 * (116.0 * inv - 5.0) * (-21.0 * inv).exp() + 0.0068 * lam).max(0.0);
                -(0.5 * 1.225 * PI * radius * radius * wind.powi(3) * cp / w.abs().max(0.5))
            }
        };
        let dw = (te - t_load - plant.friction(w)) / plant.j;
        let dvdc = match st.supply {
            Supply::DiodeRectifier { c_bus, r_load } => ((i[0].abs() + i[1].abs() + i[2].abs()) / 2.0 - x[5] / r_load.max(1e-3)) / c_bus.max(1e-9),
            _ => 0.0,
        };
        let p_el: f64 = (0..3).map(|k| v[k] * i[k]).sum();
        ([di[0], di[1], di[2], w, dw, dvdc, dif], te, v[0], p_el)
    };
    for n in 0..nsteps {
        let t = n as f64 * dt;
        let (k1, te, va, pe) = deriv(t, &x);
        let add = |a: &[f64; 7], b: &[f64; 7], h: f64| -> [f64; 7] { core::array::from_fn(|j| a[j] + h * b[j]) };
        let (k2, ..) = deriv(t + dt / 2.0, &add(&x, &k1, dt / 2.0));
        let (k3, ..) = deriv(t + dt / 2.0, &add(&x, &k2, dt / 2.0));
        let (k4, ..) = deriv(t + dt, &add(&x, &k3, dt));
        for j in 0..7 {
            x[j] += dt / 6.0 * (k1[j] + 2.0 * k2[j] + 2.0 * k3[j] + k4[j]);
        }
        if !x.iter().all(|v| v.is_finite()) {
            break;
        }
        if n % every == 0 {
            out.t.push(t);
            out.rpm.push(x[4] * 30.0 / PI);
            out.te.push(te);
            for k in 0..3 {
                out.i[k].push(x[k]);
            }
            out.va.push(va);
            out.vdc.push(x[5]);
            out.i_fault.push(x[6]);
            out.p_elec.push(pe);
        }
    }
    out
}

/// Lance la simulation en tâche de fond (fil sur ordinateur, Web Worker dans
/// le navigateur) ; renvoie le canal du résultat et le Worker éventuel.
pub fn run(r: &Results, setup: SimSetup) -> (std::sync::mpsc::Receiver<SimResult>, Option<crate::compute::WorkerHandle>) {
    let (tx, rx) = std::sync::mpsc::channel();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let res = r.light_clone();
        std::thread::spawn(move || {
            let _ = tx.send(simulate(&res, &setup));
        });
        (rx, None)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let w = crate::web::worker(crate::bg::Request::Sim(r.to_wire(), setup), move |m| {
            if let crate::bg::Reply::Sim(s) = m {
                let _ = tx.send(s);
            }
        });
        (rx, Some(w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::{self, Msg, Settings};
    use crate::geometry::MotorSpec;

    fn res() -> Results {
        let job = compute::run_main(MotorSpec::default(), Settings { n_cog: 120, n_emf: 360, ..Default::default() });
        match job.rx.recv().unwrap() {
            Msg::Main(Ok(r)) => r,
            _ => panic!(),
        }
    }

    #[test]
    fn demarrage_esc_accelere_et_se_stabilise() {
        let r = res();
        let st = SimSetup { duration: 0.6, ..Default::default() };
        let o = simulate(&r, &st);
        let n = o.rpm.len();
        assert!(o.rpm[n - 1] > 1000.0, "vitesse finale {}", o.rpm[n - 1]);
        // vitesse stabilisée : faible variation sur les 10 derniers %
        let a = o.rpm[n * 9 / 10];
        assert!((o.rpm[n - 1] - a).abs() / a < 0.05, "{a} → {}", o.rpm[n - 1]);
        // à vide (hélice), la vitesse approche Vdc·duty·KV (ordre de grandeur)
        assert!(o.rpm[n - 1] < 16.0 * 0.6 * r.report.kv_estime * 1.3);
    }

    #[test]
    fn eolienne_sur_redresseur_charge_le_bus() {
        let r = res();
        let st = SimSetup {
            supply: Supply::DiodeRectifier { c_bus: 1e-3, r_load: 10.0 },
            load: Load::WindTurbine { radius: 0.12, wind: 12.0 },
            fault: Fault::None,
            duration: 0.4,
            rpm0: 3000.0,
            fault_t: 1.0,
        };
        let o = simulate(&r, &st);
        assert!(*o.vdc.last().unwrap() > 0.5, "bus {}", o.vdc.last().unwrap());
    }

    #[test]
    fn phase_ouverte_annule_son_courant() {
        let r = res();
        let st = SimSetup { fault: Fault::OpenPhase(0), duration: 0.4, fault_t: 0.2, ..Default::default() };
        let o = simulate(&r, &st);
        let n = o.i[0].len();
        let tail = o.i[0][n * 9 / 10..].iter().fold(0.0f64, |a, v| a.max(v.abs()));
        let other = o.i[1][n * 9 / 10..].iter().fold(0.0f64, |a, v| a.max(v.abs()));
        assert!(tail < 0.05 * other.max(1e-6), "iA {tail} vs iB {other}");
    }
}
