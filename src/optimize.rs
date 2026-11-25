//! « Ajustement I.A » : recherche automatique des meilleurs réglages.
//!
//! Méthode : optimisation numérique multi-départs (Nelder–Mead dans l'espace
//! normalisé des variables choisies), avec contraintes par pénalités
//! (aimants sans chevauchement, balourd, FEM minimale, force radiale). Chaque
//! essai est un calcul complet du modèle (résolution allégée pour la vitesse).
//! Les cinq meilleures solutions DISTINCTES sont envoyées en continu à
//! l'interface, qui les affiche et permet de les appliquer d'un clic.
//!
//! Chaque essai est aussi tracé (`Trial` : étape du simplexe, valeurs,
//! grandeurs, contraintes violées, nouveau meilleur) et les événements de la
//! résolution (référence, départs, convergence, arrêt) sont signalés : la
//! console du moteur I.A les affiche en direct.

use crate::analysis;
use crate::geometry::{self, MotorSpec};
use crate::subdomain::{Model, Resolution};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

/// Variable de réglage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Var {
    /// Arc des aimants / pas polaire.
    ArcRatio,
    /// Ouverture d'encoche.
    SlotOpen,
    /// Épaisseur des aimants.
    MagnetTh,
    /// Amplitude des décalages de la répartition.
    SkewAmp,
    /// Épaisseur des becs de dents.
    TipH,
    /// Entrefer.
    Airgap,
}

/// Toutes les variables proposées.
pub const ALL_VARS: [Var; 6] = [Var::ArcRatio, Var::SlotOpen, Var::MagnetTh, Var::SkewAmp, Var::TipH, Var::Airgap];

impl Var {
    /// Nom français (clé de traduction).
    pub fn name(self) -> &'static str {
        match self {
            Var::ArcRatio => "Arc aimant / pas polaire",
            Var::SlotOpen => "Ouverture d'encoche",
            Var::MagnetTh => "Épaisseur aimant",
            Var::SkewAmp => "Amplitude max des décalages",
            Var::TipH => "Becs de dents (0 = encoches ouvertes)",
            Var::Airgap => "Entrefer",
        }
    }
    /// Bornes (min, max) autour de la spécification de départ.
    pub fn bounds(self, s: &MotorSpec) -> (f64, f64) {
        let pitch = 2.0 * std::f64::consts::PI * s.r_bore() * 1e3 / s.n_slots as f64;
        match self {
            Var::ArcRatio => (0.5, 0.98),
            Var::SlotOpen => (0.15 * pitch, 0.7 * pitch),
            Var::MagnetTh => (0.5 * s.magnet_th, 1.5 * s.magnet_th),
            Var::SkewAmp => (0.0, 8.0),
            Var::TipH => (0.0, (0.3 * s.slot_depth_m() * 1e3).max(0.1)),
            Var::Airgap => (0.6 * s.airgap, 1.6 * s.airgap),
        }
    }
    /// Valeur actuelle dans la spécification.
    pub fn get(self, s: &MotorSpec) -> f64 {
        match self {
            Var::ArcRatio => s.magnet_arc_ratio,
            Var::SlotOpen => s.slot_open,
            Var::MagnetTh => s.magnet_th,
            Var::SkewAmp => s.skew_max_deg,
            Var::TipH => s.tip_h,
            Var::Airgap => s.airgap,
        }
    }
    /// Modifie la spécification.
    pub fn set(self, s: &mut MotorSpec, v: f64) {
        match self {
            Var::ArcRatio => s.magnet_arc_ratio = v,
            Var::SlotOpen => s.slot_open = v,
            Var::MagnetTh => s.magnet_th = v,
            Var::SkewAmp => s.skew_max_deg = v,
            Var::TipH => {
                s.tip_h = v;
                if v > 0.0 && s.slot_w <= s.slot_open {
                    s.slot_w = s.slot_open * 1.8;
                }
            }
            Var::Airgap => s.airgap = v,
        }
    }
}

/// Objectif de l'ajustement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Goal {
    /// Couple de détente minimal (FEM conservée).
    MinCogging,
    /// Force radiale résultante minimale.
    MinForce,
    /// FEM (donc couple) maximale à cogging limité.
    MaxEmf,
    /// Compromis cogging / force / FEM.
    Balanced,
    /// Ondulation de couple en charge minimale (couple moyen conservé).
    MinRipple,
    /// Spectre du couple de détente le plus étalé : part de la raie dominante minimale.
    MinDominant,
    /// Entropie spectrale du couple de détente maximale.
    MaxEntropy,
    /// Compromis : spectre étalé et couple de détente faible.
    SpreadLow,
}

/// Tous les objectifs proposés.
pub const ALL_GOALS: [Goal; 8] = [Goal::MinCogging, Goal::MinForce, Goal::MaxEmf, Goal::Balanced, Goal::MinRipple, Goal::MinDominant, Goal::MaxEntropy, Goal::SpreadLow];

impl Goal {
    /// Nom français (clé de traduction).
    pub fn name(self) -> &'static str {
        match self {
            Goal::MinCogging => "Couple de détente minimal",
            Goal::MinForce => "Force radiale minimale",
            Goal::MaxEmf => "FEM maximale",
            Goal::Balanced => "Compromis cogging / force / FEM",
            Goal::MinRipple => "Ondulation de couple minimale",
            Goal::MinDominant => "Spectre le plus étalé (raie dominante minimale)",
            Goal::MaxEntropy => "Entropie spectrale maximale",
            Goal::SpreadLow => "Compromis spectre étalé / cogging faible",
        }
    }
}

/// Réglages de l'ajustement.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Setup {
    /// Variables libres.
    pub vars: Vec<Var>,
    /// Objectif.
    pub goal: Goal,
    /// Nombre maximal d'essais.
    pub budget: usize,
    /// FEM minimale conservée (fraction de celle de départ).
    pub emf_keep: f64,
    /// Culasse du rotor non saturée : l'épaisseur de culasse de chaque essai
    /// est portée à la valeur qui ramène son induction à vide (linéaire) au
    /// coude du matériau. La culasse ne change pas le champ dans l'entrefer
    /// (fer idéal), seulement sa propre induction : ce n'est donc pas une
    /// variable de l'optimisation mais une conséquence de chaque essai.
    pub sat_limit: bool,
    /// Amplitude minimale des décalages [°] quand elle est un réglage libre :
    /// empêche l'ajustement de revenir à un rotor régulier.
    pub skew_min: f64,
    /// Pertes Joule au plus : original × (1 + x/100) ; `None` = libres.
    pub joule_max_pct: Option<f64>,
    /// Couple moyen en charge au moins : fraction de l'original ; `None` = libre.
    pub torque_keep: Option<f64>,
}

impl Setup {
    /// Bornes d'une variable pour cet ajustement.
    pub fn bounds(&self, v: Var, s: &MotorSpec) -> (f64, f64) {
        let (lo, hi) = v.bounds(s);
        if v == Var::SkewAmp && self.skew_min > 0.0 {
            (lo.max(self.skew_min), hi.max(self.skew_min + 0.5))
        } else {
            (lo, hi)
        }
    }
}

/// Une proposition évaluée.
#[derive(Clone, Debug, Serialize, serde::Deserialize)]
pub struct Candidate {
    /// Valeurs des variables (même ordre que `Setup::vars`).
    pub values: Vec<f64>,
    /// Score (plus bas = meilleur).
    pub score: f64,
    /// Cogging crête-à-crête [mN·m].
    pub cogging_mnm: f64,
    /// Force radiale max [N].
    pub force_n: f64,
    /// FEM crête [V].
    pub emf_v: f64,
    /// Écart minimal entre aimants [°].
    pub min_gap_deg: f64,
    /// Induction crête à vide dans la culasse du rotor, linéaire [T].
    pub b_rotor_t: f64,
    /// Épaisseur de la culasse du rotor de l'essai [mm].
    pub rotor_yoke_mm: f64,
    /// Part de la raie dominante dans le spectre du couple de détente [%].
    #[serde(default)]
    pub dominant_share_pct: f64,
    /// Entropie spectrale du couple de détente.
    #[serde(default)]
    pub entropy: f64,
    /// Pertes Joule au courant de la spécification [W].
    #[serde(default)]
    pub joule_w: f64,
    /// Couple moyen en charge [N·m].
    pub torque_nm: f64,
    /// Ondulation de couple en charge [%].
    pub ripple_pct: f64,
    /// Contraintes respectées ?
    pub feasible: bool,
}

/// Étape de la résolution qui a produit un essai.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    /// Sommet du simplexe initial d'un départ.
    Initial,
    /// Réflexion du plus mauvais sommet.
    Reflexion,
    /// Expansion (la réflexion était la meilleure).
    Expansion,
    /// Contraction vers le centre.
    Contraction,
    /// Rétrécissement de tout le simplexe vers le meilleur sommet.
    Shrink,
}

impl Step {
    /// Nom français (clé de traduction).
    pub fn name(self) -> &'static str {
        match self {
            Step::Initial => "simplexe initial",
            Step::Reflexion => "réflexion",
            Step::Expansion => "expansion",
            Step::Contraction => "contraction",
            Step::Shrink => "rétrécissement",
        }
    }
}

/// Contrainte violée par un essai.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Violation {
    /// Géométrie impossible (le modèle ne se construit pas).
    Invalid,
    /// Aimants qui se chevauchent.
    Overlap,
    /// Balourd trop grand.
    Unbalance,
    /// FEM sous le seuil conservé.
    EmfLow,
    /// Couple moyen en charge sous le seuil conservé.
    TorqueLow,
    /// Pertes Joule au-dessus du seuil.
    JouleHigh,
}

impl Violation {
    /// Nom français (clé de traduction).
    pub fn name(self) -> &'static str {
        match self {
            Violation::Invalid => "géométrie impossible",
            Violation::Overlap => "aimants qui se chevauchent",
            Violation::Unbalance => "balourd au-dessus du seuil",
            Violation::EmfLow => "FEM sous le seuil",
            Violation::TorqueLow => "couple moyen sous le seuil",
            Violation::JouleHigh => "pertes Joule au-dessus du seuil",
        }
    }
}

/// Un essai tracé.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trial {
    /// Numéro de l'essai (1…).
    pub n: usize,
    /// Numéro du départ (1 = réglage actuel, puis départs aléatoires).
    pub start: usize,
    /// Étape du simplexe.
    pub step: Step,
    /// Résultat de l'essai.
    pub cand: Candidate,
    /// Contraintes violées.
    pub violations: Vec<Violation>,
    /// Nouveau meilleur score faisable ?
    pub new_best: bool,
    /// Durée du calcul de l'essai [ms].
    pub ms: f64,
}

/// Événement de la résolution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    /// Grandeurs du projet de départ (référence des scores).
    Reference(Candidate),
    /// Nouveau départ (numéro, aléatoire ?, point de départ en valeurs réelles).
    Start(usize, bool, Vec<f64>),
    /// Départ terminé : (numéro, convergé ? sinon limite d'itérations, meilleur score du départ).
    StartEnd(usize, bool, f64),
    /// Fin : (essais, budget épuisé ?, arrêt demandé ?).
    End(usize, bool, bool),
    /// Le projet de départ ne se calcule pas.
    Failed(String),
}

/// Messages de l'ajustement.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum OptMsg {
    /// Avancement : (essais faits, budget, 5 meilleurs, historique du meilleur score).
    Progress(usize, usize, Vec<Candidate>, Vec<f64>),
    /// Essais tracés depuis le dernier envoi.
    Trace(Vec<Trial>),
    /// Événement de la résolution.
    Event(Event),
    /// Terminé.
    Done,
}

/// Tâche d'ajustement en cours.
pub struct OptJob {
    /// Canal des messages.
    pub rx: Receiver<OptMsg>,
    /// Demande d'arrêt.
    pub stop: Arc<AtomicBool>,
    /// Web Worker (navigateur).
    pub worker: Option<crate::compute::WorkerHandle>,
}

impl OptJob {
    /// Arrête l'ajustement : drapeau d'arrêt sur ordinateur ; dans le
    /// navigateur, le Worker est arrêté (le meilleur résultat reçu est gardé).
    pub fn cancel(&mut self) -> bool {
        self.stop.store(true, Ordering::Relaxed);
        // vrai si la tâche est terminée tout de suite (Worker arrêté)
        self.worker.take().is_some()
    }
}

/// Applique des valeurs de variables à une spécification.
pub fn apply(base: &MotorSpec, vars: &[Var], values: &[f64]) -> MotorSpec {
    let mut s = base.clone();
    for (v, x) in vars.iter().zip(values) {
        v.set(&mut s, *x);
    }
    s
}

/// Applique une proposition complète (variables et culasse du rotor dimensionnée).
pub fn apply_candidate(base: &MotorSpec, vars: &[Var], c: &Candidate) -> MotorSpec {
    let mut s = apply(base, vars, &c.values);
    if c.rotor_yoke_mm > 0.0 {
        s.rotor_yoke_th = c.rotor_yoke_mm;
    }
    s
}

struct Ref {
    cog: f64,
    force: f64,
    emf: f64,
    torque: f64,
    ripple: f64,
    share: f64,
    joule: f64,
    b_knee: f64,
}

/// Mesure un projet : grandeurs d'un candidat (sans score).
fn measure(s: &MotorSpec, values: Vec<f64>) -> Result<Candidate, String> {
    let m = Model::new(s, Resolution { nh: 160, nk: 10 })?;
    let span = 360.0 / s.n_slots as f64;
    let cg = analysis::cogging(&m, span, 90);
    let fc = analysis::forces(&m, 360.0, 72);
    let em = analysis::emf(&m, 144);
    let (tq, rp) = analysis::ripple_quick(&m, &em, 48);
    let (_, _, b_rotor, _) = crate::saturation::no_load_inductions(&m);
    let gap = geometry::magnet_gaps_deg(s).into_iter().fold(f64::MAX, f64::min);
    Ok(Candidate {
        values,
        score: 0.0,
        cogging_mnm: cg.metrics.cogging_pp_mnm,
        force_n: fc.max_n,
        emf_v: em.emf_peak_v,
        min_gap_deg: gap,
        b_rotor_t: b_rotor,
        rotor_yoke_mm: s.rotor_yoke_th,
        joule_w: 1.5 * analysis::phase_resistance(&m) * s.load.i_peak * s.load.i_peak,
        dominant_share_pct: cg.metrics.dominant_share_pct,
        entropy: cg.metrics.spectral_entropy,
        torque_nm: tq,
        ripple_pct: rp,
        feasible: true,
    })
}

fn evaluate(base: &MotorSpec, st: &Setup, x: &[f64], r: &Ref) -> (Candidate, Vec<Violation>) {
    let values: Vec<f64> = st.vars.iter().zip(x).map(|(v, u)| {
        let (lo, hi) = st.bounds(*v, base);
        lo + u.clamp(0.0, 1.0) * (hi - lo)
    }).collect();
    let s = apply(base, &st.vars, &values);
    let bad = Candidate { values: values.clone(), score: 1e9, cogging_mnm: 0.0, force_n: 0.0, emf_v: 0.0, min_gap_deg: 0.0, b_rotor_t: 0.0, rotor_yoke_mm: 0.0, dominant_share_pct: 0.0, entropy: 0.0, joule_w: 0.0, torque_nm: 0.0, ripple_pct: 0.0, feasible: false };
    if s.geometry_problem().is_some() {
        return (bad, vec![Violation::Invalid]);
    }
    let Ok(mut c) = measure(&s, values) else { return (bad, vec![Violation::Invalid]) };
    let bal = geometry::balourd_mg_cm(&s);
    let (cg, f, e) = (c.cogging_mnm, c.force_n, c.emf_v);
    let rc = cg / r.cog.max(1e-9);
    let base_score = match st.goal {
        Goal::MinCogging => rc,
        Goal::MinForce => f / r.force.max(1e-3) + 0.05 * rc,
        Goal::MaxEmf => -e / r.emf.max(1e-9) + 0.3 * (rc - 1.0).max(0.0),
        Goal::Balanced => rc + 0.5 * f / r.force.max(0.1) - e / r.emf.max(1e-9),
        Goal::MinRipple => c.ripple_pct / r.ripple.max(1e-6),
        Goal::MinDominant => c.dominant_share_pct / 100.0,
        Goal::MaxEntropy => -c.entropy,
        Goal::SpreadLow => c.dominant_share_pct / r.share.max(1e-6) + rc,
    };
    // pénalités de contraintes
    let mut pen = 0.0;
    let mut viol = Vec::new();
    if c.min_gap_deg < 0.05 {
        pen += 10.0 * (0.05 - c.min_gap_deg);
        viol.push(Violation::Overlap);
    }
    if let Some(bm) = crate::repartition::criteria().balourd_max_mg_cm
        && bal > bm
    {
        pen += 0.1 * (bal - bm);
        viol.push(Violation::Unbalance);
    }
    if st.goal != Goal::MaxEmf && e < st.emf_keep * r.emf {
        pen += 5.0 * (st.emf_keep * r.emf - e) / r.emf.max(1e-9);
        viol.push(Violation::EmfLow);
    }
    let keep = st.torque_keep.or((st.goal == Goal::MinRipple).then_some(st.emf_keep));
    if let Some(k) = keep
        && c.torque_nm < k * r.torque
    {
        pen += 5.0 * (k * r.torque - c.torque_nm) / r.torque.abs().max(1e-9);
        viol.push(Violation::TorqueLow);
    }
    if let Some(x) = st.joule_max_pct
        && c.joule_w > r.joule * (1.0 + x / 100.0)
    {
        pen += 2.0 * (c.joule_w / (r.joule * (1.0 + x / 100.0)).max(1e-12) - 1.0);
        viol.push(Violation::JouleHigh);
    }
    if st.sat_limit && c.b_rotor_t > r.b_knee {
        // même flux, culasse plus épaisse : B ∝ 1 / épaisseur
        c.rotor_yoke_mm = (s.rotor_yoke_th * c.b_rotor_t / r.b_knee * 100.0).ceil() / 100.0;
        c.b_rotor_t *= s.rotor_yoke_th / c.rotor_yoke_mm;
    }
    c.score = base_score + pen;
    c.feasible = viol.is_empty();
    (c, viol)
}

/// Exécute l'ajustement ; les messages sont passés à `send`, `stop` l'interrompt.
pub fn execute(base: MotorSpec, st: Setup, stop: &AtomicBool, send: &mut dyn FnMut(OptMsg)) {
    let d = st.vars.len().max(1);
    // référence : la spécification de départ
    let x_ref: Vec<f64> = st.vars.iter().map(|v| v.get(&base)).collect();
    let c0 = match measure(&base, x_ref) {
        Ok(c) => c,
        Err(e) => {
            send(OptMsg::Event(Event::Failed(e)));
            send(OptMsg::Done);
            return;
        }
    };
    let r = Ref { cog: c0.cogging_mnm, force: c0.force_n, emf: c0.emf_v, torque: c0.torque_nm, ripple: c0.ripple_pct, share: c0.dominant_share_pct, joule: c0.joule_w, b_knee: base.saturation.rotor.knee() };
    send(OptMsg::Event(Event::Reference(c0)));
    let x0: Vec<f64> = st.vars.iter().map(|v| {
        let (lo, hi) = st.bounds(*v, &base);
        ((v.get(&base) - lo) / (hi - lo).max(1e-12)).clamp(0.0, 1.0)
    }).collect();
    let real = |x: &[f64]| -> Vec<f64> { st.vars.iter().zip(x).map(|(v, u)| { let (lo, hi) = st.bounds(*v, &base); lo + u * (hi - lo) }).collect() };
    let mut all: Vec<Candidate> = Vec::new();
    let mut hist: Vec<f64> = Vec::new();
    let mut trace: Vec<Trial> = Vec::new();
    let mut evals = 0usize;
    let mut n_start = 0usize;
    let mut best_feasible = f64::INFINITY;
    let mut seed = 0x9e3779b97f4a7c15u64;
    let mut rnd = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    let top5 = |all: &Vec<Candidate>| -> Vec<Candidate> {
        let mut v: Vec<Candidate> = all.clone();
        v.sort_by(|a, b| a.score.total_cmp(&b.score));
        let mut out: Vec<Candidate> = Vec::new();
        for c in v {
            // solutions distinctes : écart normalisé > 3 %
            let distinct = out.iter().all(|o| o.values.iter().zip(&c.values).zip(&st.vars).any(|((a, b), var)| {
                let (lo, hi) = st.bounds(*var, &base);
                (a - b).abs() / (hi - lo).max(1e-12) > 0.03
            }));
            if distinct {
                out.push(c);
            }
            if out.len() == 5 {
                break;
            }
        }
        out
    };
    let eval = |x: &[f64], step: Step, n_start: usize, all: &mut Vec<Candidate>, hist: &mut Vec<f64>, trace: &mut Vec<Trial>, evals: &mut usize, best_feasible: &mut f64, send: &mut dyn FnMut(OptMsg)| -> f64 {
        let t0 = web_time::Instant::now();
        let (c, violations) = evaluate(&base, &st, x, &r);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let sc = c.score;
        let new_best = c.feasible && sc < *best_feasible - 1e-12;
        if new_best {
            *best_feasible = sc;
        }
        *evals += 1;
        trace.push(Trial { n: *evals, start: n_start, step, cand: c.clone(), violations, new_best, ms });
        all.push(c);
        let best = all.iter().map(|c| c.score).fold(f64::MAX, f64::min);
        hist.push(best);
        if (*evals).is_multiple_of(3) {
            send(OptMsg::Trace(std::mem::take(trace)));
            send(OptMsg::Progress(*evals, st.budget, top5(all), hist.clone()));
        }
        sc
    };
    // multi-départs : départ = réglage actuel, puis points aléatoires
    let mut start = x0.clone();
    let mut stopped = false;
    'outer: while evals < st.budget {
        n_start += 1;
        send(OptMsg::Event(Event::Start(n_start, n_start > 1, real(&start))));
        let mut converged = false;
        // simplexe initial
        let step = 0.15;
        let mut simplex: Vec<Vec<f64>> = vec![start.clone()];
        for i in 0..d {
            let mut p = start.clone();
            p[i] = if p[i] + step <= 1.0 { p[i] + step } else { p[i] - step };
            simplex.push(p);
        }
        let mut f: Vec<f64> = Vec::new();
        for p in &simplex {
            if stop.load(Ordering::Relaxed) || evals >= st.budget {
                stopped = stop.load(Ordering::Relaxed);
                break 'outer;
            }
            f.push(eval(p, Step::Initial, n_start, &mut all, &mut hist, &mut trace, &mut evals, &mut best_feasible, send));
        }
        for _iter in 0..(40 * d) {
            if stop.load(Ordering::Relaxed) || evals >= st.budget {
                stopped = stop.load(Ordering::Relaxed);
                break 'outer;
            }
            let mut idx: Vec<usize> = (0..simplex.len()).collect();
            idx.sort_by(|a, b| f[*a].total_cmp(&f[*b]));
            simplex = idx.iter().map(|&i| simplex[i].clone()).collect();
            f = idx.iter().map(|&i| f[i]).collect();
            if (f[d] - f[0]).abs() < 1e-4 {
                converged = true;
                break;
            }
            let cen: Vec<f64> = (0..d).map(|j| simplex[..d].iter().map(|p| p[j]).sum::<f64>() / d as f64).collect();
            let lerp = |a: &Vec<f64>, b: &Vec<f64>, t: f64| -> Vec<f64> { a.iter().zip(b).map(|(x, y)| (x + t * (y - x)).clamp(0.0, 1.0)).collect() };
            let xr = lerp(&cen, &simplex[d], -1.0);
            let fr = eval(&xr, Step::Reflexion, n_start, &mut all, &mut hist, &mut trace, &mut evals, &mut best_feasible, send);
            if fr < f[0] {
                let xe = lerp(&cen, &simplex[d], -2.0);
                let fe = eval(&xe, Step::Expansion, n_start, &mut all, &mut hist, &mut trace, &mut evals, &mut best_feasible, send);
                if fe < fr { simplex[d] = xe; f[d] = fe; } else { simplex[d] = xr; f[d] = fr; }
            } else if fr < f[d - 1] {
                simplex[d] = xr;
                f[d] = fr;
            } else {
                let xc = lerp(&cen, &simplex[d], 0.5);
                let fcn = eval(&xc, Step::Contraction, n_start, &mut all, &mut hist, &mut trace, &mut evals, &mut best_feasible, send);
                if fcn < f[d] {
                    simplex[d] = xc;
                    f[d] = fcn;
                } else {
                    for i in 1..=d {
                        simplex[i] = lerp(&simplex[0], &simplex[i], 0.5);
                        f[i] = eval(&simplex[i].clone(), Step::Shrink, n_start, &mut all, &mut hist, &mut trace, &mut evals, &mut best_feasible, send);
                    }
                }
            }
        }
        let best_here = f.iter().cloned().fold(f64::INFINITY, f64::min);
        send(OptMsg::Event(Event::StartEnd(n_start, converged, best_here)));
        start = (0..d).map(|_| rnd()).collect();
    }
    send(OptMsg::Trace(trace));
    send(OptMsg::Progress(evals, st.budget, top5(&all), hist));
    send(OptMsg::Event(Event::End(evals, evals >= st.budget, stopped)));
    send(OptMsg::Done);
}

/// Lance l'ajustement en tâche de fond (fil sur ordinateur, Web Worker dans le navigateur).
pub fn run(base: MotorSpec, st: Setup) -> OptJob {
    let (tx, rx) = channel();
    let stop = Arc::new(AtomicBool::new(false));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let stop2 = stop.clone();
        std::thread::spawn(move || {
            execute(base, st, &stop2, &mut |m| {
                let _ = tx.send(m);
            })
        });
        OptJob { rx, stop, worker: None }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let w = crate::web::worker(crate::bg::Request::Opt(base, st), move |r| {
            if let crate::bg::Reply::Opt(m) = r {
                let _ = tx.send(m);
            }
        });
        OptJob { rx, stop, worker: Some(w) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ajustement_reduit_le_cogging_sans_chevauchement() {
        let base = MotorSpec::irregular_example();
        let st = Setup { vars: vec![Var::ArcRatio, Var::SlotOpen], goal: Goal::MinCogging, budget: 40, emf_keep: 0.85, sat_limit: false, skew_min: 0.0, joule_max_pct: None, torque_keep: None };
        let job = run(base.clone(), st);
        let mut best = None;
        while let Ok(m) = job.rx.recv() {
            match m {
                OptMsg::Progress(_, _, top, _) => best = top.first().cloned(),
                OptMsg::Done => break,
                _ => {}
            }
        }
        let b = best.unwrap();
        let m0 = Model::new(&base, Resolution { nh: 160, nk: 10 }).unwrap();
        let c0 = analysis::cogging(&m0, 360.0 / base.n_slots as f64, 90).metrics.cogging_pp_mnm;
        assert!(b.feasible, "contraintes non respectées : {:?}", b);
        assert!(b.cogging_mnm < 0.5 * c0, "cogging {} vs départ {}", b.cogging_mnm, c0);
        assert!(b.min_gap_deg >= 0.05);
    }
}
