//! Console de script : enchaîne des calculs par commandes, une par ligne.
//!
//! Le script travaille sur une copie du projet ; `applique` renvoie la copie
//! modifiée à l'interface. Les commandes (français, alias anglais) :
//!
//! | commande | effet |
//! |---|---|
//! | `aide` | liste des commandes et des paramètres |
//! | `variante reguliere` | répartition des aimants proposée |
//! | `regle arc 0,8` | modifie un paramètre (`set`) |
//! | `montre [arc]` | affiche un ou tous les paramètres (`show`) |
//! | `calcul` | calcul complet, grandeurs principales (`calc`) |
//! | `ondulation` | décomposition de l'ondulation du dernier calcul (`ripple`) |
//! | `inductance` | matrice d'inductances du dernier calcul |
//! | `balaye arc 0,6 0,9 7` | étude paramétrique (`sweep`) |
//! | `compare` | toutes les variantes avec les réglages du projet |
//! | `ia cogging 120 arc,amplitude [sat]` | Ajustement I.A tracé, garde la meilleure solution |
//! | `applique` | reporte le projet du script dans l'interface (`apply`) |
//! | `initial` | revient au projet de départ du script (`reset`) |
//!
//! Les lignes vides et celles qui commencent par `#` sont ignorées. Les
//! nombres acceptent la virgule ou le point décimal.

use crate::compute::{compute_results, Results, Settings};
use crate::geometry::MotorSpec;
use crate::i18n::tr;
use crate::optimize::{self, Event, Goal, OptMsg, Trial, Var};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

/// Nature d'une ligne de sortie (couleur dans la console).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    /// Commande exécutée (écho).
    Cmd,
    /// Information.
    Info,
    /// Résultat chiffré.
    Result,
    /// Erreur.
    Error,
    /// Trace du moteur I.A.
    Ia,
}

/// Message de la console de script.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ScriptMsg {
    /// Une ligne de sortie.
    Line(Kind, String),
    /// Projet à reporter dans l'interface (`applique`).
    Spec(Box<MotorSpec>),
    /// Script terminé, avec le projet du script à ce moment (la ligne de
    /// commande de la console repart de lui à la commande suivante).
    Done(Box<MotorSpec>),
}

/// Script en cours.
pub struct ScriptJob {
    /// Canal des messages.
    pub rx: Receiver<ScriptMsg>,
    /// Demande d'arrêt.
    pub stop: Arc<AtomicBool>,
    /// Web Worker (navigateur).
    pub worker: Option<crate::compute::WorkerHandle>,
}

impl ScriptJob {
    /// Arrête le script (vrai si la tâche est terminée tout de suite).
    pub fn cancel(&mut self) -> bool {
        self.stop.store(true, Ordering::Relaxed);
        self.worker.take().is_some()
    }
}

/// Paramètre réglable par script : (nom court, nom français, unité).
pub const PARAMS: [(&str, &str, &str); 17] = [
    ("arc", "Arc aimant / pas polaire", ""),
    ("amplitude", "Amplitude max des décalages", "°"),
    ("ouverture", "Ouverture d'encoche", "mm"),
    ("aimant", "Épaisseur aimant", "mm"),
    ("becs", "Becs de dents (0 = encoches ouvertes)", "mm"),
    ("corps", "Largeur du corps d'encoche", "mm"),
    ("entrefer", "Entrefer", "mm"),
    ("culasse_rotor", "Culasse rotor", "mm"),
    ("culasse_stator", "Culasse stator", "mm"),
    ("dent", "Largeur de dent", "mm"),
    ("br", "Induction rémanente Br", "T"),
    ("longueur", "Longueur active", "mm"),
    ("courant", "Courant de phase crête", "A"),
    ("angle", "Angle du courant / FEM", "°él"),
    ("spires", "Spires par bobine", ""),
    ("vitesse", "Vitesse", "tr/min"),
    ("mur", "Perméabilité de recul μr", ""),
];

/// Valeur d'un paramètre (nom court).
pub fn param_get(s: &MotorSpec, name: &str) -> Option<f64> {
    Some(match name {
        "arc" => s.magnet_arc_ratio,
        "amplitude" => s.skew_max_deg,
        "ouverture" => s.slot_open,
        "aimant" => s.magnet_th,
        "becs" => s.tip_h,
        "corps" => s.slot_w,
        "entrefer" => s.airgap,
        "culasse_rotor" => s.rotor_yoke_th,
        "culasse_stator" => s.yoke_th,
        "dent" => s.tooth_w,
        "br" => s.br_magnet,
        "longueur" => s.stack_len,
        "courant" => s.load.i_peak,
        "angle" => s.load.angle_deg,
        "spires" => s.turns_per_coil,
        "vitesse" => s.rpm,
        "mur" => s.mur_magnet,
        _ => return None,
    })
}

/// Modifie un paramètre (nom court) ; faux si le nom est inconnu.
pub fn param_set(s: &mut MotorSpec, name: &str, v: f64) -> bool {
    match name {
        "arc" => s.magnet_arc_ratio = v,
        "amplitude" => s.skew_max_deg = v,
        "ouverture" => s.slot_open = v,
        "aimant" => s.magnet_th = v,
        "becs" => Var::TipH.set(s, v),
        "corps" => s.slot_w = v,
        "entrefer" => s.airgap = v,
        "culasse_rotor" => s.rotor_yoke_th = v,
        "culasse_stator" => s.yoke_th = v,
        "dent" => s.tooth_w = v,
        "br" => s.br_magnet = v,
        "longueur" => s.stack_len = v,
        "courant" => s.load.i_peak = v,
        "angle" => s.load.angle_deg = v,
        "spires" => {
            s.turns_per_coil = v;
            for c in s.coils.iter_mut() {
                c.turns = v;
            }
        }
        "vitesse" => s.rpm = v,
        "mur" => s.mur_magnet = v,
        _ => return false,
    }
    true
}

/// Variable de l'Ajustement I.A correspondant à un nom court.
pub fn var_of(name: &str) -> Option<Var> {
    Some(match name {
        "arc" => Var::ArcRatio,
        "amplitude" => Var::SkewAmp,
        "ouverture" => Var::SlotOpen,
        "aimant" => Var::MagnetTh,
        "becs" => Var::TipH,
        "entrefer" => Var::Airgap,
        _ => return None,
    })
}

/// Nom court d'une variable de l'Ajustement I.A.
pub fn short_of(v: Var) -> &'static str {
    match v {
        Var::ArcRatio => "arc",
        Var::SkewAmp => "amplitude",
        Var::SlotOpen => "ouverture",
        Var::MagnetTh => "aimant",
        Var::TipH => "becs",
        Var::Airgap => "entrefer",
    }
}

fn goal_of(name: &str) -> Option<Goal> {
    Some(match name {
        "cogging" | "detente" | "détente" => Goal::MinCogging,
        "force" => Goal::MinForce,
        "fem" | "emf" => Goal::MaxEmf,
        "compromis" | "balanced" => Goal::Balanced,
        "ondulation" | "ripple" => Goal::MinRipple,
        _ => return None,
    })
}

/// Nombre écrit avec virgule ou point décimal.
pub fn num(s: &str) -> Option<f64> {
    s.trim().replace(',', ".").parse().ok()
}

fn fmt(v: f64) -> String {
    let a = v.abs();
    if a == 0.0 {
        "0".into()
    } else if a >= 1000.0 {
        format!("{v:.0}")
    } else if a >= 10.0 {
        format!("{v:.2}")
    } else if a >= 0.01 {
        format!("{v:.4}")
    } else {
        format!("{v:.3e}")
    }
}

/// Une ligne de trace de l'Ajustement I.A, lisible.
pub fn trial_line(t: &Trial, vars: &[Var]) -> String {
    let c = &t.cand;
    let vals: Vec<String> = vars.iter().zip(&c.values).map(|(v, x)| format!("{}={}", short_of(*v), fmt(*x))).collect();
    let mut l = format!(
        "#{:<4} {}{:<2} {:<15} {}  → {} {:.4}  cog {} mN·m  F {} N  FEM {} V  Bcr {:.2} T ({} mm)",
        t.n,
        tr("dép."),
        t.start,
        tr(t.step.name()),
        vals.join(" "),
        tr("score"),
        c.score,
        fmt(c.cogging_mnm),
        fmt(c.force_n),
        fmt(c.emf_v),
        c.b_rotor_t,
        fmt(c.rotor_yoke_mm)
    );
    if c.torque_nm != 0.0 {
        l += &format!("  {} {:.1} %", tr("ond."), c.ripple_pct);
    }
    if !t.violations.is_empty() {
        let v: Vec<&str> = t.violations.iter().map(|v| tr(v.name())).collect();
        l += &format!("  ✖ {}", v.join(", "));
    }
    if t.new_best {
        l += &format!("  ★ {}", tr("nouveau meilleur"));
    }
    l
}

/// Un événement de l'Ajustement I.A, lisible.
pub fn event_line(e: &Event, vars: &[Var]) -> String {
    match e {
        Event::Reference(c) => format!(
            "{} : cog {} mN·m · F {} N · FEM {} V · {} {} N·m · {} {:.1} % · Bcr {:.2} T",
            tr("Référence (projet de départ)"),
            fmt(c.cogging_mnm),
            fmt(c.force_n),
            fmt(c.emf_v),
            tr("couple"),
            fmt(c.torque_nm),
            tr("ond."),
            c.ripple_pct,
            c.b_rotor_t
        ),
        Event::Start(n, random, x) => {
            let vals: Vec<String> = vars.iter().zip(x).map(|(v, x)| format!("{}={}", short_of(*v), fmt(*x))).collect();
            format!("── {} {n} ({}) : {}", tr("départ"), if *random { tr("point aléatoire") } else { tr("réglage actuel") }, vals.join(" "))
        }
        Event::StartEnd(n, conv, best) => format!("── {} {n} {} · {} {:.4}", tr("départ"), if *conv { tr("convergé") } else { tr("limite d'itérations atteinte") }, tr("meilleur score"), best),
        Event::End(n, budget, stop) => format!("■ {} : {n} {} ({})", tr("Fin de la résolution"), tr("essais"), if *stop { tr("arrêt demandé") } else if *budget { tr("budget épuisé") } else { tr("terminé") }),
        Event::Failed(e) => format!("✖ {} : {e}", tr("le projet de départ ne se calcule pas")),
    }
}

struct Ctx<'a> {
    spec: MotorSpec,
    initial: MotorSpec,
    st: Settings,
    last: Option<Results>,
    stop: &'a AtomicBool,
}

fn summary(r: &Results, out: &mut dyn FnMut(Kind, String)) {
    let o = crate::sweep::outputs(r);
    for ((n, u), v) in crate::sweep::OUTPUTS.iter().zip(&o) {
        out(Kind::Result, format!("  {:<34} {:>12} {u}", tr(n), fmt(*v)));
    }
    let sat = &r.sat.no_load;
    out(Kind::Result, format!("  {:<34} {:>5.2} / {:.2} / {:.2} T", tr("Induction dents / culasse stator / culasse rotor"), sat.b_tooth, sat.b_yoke, sat.b_rotor));
    out(Kind::Info, format!("  ({} {:.2} s)", tr("calcul en"), r.seconds));
}

/// Calcule le projet du script s'il ne l'a pas encore été depuis sa dernière modification.
fn ensure(c: &mut Ctx) -> Result<(), String> {
    let same = |a: &MotorSpec, b: &MotorSpec| serde_json::to_string(a).ok() == serde_json::to_string(b).ok();
    if c.last.as_ref().is_none_or(|r| !same(&r.spec, &c.spec)) {
        c.last = Some(compute_results(c.spec.clone(), &c.st)?);
    }
    Ok(())
}

fn cmd(c: &mut Ctx, line: &str, out: &mut dyn FnMut(Kind, String), spec_out: &mut dyn FnMut(MotorSpec)) -> Result<(), String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let Some(first) = words.first() else { return Ok(()) };
    let head = first.to_lowercase();
    let arg = |i: usize| words.get(i).copied().ok_or_else(|| format!("{} : {line}", tr("argument manquant")));
    let pname = |w: &str| w.to_lowercase().replace('-', "_");
    match head.as_str() {
        "aide" | "help" | "?" => {
            for l in HELP {
                out(Kind::Info, tr(l).to_string());
            }
            out(Kind::Info, format!("{} :", tr("Paramètres")));
            for (k, n, u) in PARAMS {
                out(Kind::Info, format!("  {k:<15} {} {}", tr(n), if u.is_empty() { String::new() } else { format!("[{u}]") }));
            }
        }
        "variante" | "variant" => {
            let w = arg(1)?;
            let reg = ["reguliere", "régulière", "regular"].contains(&w.to_lowercase().as_str());
            let Some(v) = crate::repartition::variants().into_iter().find(|v| v.eq_ignore_ascii_case(w) || (reg && crate::repartition::is_regular_name(v))) else {
                return Err(format!("{} : {w} ({})", tr("variante inconnue"), crate::repartition::variants().join(", ")));
            };
            c.spec.variant = v.into();
            c.spec.custom_angles_deg = None;
            crate::repartition::on_select(&mut c.spec);
            out(Kind::Info, format!("{} {v}", tr("variante")));
        }
        "regle" | "règle" | "set" => {
            let p = pname(arg(1)?);
            let v = num(arg(2)?).ok_or_else(|| format!("{} : {}", tr("nombre illisible"), words[2]))?;
            if !param_set(&mut c.spec, &p, v) {
                return Err(format!("{} : {p}", tr("paramètre inconnu")));
            }
            out(Kind::Info, format!("{p} = {}", fmt(v)));
        }
        "montre" | "show" => {
            let list: Vec<&str> = match words.get(1) {
                Some(w) => vec![*w],
                None => PARAMS.iter().map(|p| p.0).collect(),
            };
            for p in list {
                let p = pname(p);
                let v = param_get(&c.spec, &p).ok_or_else(|| format!("{} : {p}", tr("paramètre inconnu")))?;
                let u = PARAMS.iter().find(|x| x.0 == p).map(|x| x.2).unwrap_or("");
                out(Kind::Result, format!("  {p:<15} {:>12} {u}", fmt(v)));
            }
            out(Kind::Result, format!("  {:<15} {:>12}", "variante", c.spec.variant));
        }
        "calcul" | "calc" => {
            let r = compute_results(c.spec.clone(), &c.st)?;
            summary(&r, out);
            c.last = Some(r);
        }
        "ondulation" | "ripple" => {
            ensure(c)?;
            let r = c.last.as_ref().ok_or(tr("faire d'abord « calcul »"))?;
            let l = &r.load;
            out(Kind::Result, format!("  {:<34} {:>10} N·m", tr("Couple moyen"), fmt(l.avg_torque)));
            out(Kind::Result, format!("  {:<34} {:>10.2} %", tr("Ondulation totale"), l.ripple_pct));
            for (n, v) in [("dont couple de détente", l.ripple_parts_pct[0]), ("dont couple mutuel (FEM × courant)", l.ripple_parts_pct[1]), ("dont réluctance et harmoniques", l.ripple_parts_pct[2])] {
                out(Kind::Result, format!("  {:<34} {:>10.2} %", tr(n), v));
            }
            out(Kind::Info, tr("(les parties se compensent en partie : leur somme dépasse l'ondulation totale)").to_string());
        }
        "inductance" | "inductances" => {
            ensure(c)?;
            let r = c.last.as_ref().ok_or(tr("faire d'abord « calcul »"))?;
            let l = &r.load;
            for row in &l.inductances {
                out(Kind::Result, format!("  {:>10.4} {:>10.4} {:>10.4} mH", row[0] * 1e3, row[1] * 1e3, row[2] * 1e3));
            }
            out(Kind::Result, format!("  L − M = {:.4} mH · {} {:.4} mH · {} {:.4} mH", (l.inductances[0][0] - l.inductances[0][1]) * 1e3, tr("têtes"), l.l_end * 1e3, tr("synchrone"), l.ls * 1e3));
        }
        "balaye" | "sweep" => {
            let p = pname(arg(1)?);
            let a = num(arg(2)?).ok_or(tr("nombre illisible"))?;
            let b = num(arg(3)?).ok_or(tr("nombre illisible"))?;
            let n = words.get(4).and_then(|w| w.parse::<usize>().ok()).unwrap_or(5).clamp(2, 200);
            param_get(&c.spec, &p).ok_or_else(|| format!("{} : {p}", tr("paramètre inconnu")))?;
            out(Kind::Result, format!("  {:>12} {:>12} {:>10} {:>10} {:>10} {:>10} {:>8}", p, "cog mN·m", "F N", "FEM V", "C N·m", "ond. %", "Bcr T"));
            for k in 0..n {
                if c.stop.load(Ordering::Relaxed) {
                    return Err(tr("arrêt demandé").into());
                }
                let v = a + (b - a) * k as f64 / (n - 1) as f64;
                let mut s = c.spec.clone();
                param_set(&mut s, &p, v);
                match compute_results(s, &c.st) {
                    Ok(r) => {
                        let o = crate::sweep::outputs(&r);
                        out(Kind::Result, format!("  {:>12} {:>12} {:>10} {:>10} {:>10} {:>10.2} {:>8.2}", fmt(v), fmt(o[0]), fmt(o[2]), fmt(o[3]), fmt(o[5]), o[6], r.sat.no_load.b_rotor));
                    }
                    Err(e) => out(Kind::Error, format!("  {:>12} ✖ {e}", fmt(v))),
                }
            }
        }
        "compare" => {
            out(Kind::Result, format!("  {:<6} {:>12} {:>10} {:>10}", "", "cog mN·m", "F N", "FEM V"));
            for v in crate::repartition::variants() {
                if c.stop.load(Ordering::Relaxed) {
                    return Err(tr("arrêt demandé").into());
                }
                let mut s = c.spec.clone();
                s.variant = v.into();
                s.custom_angles_deg = None;
                crate::repartition::on_select(&mut s);
                match compute_results(s, &c.st) {
                    Ok(r) => {
                        let o = crate::sweep::outputs(&r);
                        out(Kind::Result, format!("  {v:<6} {:>12} {:>10} {:>10}", fmt(o[0]), fmt(o[2]), fmt(o[3])));
                    }
                    Err(e) => out(Kind::Error, format!("  {v:<6} ✖ {e}")),
                }
            }
        }
        "ia" | "ai" => {
            let goal = goal_of(&arg(1)?.to_lowercase()).ok_or_else(|| format!("{} : {} (cogging, force, fem, compromis, ondulation)", tr("objectif inconnu"), words[1]))?;
            let budget = words.get(2).and_then(|w| w.parse::<usize>().ok()).unwrap_or(120).clamp(10, 5000);
            let vars: Vec<Var> = match words.get(3) {
                Some(w) if *w != "sat" => w.split(',').map(|x| var_of(&pname(x)).ok_or_else(|| format!("{} : {x}", tr("variable inconnue")))).collect::<Result<_, _>>()?,
                _ => vec![Var::ArcRatio, Var::SkewAmp],
            };
            let sat_limit = words.iter().any(|w| *w == "sat");
            let setup = optimize::Setup { vars: vars.clone(), goal, budget, emf_keep: 0.9, sat_limit };
            out(Kind::Info, format!("{} : {} · {budget} {} · {}", tr("Ajustement I.A"), tr(goal.name()), tr("essais"), vars.iter().map(|v| short_of(*v)).collect::<Vec<_>>().join(", ")));
            let mut best: Option<optimize::Candidate> = None;
            optimize::execute(c.spec.clone(), setup, c.stop, &mut |m| match m {
                OptMsg::Trace(v) => {
                    for t in v.iter().filter(|t| t.new_best) {
                        out(Kind::Ia, trial_line(t, &vars));
                    }
                }
                OptMsg::Event(e) => out(Kind::Ia, event_line(&e, &vars)),
                OptMsg::Progress(_, _, top, _) => best = top.into_iter().find(|c| c.feasible),
                OptMsg::Done => {}
            });
            match best {
                Some(b) => {
                    c.spec = optimize::apply_candidate(&c.spec, &vars, &b);
                    let mut vals: Vec<String> = vars.iter().zip(&b.values).map(|(v, x)| format!("{}={}", short_of(*v), fmt(*x))).collect();
                    vals.push(format!("culasse_rotor={}", fmt(b.rotor_yoke_mm)));
                    out(Kind::Info, format!("{} : {} ({} « applique » {})", tr("Meilleure solution retenue dans le script"), vals.join(" "), tr("taper"), tr("pour la reporter dans le projet")));
                }
                None => out(Kind::Error, tr("aucune solution respectant les contraintes").into()),
            }
        }
        "applique" | "apply" => {
            spec_out(c.spec.clone());
            out(Kind::Info, tr("Projet du script reporté dans l'interface").into());
        }
        "initial" | "reset" => {
            c.spec = c.initial.clone();
            c.last = None;
            out(Kind::Info, tr("Retour au projet de départ du script").into());
        }
        _ => return Err(format!("{} : {head} ({} « aide »)", tr("commande inconnue"), tr("taper"))),
    }
    Ok(())
}

/// Texte de l'aide (clés de traduction).
const HELP: [&str; 13] = [
    "aide — cette liste",
    "variante <nom> — répartition des aimants (« reguliere » ou une répartition proposée)",
    "regle arc 0,8 — modifie un paramètre",
    "montre [arc] — affiche un ou tous les paramètres",
    "calcul — calcul complet et grandeurs principales",
    "ondulation — décomposition de l'ondulation du dernier calcul",
    "inductance — matrice d'inductances du dernier calcul",
    "balaye arc 0,6 0,9 7 — étude paramétrique",
    "compare — toutes les variantes avec les réglages du projet",
    "ia cogging 120 arc,amplitude [sat] — Ajustement I.A (cogging, force, fem, compromis, ondulation)",
    "applique — reporte le projet du script dans l'interface",
    "initial — revient au projet de départ du script",
    "# … — commentaire",
];

/// Exécute un script ; chaque ligne de sortie est passée à `send`.
pub fn execute(spec: MotorSpec, st: Settings, src: &str, stop: &AtomicBool, send: &mut dyn FnMut(ScriptMsg)) {
    let mut c = Ctx { initial: spec.clone(), spec, st, last: None, stop };
    let mut specs: Vec<MotorSpec> = Vec::new();
    for line in src.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        if stop.load(Ordering::Relaxed) {
            send(ScriptMsg::Line(Kind::Error, tr("arrêt demandé").into()));
            break;
        }
        send(ScriptMsg::Line(Kind::Cmd, format!("› {l}")));
        let r = {
            let mut out = |k: Kind, s: String| send(ScriptMsg::Line(k, s));
            cmd(&mut c, l, &mut out, &mut |s| specs.push(s))
        };
        for s in specs.drain(..) {
            send(ScriptMsg::Spec(Box::new(s)));
        }
        if let Err(e) = r {
            send(ScriptMsg::Line(Kind::Error, format!("✖ {e}")));
            break;
        }
    }
    send(ScriptMsg::Done(Box::new(c.spec)));
}

/// Lance un script en tâche de fond (fil sur ordinateur, Web Worker dans le navigateur).
pub fn run(spec: MotorSpec, st: Settings, src: String) -> ScriptJob {
    let (tx, rx) = channel();
    let stop = Arc::new(AtomicBool::new(false));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let stop2 = stop.clone();
        std::thread::spawn(move || {
            execute(spec, st, &src, &stop2, &mut |m| {
                let _ = tx.send(m);
            })
        });
        ScriptJob { rx, stop, worker: None }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let w = crate::web::worker(crate::bg::Request::Script(spec, st, src, crate::i18n::current()), move |r| {
            if let crate::bg::Reply::Script(m) = r {
                let _ = tx.send(m);
            }
        });
        ScriptJob { rx, stop, worker: Some(w) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(src: &str) -> (Vec<(Kind, String)>, Vec<MotorSpec>) {
        let st = Settings { n_cog: 90, n_emf: 90, ..Default::default() };
        let stop = AtomicBool::new(false);
        let (mut out, mut specs) = (Vec::new(), Vec::new());
        execute(MotorSpec::irregular_example(), st, src, &stop, &mut |m| match m {
            ScriptMsg::Line(k, s) => out.push((k, s)),
            ScriptMsg::Spec(s) => specs.push(*s),
            ScriptMsg::Done(_) => {}
        });
        (out, specs)
    }

    /// Un script complet s'exécute sans erreur et `applique` renvoie le projet modifié.
    #[test]
    fn script_complet() {
        let (out, specs) = lines("# essai\nvariante reguliere\nregle arc 0,8\nmontre arc\ncalcul\nondulation\ninductance\nbalaye entrefer 0,4 0,6 2\napplique");
        assert!(out.iter().all(|(k, _)| *k != Kind::Error), "{out:#?}");
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].variant, crate::repartition::variants()[0]);
        assert!((specs[0].magnet_arc_ratio - 0.8).abs() < 1e-12);
        assert!(out.iter().any(|(_, s)| s.contains("0.8000")));
    }

    /// Une commande inconnue arrête le script avec une erreur.
    #[test]
    fn erreur_arrete() {
        let (out, specs) = lines("regle arc 0.8\nfoo\napplique");
        assert_eq!(out.last().unwrap().0, Kind::Error);
        assert!(specs.is_empty());
    }

    /// L'Ajustement I.A par script est tracé et garde une solution faisable.
    #[test]
    fn ia_par_script() {
        let (out, specs) = lines("ia cogging 20 arc,ouverture\napplique");
        assert!(out.iter().any(|(k, s)| *k == Kind::Ia && s.contains('★')), "{out:#?}");
        assert!(out.iter().any(|(k, _)| *k == Kind::Ia));
        assert_eq!(specs.len(), 1);
    }
}
