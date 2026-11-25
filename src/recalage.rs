//! Recalage sur mesures : les paramètres incertains du modèle (Br réelle des
//! aimants, entrefer réel, longueur magnétique effective qui traduit les
//! effets 3D) sont ajustés pour que la FEM ou le couple de détente calculés
//! collent à une courbe mesurée sur le prototype.
//!
//! L'origine angulaire de la mesure est inconnue : pour chaque jeu de
//! paramètres, le décalage angulaire qui minimise l'écart est recherché
//! (balayage fin sur une période). L'écart est la valeur efficace de la
//! différence rapportée à la valeur efficace de la mesure. Les paramètres
//! sont ajustés par la méthode du simplexe (Nelder–Mead).
//!
//! Br et la longueur ne font que multiplier l'amplitude : seul leur produit
//! est identifiable, l'interface interdit donc de les ajuster ensemble.
//! L'entrefer modifie aussi la forme d'onde et se distingue de Br.

use crate::analysis;
use crate::geometry::MotorSpec;
use crate::subdomain::{Model, Resolution};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

/// Grandeur mesurée.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeasKind {
    /// FEM de phase à vide [V] en fonction de l'angle du rotor [°].
    Emf,
    /// Couple de détente [N·m] en fonction de l'angle du rotor [°].
    Cogging,
}

/// Paramètre ajustable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FitVar {
    /// Induction rémanente réelle des aimants.
    Br,
    /// Entrefer réel.
    Airgap,
    /// Longueur magnétique effective (effets 3D).
    Length,
}

/// Tous les paramètres ajustables.
pub const ALL_FIT: [FitVar; 3] = [FitVar::Br, FitVar::Airgap, FitVar::Length];

impl FitVar {
    /// Nom français (clé de traduction).
    pub fn name(self) -> &'static str {
        match self {
            FitVar::Br => "Induction rémanente Br",
            FitVar::Airgap => "Entrefer",
            FitVar::Length => "Longueur active",
        }
    }
    /// Valeur actuelle.
    pub fn get(self, s: &MotorSpec) -> f64 {
        match self {
            FitVar::Br => s.br_magnet,
            FitVar::Airgap => s.airgap,
            FitVar::Length => match s.topology {
                crate::geometry::Topology::Axial => s.axial.r_out - s.axial.r_in,
                _ => s.stack_len,
            },
        }
    }
    /// Applique une valeur.
    pub fn set(self, s: &mut MotorSpec, v: f64) {
        match self {
            FitVar::Br => s.br_magnet = v.clamp(0.1, 1.6),
            FitVar::Airgap => s.airgap = v.max(0.02),
            FitVar::Length => match s.topology {
                // flux axial : la longueur radiale active est ajustée autour du rayon moyen
                crate::geometry::Topology::Axial => {
                    let m = 0.5 * (s.axial.r_out + s.axial.r_in);
                    s.axial.r_in = m - 0.5 * v.max(0.5);
                    s.axial.r_out = m + 0.5 * v.max(0.5);
                }
                _ => s.stack_len = v.max(0.5),
            },
        }
    }
    /// Unité.
    pub fn unit(self) -> &'static str {
        match self {
            FitVar::Br => "T",
            _ => "mm",
        }
    }
}

/// Réglages du recalage.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FitSetup {
    /// Grandeur mesurée.
    pub kind: MeasKind,
    /// Points mesurés (angle du rotor [°], valeur [V ou N·m]).
    pub meas: Vec<[f64; 2]>,
    /// Vitesse de la mesure de FEM [tr/min].
    pub meas_rpm: f64,
    /// Paramètres libres.
    pub vars: Vec<FitVar>,
    /// Nombre maximal d'évaluations.
    pub budget: usize,
}

/// Résultat du recalage.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FitResult {
    /// Valeurs initiales et ajustées des paramètres (même ordre que `vars`).
    pub initial: Vec<f64>,
    /// Valeurs ajustées.
    pub fitted: Vec<f64>,
    /// Écart initial [%].
    pub err_initial: f64,
    /// Écart après recalage [%].
    pub err_fitted: f64,
    /// Courbe calculée initiale, recalée en angle sur la mesure.
    pub curve_initial: Vec<[f64; 2]>,
    /// Courbe calculée après recalage, recalée en angle.
    pub curve_fitted: Vec<[f64; 2]>,
    /// Nombre d'évaluations.
    pub evals: usize,
}

/// Messages du recalage.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum FitMsg {
    /// Avancement : (évaluations, meilleur écart [%]).
    Progress(usize, f64),
    /// Terminé.
    Done(Result<FitResult, String>),
}

/// Tâche de recalage.
pub struct FitJob {
    /// Messages.
    pub rx: Receiver<FitMsg>,
    /// Demande d'arrêt.
    pub stop: Arc<AtomicBool>,
    /// Web Worker (navigateur).
    pub worker: Option<crate::compute::WorkerHandle>,
}

impl FitJob {
    /// Arrête le recalage ; vrai si la tâche est terminée tout de suite (navigateur).
    pub fn cancel(&mut self) -> bool {
        self.stop.store(true, Ordering::Relaxed);
        self.worker.take().is_some()
    }
}

/// Courbe calculée (angle [°], valeur) sur une période et sa période [°].
fn computed(spec: &MotorSpec, kind: MeasKind, meas_rpm: f64) -> Result<(Vec<f64>, Vec<f64>, f64), String> {
    let m = Model::new(spec, Resolution { nh: 300, nk: 16 })?;
    Ok(match kind {
        MeasKind::Emf => {
            let e = analysis::emf(&m, 720);
            let k = if spec.rpm > 0.0 { meas_rpm / spec.rpm } else { 1.0 };
            (e.angles_deg.clone(), e.emf[0].iter().map(|v| v * k).collect(), 360.0)
        }
        MeasKind::Cogging => {
            let per = spec.cogging_period_deg();
            let c = analysis::cogging(&m, per, 240);
            (c.angles_deg.clone(), c.torque_nm.clone(), per)
        }
    })
}

/// Écart relatif [%] et meilleur décalage angulaire [°].
fn mismatch(xs: &[f64], ys: &[f64], per: f64, meas: &[[f64; 2]], search: f64) -> (f64, f64) {
    let rms_m = (meas.iter().map(|p| p[1] * p[1]).sum::<f64>() / meas.len() as f64).sqrt().max(1e-15);
    let err = |off: f64| (meas.iter().map(|p| (crate::dynamics::interp_periodic(xs, ys, per, p[0] + off) - p[1]).powi(2)).sum::<f64>() / meas.len() as f64).sqrt() / rms_m;
    // balayage grossier puis affinage
    let n = 720;
    let mut best = (f64::MAX, 0.0);
    for k in 0..n {
        let off = search * k as f64 / n as f64;
        let e = err(off);
        if e < best.0 {
            best = (e, off);
        }
    }
    let mut step = search / n as f64;
    for _ in 0..30 {
        for d in [-step, step] {
            let e = err(best.1 + d);
            if e < best.0 {
                best = (e, best.1 + d);
            }
        }
        step *= 0.6;
    }
    (100.0 * best.0, best.1)
}

/// Évalue un jeu de paramètres : (écart [%], courbe recalée en angle aux abscisses mesurées).
pub fn evaluate(base: &MotorSpec, setup: &FitSetup, values: &[f64]) -> Result<(f64, Vec<[f64; 2]>), String> {
    let mut s = base.clone();
    for (v, x) in setup.vars.iter().zip(values) {
        v.set(&mut s, *x);
    }
    let (xs, ys, per) = computed(&s, setup.kind, setup.meas_rpm)?;
    let search = match setup.kind {
        MeasKind::Emf => 360.0,
        MeasKind::Cogging => per,
    };
    let (e, off) = mismatch(&xs, &ys, per, &setup.meas, search);
    let curve = setup.meas.iter().map(|p| [p[0], crate::dynamics::interp_periodic(&xs, &ys, per, p[0] + off)]).collect();
    Ok((e, curve))
}

/// Recalage par le simplexe de Nelder–Mead (paramètres normalisés par leur valeur initiale).
pub fn fit(base: &MotorSpec, setup: &FitSetup, stop: &AtomicBool, mut progress: impl FnMut(usize, f64)) -> Result<FitResult, String> {
    if setup.meas.len() < 4 {
        return Err("il faut au moins 4 points mesurés".into());
    }
    let x0: Vec<f64> = setup.vars.iter().map(|v| v.get(base)).collect();
    let (e0, c0) = evaluate(base, setup, &x0)?;
    let d = x0.len();
    let mut evals = 1;
    let mut best = (e0, x0.clone(), c0.clone());
    progress(evals, e0);
    if d > 0 {
        let to_x = |u: &[f64]| -> Vec<f64> { u.iter().zip(&x0).map(|(a, b)| a * b).collect() };
        let f = |u: &[f64], evals: &mut usize, best: &mut (f64, Vec<f64>, Vec<[f64; 2]>)| -> f64 {
            *evals += 1;
            match evaluate(base, setup, &to_x(u)) {
                Ok((e, c)) => {
                    if e < best.0 {
                        *best = (e, to_x(u), c);
                    }
                    e
                }
                Err(_) => 1e9,
            }
        };
        let mut simplex: Vec<Vec<f64>> = vec![vec![1.0; d]];
        for j in 0..d {
            let mut p = vec![1.0; d];
            p[j] = 0.9;
            simplex.push(p);
        }
        let mut fv: Vec<f64> = simplex.iter().map(|p| f(p, &mut evals, &mut best)).collect();
        while evals < setup.budget && !stop.load(Ordering::Relaxed) {
            let mut idx: Vec<usize> = (0..=d).collect();
            idx.sort_by(|a, b| fv[*a].total_cmp(&fv[*b]));
            simplex = idx.iter().map(|&i| simplex[i].clone()).collect();
            fv = idx.iter().map(|&i| fv[i]).collect();
            progress(evals, best.0);
            if (fv[d] - fv[0]).abs() < 1e-4 {
                break;
            }
            let cen: Vec<f64> = (0..d).map(|j| simplex[..d].iter().map(|p| p[j]).sum::<f64>() / d as f64).collect();
            let along = |t: f64| -> Vec<f64> { cen.iter().zip(&simplex[d]).map(|(c, w)| c + t * (w - c)).collect() };
            let xr = along(-1.0);
            let fr = f(&xr, &mut evals, &mut best);
            if fr < fv[0] {
                let xe = along(-2.0);
                let fe = f(&xe, &mut evals, &mut best);
                if fe < fr {
                    simplex[d] = xe;
                    fv[d] = fe;
                } else {
                    simplex[d] = xr;
                    fv[d] = fr;
                }
            } else if fr < fv[d - 1] {
                simplex[d] = xr;
                fv[d] = fr;
            } else {
                let xc = along(0.5);
                let fc = f(&xc, &mut evals, &mut best);
                if fc < fv[d] {
                    simplex[d] = xc;
                    fv[d] = fc;
                } else {
                    for i in 1..=d {
                        simplex[i] = simplex[0].iter().zip(&simplex[i]).map(|(a, b)| a + 0.5 * (b - a)).collect();
                        fv[i] = f(&simplex[i].clone(), &mut evals, &mut best);
                    }
                }
            }
        }
    }
    Ok(FitResult { initial: x0, fitted: best.1, err_initial: e0, err_fitted: best.0, curve_initial: c0, curve_fitted: best.2, evals })
}

/// Exécute le recalage ; les messages sont passés à `send`.
pub fn execute(base: MotorSpec, setup: FitSetup, stop: &AtomicBool, send: &mut dyn FnMut(FitMsg)) {
    let r = fit(&base, &setup, stop, |n, e| send(FitMsg::Progress(n, e)));
    send(FitMsg::Done(r));
}

/// Lance le recalage en tâche de fond (fil sur ordinateur, Web Worker dans le navigateur).
pub fn run(base: MotorSpec, setup: FitSetup) -> FitJob {
    let (tx, rx) = channel();
    let stop = Arc::new(AtomicBool::new(false));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let stop2 = stop.clone();
        std::thread::spawn(move || {
            execute(base, setup, &stop2, &mut |m| {
                let _ = tx.send(m);
            })
        });
        FitJob { rx, stop, worker: None }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let w = crate::web::worker(crate::bg::Request::Fit(base, setup), move |r| {
            if let crate::bg::Reply::Fit(m) = r {
                let _ = tx.send(m);
            }
        });
        FitJob { rx, stop, worker: Some(w) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mesure synthétique (Br = 1,30 T, entrefer 0,25 mm, origine décalée de
    /// 7°) : le recalage retrouve les valeurs à partir de 1,45 T / 0,20 mm.
    #[test]
    fn retrouve_br_et_entrefer() {
        let mut truth = MotorSpec::default();
        truth.br_magnet = 1.30;
        truth.airgap = 0.25;
        let (xs, ys, per) = computed(&truth, MeasKind::Emf, truth.rpm).unwrap();
        let meas: Vec<[f64; 2]> = (0..90).map(|k| {
            let a = 4.0 * k as f64;
            [a, crate::dynamics::interp_periodic(&xs, &ys, per, a + 7.0)]
        }).collect();
        let mut base = truth.clone();
        base.br_magnet = 1.45;
        base.airgap = 0.20;
        let setup = FitSetup { kind: MeasKind::Emf, meas, meas_rpm: truth.rpm, vars: vec![FitVar::Br, FitVar::Airgap], budget: 80 };
        let r = fit(&base, &setup, &AtomicBool::new(false), |_, _| {}).unwrap();
        assert!(r.err_initial > 5.0, "{}", r.err_initial);
        assert!(r.err_fitted < 0.5, "écart final {} %", r.err_fitted);
        // la FEM dépend surtout du produit Br·(flux) : on vérifie la FEM et une valeur plausible
        assert!((r.fitted[0] - 1.30).abs() < 0.08 && (r.fitted[1] - 0.25).abs() < 0.06, "{:?}", r.fitted);
    }
}
