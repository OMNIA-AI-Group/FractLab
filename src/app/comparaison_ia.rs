//! Comparaison des solutions de l'Ajustement I.A : le moteur original et
//! les propositions, chacun recalculé en calcul complet, côte à côte.
//! Tableau des métriques (écart à l'original, meilleure valeur marquée),
//! graphiques superposés, profil normalisé ; le chercheur applique la
//! solution de son choix ou garde le moteur original.

use super::*;
use crate::compute::ResultsWire;

/// Sens favorable d'une métrique.
#[derive(Clone, Copy, PartialEq)]
enum Better {
    Lower,
    Higher,
    None,
}

type Getter = fn(&ResultsWire) -> f64;

/// Métriques comparées : (nom, unité, valeur, sens favorable, dans le profil normalisé).
const METRICS: [(&str, &str, Getter, Better, bool); 18] = [
    ("Cogging crête-à-crête", "mN·m", |r| r.report.cogging.cogging_pp_mnm, Better::Lower, true),
    ("Cogging efficace", "mN·m", |r| r.report.cogging.cogging_rms_mnm, Better::Lower, false),
    ("Part de la dominante", "%", |r| r.report.cogging.dominant_share_pct, Better::Lower, true),
    ("Entropie spectrale", "", |r| r.report.cogging.spectral_entropy, Better::Higher, true),
    ("Force radiale max", "N", |r| r.report.force_max_n, Better::Lower, true),
    ("FEM crête", "V", |r| r.emf.emf_peak_v, Better::Higher, true),
    ("THD de la FEM", "%", |r| r.report.emf_thd_pct, Better::Lower, true),
    ("Couple moyen", "N·m", |r| r.load.avg_torque, Better::Higher, true),
    ("Ondulation", "%", |r| r.load.ripple_pct, Better::Lower, true),
    ("Constante de couple Kt", "N·m/A", |r| r.load.kt, Better::Higher, false),
    ("Inductance synchrone", "mH", |r| r.load.ls * 1e3, Better::None, false),
    ("Pertes Joule", "W", |r| r.load.p_joule, Better::Lower, true),
    ("Rendement (pertes Joule)", "%", |r| if r.load.p_mech > 0.0 { 100.0 * r.load.p_mech / (r.load.p_mech + r.load.p_joule) } else { 0.0 }, Better::Higher, false),
    ("Induction des dents (en charge)", "T", |r| r.sat.at_load.b_tooth, Better::Lower, false),
    ("Induction culasse stator (en charge)", "T", |r| r.sat.at_load.b_yoke, Better::Lower, false),
    ("Induction culasse rotor (en charge)", "T", |r| r.sat.at_load.b_rotor, Better::Lower, true),
    ("Écart minimal entre aimants", "°", |r| r.report.min_magnet_gap_deg, Better::Higher, false),
    ("Balourd", "mg·cm", |r| r.report.balourd_mg_cm, Better::Lower, false),
];

/// Graphique affiché.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CmpPlot {
    Cogging,
    Spectre,
    Fem,
    Forces,
    Charge,
    Profil,
}

/// Comparaison enregistrée dans l'historique (réglages, métriques, choix).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct CmpRecord {
    /// Date (secondes depuis 1970, UTC).
    when: u64,
    project: String,
    /// Objectif de l'ajustement (clé de traduction).
    goal: String,
    vars: Vec<crate::optimize::Var>,
    names: Vec<String>,
    specs: Vec<MotorSpec>,
    values: Vec<Vec<f64>>,
    scores: Vec<Option<f64>>,
    /// métriques de chaque solution, dans l'ordre de `METRICS`
    metrics: Vec<Vec<Option<f64>>>,
    /// solution retenue (0 = original gardé)
    chosen: Option<usize>,
}

/// Nombre de comparaisons gardées.
const HISTORY_MAX: usize = 40;

fn load_history() -> Vec<CmpRecord> {
    crate::platform::load_kv("comparaisons").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save_history(h: &[CmpRecord]) {
    crate::platform::save_kv("comparaisons", &serde_json::to_string(h).unwrap_or_default());
}

/// Date et heure de France (UTC+1, UTC+2 en heure d'été) : « jj/mm/aaaa hh:mm ».
fn date_fr(unix: u64) -> String {
    // jours → date civile (algorithme de Howard Hinnant)
    let civil = |days: i64| {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        (yoe + era * 400 + if m <= 2 { 1 } else { 0 }, m, d)
    };
    let (y, _, _) = civil(unix as i64 / 86_400);
    // dernier dimanche de mars et d'octobre, 1 h UTC
    let last_sunday = |month: i64| {
        let days_to = |yy: i64, mm: i64, dd: i64| {
            let (yy, mm) = if mm <= 2 { (yy - 1, mm + 12) } else { (yy, mm) };
            let era = yy.div_euclid(400);
            let yoe = yy - era * 400;
            let doy = (153 * (mm - 3) + 2) / 5 + dd - 1;
            era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
        };
        let d31 = days_to(y, month, 31);
        let wd = (d31 + 4).rem_euclid(7); // 0 = dimanche
        (d31 - wd) * 86_400 + 3_600
    };
    let t = unix as i64;
    let off = if t >= last_sunday(3) && t < last_sunday(10) { 7_200 } else { 3_600 };
    let lt = t + off;
    let (y, m, d) = civil(lt.div_euclid(86_400));
    let sec = lt.rem_euclid(86_400);
    format!("{d:02}/{m:02}/{y} {:02}:{:02}", sec / 3_600, sec % 3_600 / 60)
}

/// État de la fenêtre de comparaison.
#[derive(Default)]
pub(super) struct CmpIa {
    pub open: bool,
    names: Vec<String>,
    specs: Vec<MotorSpec>,
    /// valeurs des réglages libres de chaque solution (vide pour l'original)
    values: Vec<Vec<f64>>,
    vars: Vec<crate::optimize::Var>,
    scores: Vec<Option<f64>>,
    res: Vec<Option<ResultsWire>>,
    errs: Vec<Option<String>>,
    shown: Vec<bool>,
    job: Option<crate::compute::Job>,
    plot: Option<CmpPlot>,
    /// métriques de chaque solution (calculées ou relues dans l'historique)
    metrics: Vec<Vec<Option<f64>>>,
    when: u64,
    project: String,
    goal: String,
    /// historique (chargé à la première ouverture)
    history: Option<Vec<CmpRecord>>,
    /// rang de la comparaison affichée dans l'historique
    rec: Option<usize>,
    /// recalcul des graphiques d'une comparaison relue (pas de nouvel enregistrement)
    replay: bool,
}

fn col(k: usize) -> Color32 {
    if k == 0 { Color32::from_gray(200) } else { SERIES[(k - 1) % SERIES.len()] }
}

impl CmpIa {
    /// Aucune comparaison affichée (ni calculée ni relue) ?
    pub(super) fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    /// Nouvel ajustement : la comparaison affichée est oubliée, l'historique reste.
    pub(super) fn reset_keep_history(&mut self) {
        let history = self.history.take();
        let open = self.open;
        let _ = open;
        *self = CmpIa { history, ..Default::default() };
    }
}

impl App {
    /// Prépare et lance la comparaison : original + 5 meilleures propositions
    /// (+ l'essai choisi dans le graphe 3D s'il n'en fait pas partie).
    pub(super) fn cmpia_start(&mut self) {
        let lang = self.cfg.lang;
        let Some(base) = self.opt_base.clone() else { return };
        let vars = self.opt_vars_run.clone();
        let first = match std::env::var("FRACTLAB_CMPPLOT").as_deref() {
            Ok("profil") => Some(CmpPlot::Profil),
            Ok("spectre") => Some(CmpPlot::Spectre),
            _ => None,
        };
        let history = self.cmpia.history.take();
        let mut c = CmpIa { open: true, vars: vars.clone(), plot: first, history, ..Default::default() };
        c.when = web_time::SystemTime::now().duration_since(web_time::SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        c.project = base.name.clone();
        c.goal = self.opt_goal.name().to_string();
        c.names.push(T(lang, "Original").to_string());
        c.specs.push(base.clone());
        c.values.push(vars.iter().map(|v| v.get(&base)).collect());
        c.scores.push(None);
        for (k, cand) in self.opt_top.iter().enumerate() {
            c.names.push(format!("{} {}", T(lang, "Proposition"), k + 1));
            c.specs.push(crate::optimize::apply_candidate(&base, &vars, cand));
            c.values.push(cand.values.clone());
            c.scores.push(Some(cand.score));
        }
        if let Some(n) = self.console.graph.selected
            && let Some(t) = self.console.ia.iter().find_map(|e| if let super::console::IaLog::Tr(t) = e && t.n == n { Some(t.clone()) } else { None })
            && !self.opt_top.iter().any(|c| c.values == t.cand.values)
        {
            c.names.push(format!("{} #{n}", T(lang, "Essai")));
            c.specs.push(crate::optimize::apply_candidate(&base, &vars, &t.cand));
            c.values.push(t.cand.values.clone());
            c.scores.push(Some(t.cand.score));
        }
        let n = c.specs.len();
        c.res = vec![None; n];
        c.metrics = vec![vec![None; METRICS.len()]; n];
        c.errs = vec![None; n];
        c.shown = vec![true; n];
        c.job = Some(crate::compute::launch(crate::compute::Request::Solutions(c.specs.clone(), self.st)));
        self.cmpia = c;
    }

    fn cmpia_poll(&mut self, ctx: &egui::Context) {
        let mut done = false;
        if let Some(j) = &self.cmpia.job {
            while let Ok(m) = j.rx.try_recv() {
                match m {
                    Msg::Solution(k, w) => {
                        if let Some(m) = self.cmpia.metrics.get_mut(k) {
                            *m = METRICS.iter().map(|x| Some(x.2(&w))).collect();
                        }
                        if let Some(r) = self.cmpia.res.get_mut(k) {
                            *r = Some(*w);
                        }
                    }
                    Msg::SolutionError(k, e) => {
                        if let Some(r) = self.cmpia.errs.get_mut(k) {
                            *r = Some(e);
                        }
                    }
                    Msg::SolutionsDone => done = true,
                    _ => {}
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if done {
            self.cmpia.job = None;
            if !self.cmpia.replay {
                self.cmpia_record();
            }
            self.cmpia.replay = false;
        }
    }

    /// Enregistre la comparaison affichée dans l'historique.
    fn cmpia_record(&mut self) {
        let c = &mut self.cmpia;
        let rec = CmpRecord {
            when: c.when,
            project: c.project.clone(),
            goal: c.goal.clone(),
            vars: c.vars.clone(),
            names: c.names.clone(),
            specs: c.specs.clone(),
            values: c.values.clone(),
            scores: c.scores.clone(),
            metrics: c.metrics.clone(),
            chosen: None,
        };
        let h = c.history.get_or_insert_with(load_history);
        h.insert(0, rec);
        h.truncate(HISTORY_MAX);
        c.rec = Some(0);
        save_history(h);
    }

    /// Affiche une comparaison de l'historique (tableau et profil tout de
    /// suite ; les courbes se recalculent à la demande).
    fn cmpia_load(&mut self, i: usize) {
        let c = &mut self.cmpia;
        let Some(r) = c.history.as_ref().and_then(|h| h.get(i)).cloned() else { return };
        let n = r.specs.len();
        c.names = r.names;
        c.specs = r.specs;
        c.values = r.values;
        c.vars = r.vars;
        c.scores = r.scores;
        c.metrics = r.metrics;
        c.when = r.when;
        c.project = r.project;
        c.goal = r.goal;
        c.res = vec![None; n];
        c.errs = vec![None; n];
        c.shown = vec![true; n];
        c.job = None;
        c.rec = Some(i);
        c.replay = false;
        // relue : les courbes ne sont pas gardées, le profil normalisé l'est
        c.plot = Some(CmpPlot::Profil);
    }

    /// Tableau CSV des métriques (séparateur « ; »).
    fn cmpia_csv(&self) -> String {
        let c = &self.cmpia;
        let mut s = String::from("Grandeur;Unité");
        for n in &c.names {
            s += &format!(";{n}");
        }
        s.push('\n');
        for v in &c.vars {
            s += &format!("{};", v.name());
            for x in &c.values {
                s += &format!(";{}", x.get(c.vars.iter().position(|w| w == v).unwrap_or(0)).copied().unwrap_or(f64::NAN));
            }
            s.push('\n');
        }
        for (mi, (name, unit, _, _, _)) in METRICS.iter().enumerate() {
            s += &format!("{name};{unit}");
            for m in &c.metrics {
                s += &format!(";{}", m.get(mi).copied().flatten().map(|v| v.to_string()).unwrap_or_default());
            }
            s.push('\n');
        }
        s
    }

    pub(super) fn cmpia_window(&mut self, ctx: &egui::Context) {
        self.cmpia_poll(ctx);
        if !self.cmpia.open {
            return;
        }
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let u = self.cfg.units.clone();
        let mut open = true;
        let mut apply: Option<usize> = None;
        let mut export = false;
        let mut load_i: Option<usize> = None;
        let mut recompute = false;
        let mut forget = false;
        let mut open_ia = false;
        if self.cmpia.history.is_none() {
            self.cmpia.history = Some(load_history());
        }
        let sr = ctx.screen_rect();
        let size = [(sr.width() * 0.94).clamp(900.0, 3000.0), (sr.height() * 0.88).clamp(600.0, 1800.0)];
        egui::Window::new(format!("⚖ {}", t("Comparaison des solutions"))).open(&mut open).default_size(size).default_pos([sr.left() + sr.width() * 0.03, sr.top() + 30.0]).resizable(true).show(ctx, |ui| {
            let c = &mut self.cmpia;
            let n = c.specs.len();
            let ready = c.res.iter().filter(|r| r.is_some()).count();
            ui.label(RichText::new(t("Chaque solution est recalculée en calcul complet (et non avec la résolution allégée de l'ajustement). Écarts en % par rapport au moteur original ; ★ = meilleure valeur de la ligne ; souligné = meilleur score selon l'objectif de l'ajustement.")).weak());
            // historique
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(format!("🕘 {}", t("Historique"))).strong());
                let h = c.history.as_deref().unwrap_or(&[]);
                let label = |r: &CmpRecord| {
                    let choix = match r.chosen {
                        Some(0) => format!(" · {}", t("original gardé")),
                        Some(k) => format!(" · {} {}", t("choix :"), r.names.get(k).cloned().unwrap_or_default()),
                        None => String::new(),
                    };
                    format!("{} — {} — {}{choix}", date_fr(r.when), r.project, t_dyn(lang, &r.goal))
                };
                let cur = c.rec.and_then(|i| h.get(i)).map(label).unwrap_or_else(|| t("comparaison en cours").to_string());
                egui::ComboBox::from_id_source("cmpia_hist").width(520.0).selected_text(cur).show_ui(ui, |ui| {
                    for (i, r) in h.iter().enumerate() {
                        if ui.selectable_label(c.rec == Some(i), label(r)).clicked() {
                            load_i = Some(i);
                        }
                    }
                });
                if c.rec.is_some() && c.job.is_none() && ui.button(format!("🗑 {}", t("Retirer de l'historique"))).clicked() {
                    forget = true;
                }
                if c.job.is_none() && c.res.iter().all(|r| r.is_none()) && c.metrics.iter().any(|m| m.iter().any(|v| v.is_some())) && ui.button(format!("⟳ {}", t("Recalculer les courbes"))).on_hover_text(t("L'historique garde le tableau et le profil ; les courbes se recalculent à la demande.")).clicked() {
                    recompute = true;
                }
            });
            ui.horizontal_wrapped(|ui| {
                if c.job.is_some() {
                    ui.spinner();
                    ui.label(format!("{} {ready} / {n}", t("calcul des solutions")));
                }
                for k in 0..n {
                    let txt = RichText::new(format!("■ {}", c.names[k])).color(col(k)).strong();
                    ui.checkbox(&mut c.shown[k], txt);
                }
                ui.separator();
                if ui.button(format!("📋 {}", t("Copier le tableau (CSV)"))).clicked() {
                    export = true;
                }
            });
            if n == 0 {
                // rien à comparer encore : expliquer et proposer l'Ajustement I.A
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t("Aucune comparaison à afficher")).size(20.0).strong());
                    ui.add_space(6.0);
                    ui.label(t("Lancez un Ajustement I.A : à la fin, cette fenêtre s'ouvre seule avec le moteur original et les meilleures propositions, métriques et graphiques côte à côte."));
                    ui.label(RichText::new(t("Ou choisissez une comparaison précédente dans l'historique ci-dessus.")).weak());
                    ui.add_space(10.0);
                    if ui.add(egui::Button::new(RichText::new(format!("✨ {}", t("Lancer un Ajustement I.A"))).strong().size(16.0).color(Color32::WHITE)).fill(Color32::from_rgb(140, 70, 0)).min_size(egui::vec2(0.0, 34.0))).clicked() {
                        open_ia = true;
                    }
                });
                return;
            }
            let dark = self.cfg.theme == Theme::Sombre;
            let avail = ui.available_size();
            let left_w = (avail.x * 0.47).clamp(480.0, 1500.0);
            let tq = u.factor(Q::Torque);
            let ff = u.factor(Q::Force);
            let base: Vec<Option<f64>> = c.metrics.first().cloned().unwrap_or_default();
            ui.horizontal_top(|ui| {
                // ================= métriques (gauche) =================
                ui.allocate_ui_with_layout(egui::vec2(left_w, avail.y), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_min_size(egui::vec2(left_w, avail.y));
                    ui.label(RichText::new(format!("📊 {}", t("Métriques"))).strong().size(15.0));
                    egui::ScrollArea::both().id_source("cmpia_table").auto_shrink([false, false]).show(ui, |ui| {
                // ---------------- tableau des métriques ----------------
                egui::Grid::new("cmpia_grid").striped(true).num_columns(n + 2).spacing([12.0, 2.0]).show(ui, |ui| {
                    ui.label(RichText::new(t("Grandeur")).strong());
                    ui.label("");
                    for k in 0..n {
                        let mut h = RichText::new(&c.names[k]).color(col(k)).strong();
                        if k == 1 {
                            h = h.underline();
                        }
                        ui.label(h).on_hover_text(c.scores[k].map(|s| format!("{} {s:.4}", t("score"))).unwrap_or_default());
                    }
                    ui.end_row();
                    // réglages libres
                    for (vi, v) in c.vars.iter().enumerate() {
                        ui.label(RichText::new(t(v.name())).italics());
                        ui.label("");
                        for k in 0..n {
                            ui.label(RichText::new(c.values[k].get(vi).map(|x| fnum(*x)).unwrap_or_default()).monospace().italics());
                        }
                        ui.end_row();
                    }
                    for (mi, &(name, unit, _, better, _)) in METRICS.iter().enumerate() {
                        let vals: Vec<Option<f64>> = c.metrics.iter().map(|m| m.get(mi).copied().flatten()).collect();
                        let best = match better {
                            Better::None => None,
                            _ => vals.iter().enumerate().filter_map(|(k, v)| v.map(|v| (k, v))).fold(None, |acc: Option<(usize, f64)>, (k, v)| match acc {
                                None => Some((k, v)),
                                Some((bk, bv)) => {
                                    let better_here = if better == Better::Lower { v < bv - 1e-12 } else { v > bv + 1e-12 };
                                    Some(if better_here { (k, v) } else { (bk, bv) })
                                }
                            }),
                        };
                        ui.label(t(name));
                        ui.label(RichText::new(unit).weak());
                        for k in 0..n {
                            match (vals[k], &c.errs[k]) {
                                (Some(v), _) => {
                                    let b0 = base.get(mi).copied().flatten();
                                    let txt = fnum(v);
                                    let mut delta = String::new();
                                    let mut color = Color32::GRAY;
                                    if k > 0
                                        && let Some(b0) = b0
                                        && b0.abs() > 1e-12
                                    {
                                        // écart relatif, sauf pour une référence négative ou un angle : écart absolu
                                        let absolute = b0 <= 0.0 || unit == "°";
                                        let d = if absolute { v - b0 } else { 100.0 * (v - b0) / b0.abs() };
                                        delta = if absolute { format!("Δ {d:+.3} {unit}") } else { format!("{d:+.1} %") };
                                        let eps = if absolute { 1e-6 } else { 0.05 };
                                        let good = match better {
                                            Better::Lower => d < -eps,
                                            Better::Higher => d > eps,
                                            Better::None => false,
                                        };
                                        let bad = match better {
                                            Better::Lower => d > eps,
                                            Better::Higher => d < -eps,
                                            Better::None => false,
                                        };
                                        color = if good { badge(true) } else if bad { badge(false) } else { Color32::GRAY };
                                    } else if k == 0 {
                                        color = if dark { Color32::from_gray(220) } else { Color32::from_gray(30) };
                                    }
                                    let star = best.is_some_and(|(bk, _)| bk == k);
                                    let mut rt = RichText::new(if star { format!("★ {txt}") } else { txt }).monospace().color(if k == 0 { color } else if dark { Color32::from_gray(215) } else { Color32::from_gray(35) });
                                    if star {
                                        rt = rt.strong();
                                    }
                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing.y = 0.0;
                                        ui.label(rt);
                                        if !delta.is_empty() {
                                            ui.label(RichText::new(delta).small().monospace().color(color));
                                        }
                                    });
                                }
                                (None, Some(e)) => {
                                    ui.label(RichText::new("✖").color(badge(false))).on_hover_text(e);
                                }
                                (None, None) => {
                                    if c.job.is_some() {
                                        ui.spinner();
                                    } else {
                                        ui.label("—");
                                    }
                                }
                            }
                        }
                        ui.end_row();
                    }
                    // décision
                    ui.label("");
                    ui.label("");
                    for k in 0..n {
                        let (label, fill) = if k == 0 { (format!("↺ {}", t("Garder l'original")), Color32::from_rgb(70, 70, 80)) } else { (format!("✔ {}", t("Appliquer")), Color32::from_rgb(0, 110, 160)) };
                        if ui.add_enabled(c.metrics[k].iter().any(|v| v.is_some()), egui::Button::new(RichText::new(label).strong().color(Color32::WHITE)).fill(fill)).clicked() {
                            apply = Some(k);
                        }
                    }
                    ui.end_row();
                });
                    });
                });
                ui.separator();
                // ================= graphiques (droite) =================
                let rw = ui.available_width();
                ui.allocate_ui_with_layout(egui::vec2(rw, avail.y), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    let zoom = c.plot;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("📈 {}", t("Graphiques"))).strong().size(15.0));
                        if zoom.is_some() {
                            if ui.button(format!("⊞ {}", t("Tous les graphiques"))).clicked() {
                                c.plot = None;
                            }
                        } else {
                            ui.label(RichText::new(t("cliquer un titre pour agrandir")).weak().small());
                        }
                    });
                    let cr = &*c;
                    let draw = |ui: &mut egui::Ui, sel: CmpPlot, h: f32, big: bool| {
                        let shown: Vec<usize> = (0..n).filter(|k| cr.shown[*k] && (cr.res[*k].is_some() || (sel == CmpPlot::Profil && cr.metrics[*k].iter().any(|v| v.is_some())))).collect();
                        if shown.is_empty() {
                            let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), h), egui::Sense::hover());
                            ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, if cr.job.is_some() { t("calcul des solutions") } else { t("Recalculer les courbes") }, egui::FontId::proportional(13.0), Color32::GRAY);
                            return;
                        }
                        let id = format!("cmpia_{}_{}", sel as u8, big);
                        let legend = if big { Some(Legend::default().position(Corner::RightTop)) } else { None };
                        if sel == CmpPlot::Profil {
                            let rows: Vec<(usize, &str)> = METRICS.iter().enumerate().filter(|(_, m)| m.4).map(|(i, m)| (i, m.0)).collect();
                            let nb = shown.len().max(1) as f64;
                            let mut charts = Vec::new();
                            for (si, &k) in shown.iter().enumerate() {
                                let bars: Vec<Bar> = rows
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(ri, (mi, name))| {
                                        let b0 = base.get(*mi).copied().flatten()?;
                                        if b0.abs() < 1e-12 {
                                            return None;
                                        }
                                        let v = 100.0 * cr.metrics[k].get(*mi).copied().flatten()? / b0;
                                        // métriques de haut en bas dans l'ordre du tableau
                                        let y = (rows.len() - 1 - ri) as f64 - (si as f64 - (nb - 1.0) / 2.0) * 0.8 / nb;
                                        Some(Bar::new(y, v).width(0.8 / nb).name(format!("{} — {}", cr.names[k], t(name))))
                                    })
                                    .collect();
                                charts.push(BarChart::new(bars).horizontal().color(col(k)).name(&cr.names[k]));
                            }
                            let nr = rows.len();
                            let labels: Vec<String> = rows.iter().map(|(_, n)| t(n).to_string()).collect();
                            let mut p = Plot::new(id).height(h).x_axis_label(t("% de l'original")).y_axis_width(5).y_axis_formatter(move |m, _, _| {
                                let i = m.value.round();
                                if (m.value - i).abs() < 0.01 && i >= 0.0 && (i as usize) < nr { labels.get(nr - 1 - i as usize).cloned().unwrap_or_default() } else { String::new() }
                            });
                            if let Some(l) = legend {
                                p = p.legend(l);
                            }
                            p.show(ui, |p| {
                                p.vline(egui_plot::VLine::new(100.0).color(Color32::GRAY));
                                for ch in charts {
                                    p.bar_chart(ch);
                                }
                            });
                            return;
                        }
                        // spectre : raies significatives seulement (> 1 % de la plus forte), 60 au plus
                        let spec_n = shown
                            .iter()
                            .filter_map(|&k| cr.res[k].as_ref())
                            .map(|r| {
                                let mx = r.cog.spectrum.iter().skip(1).cloned().fold(0.0_f64, f64::max);
                                r.cog.spectrum.iter().enumerate().skip(1).filter(|(_, v)| **v > 0.01 * mx).map(|(i, _)| i).max().unwrap_or(1)
                            })
                            .max()
                            .unwrap_or(1)
                            .clamp(4, 60);
                        let (xl, yl) = match sel {
                            CmpPlot::Cogging => ("θr [°]".to_string(), format!("{} [{}]", t("couple de détente"), u.sym(Q::Torque))),
                            CmpPlot::Spectre => (t("ordre par tour").to_string(), format!("{} [{}]", t("amplitude"), u.sym(Q::Torque))),
                            CmpPlot::Fem => ("θr [°]".to_string(), "FEM [V]".to_string()),
                            CmpPlot::Forces => ("θr [°]".to_string(), format!("|F| [{}]", u.sym(Q::Force))),
                            _ => ("θr [°]".to_string(), format!("{} [{}]", t("couple"), u.sym(Q::Torque))),
                        };
                        let mut p = Plot::new(id).height(h).x_axis_label(xl).y_axis_label(yl);
                        if let Some(l) = legend {
                            p = p.legend(l);
                        }
                        p.show(ui, |p| {
                            for &k in &shown {
                                let r = cr.res[k].as_ref().unwrap();
                                let pts: Vec<[f64; 2]> = match sel {
                                    CmpPlot::Cogging => r.cog.angles_deg.iter().zip(&r.cog.torque_nm).map(|(a, v)| [*a, v * tq]).collect(),
                                    CmpPlot::Spectre => {
                                        let span = r.cog.metrics.span_deg.max(1e-9);
                                        r.cog.spectrum.iter().enumerate().skip(1).take(spec_n).map(|(i, v)| [i as f64 * 360.0 / span, v * tq]).collect()
                                    }
                                    CmpPlot::Fem => r.emf.angles_deg.iter().zip(&r.emf.emf[0]).map(|(a, v)| [*a, *v]).collect(),
                                    CmpPlot::Forces => r.forces.angles_deg.iter().zip(r.forces.fx.iter().zip(&r.forces.fy)).map(|(a, (x, y))| [*a, x.hypot(*y) * ff]).collect(),
                                    _ => r.load.angles_deg.iter().zip(&r.load.torque).map(|(a, v)| [*a, v * tq]).collect(),
                                };
                                if sel == CmpPlot::Spectre {
                                    p.points(Points::new(PlotPoints::from(pts.clone())).radius(2.5f32).color(col(k)).name(&cr.names[k]));
                                    for q in pts {
                                        p.line(Line::new(PlotPoints::from(vec![[q[0], 0.0], q])).color(col(k).gamma_multiply(0.6)).width(1.0f32));
                                    }
                                } else {
                                    p.line(Line::new(PlotPoints::from(pts)).color(col(k)).width(if k == 0 { 2.5f32 } else { 1.6f32 }).name(&cr.names[k]));
                                }
                            }
                        });
                    };
                    const PLOTS: [(CmpPlot, &str); 6] = [
                        (CmpPlot::Cogging, "Couple de détente"),
                        (CmpPlot::Spectre, "Spectre du couple de détente"),
                        (CmpPlot::Fem, "FEM phase A"),
                        (CmpPlot::Forces, "Force sur le rotor"),
                        (CmpPlot::Charge, "Couple en charge"),
                        (CmpPlot::Profil, "Profil normalisé (original = 100 %)"),
                    ];
                    let mut clicked: Option<CmpPlot> = None;
                    match zoom {
                        Some(sel) => {
                            let title = PLOTS.iter().find(|p| p.0 == sel).map(|p| p.1).unwrap_or("");
                            ui.label(RichText::new(t(title)).strong());
                            let h = (ui.available_height() - 8.0).max(200.0);
                            draw(ui, sel, h, true);
                        }
                        None => {
                            // grille 2 × 3 : tous les graphiques visibles en même temps
                            let head = ui.text_style_height(&egui::TextStyle::Body) + 8.0;
                            let h = ((ui.available_height() - 3.0 * head - 16.0) / 3.0).max(110.0);
                            ui.columns(2, |cols| {
                                for (i, (sel, title)) in PLOTS.iter().enumerate() {
                                    let ui = &mut cols[i % 2];
                                    if ui.add(egui::Label::new(RichText::new(format!("🔍 {}", t(title))).strong()).sense(egui::Sense::click())).on_hover_text(t("Agrandir")).clicked() {
                                        clicked = Some(*sel);
                                    }
                                    draw(ui, *sel, h, false);
                                }
                            });
                        }
                    }
                    if let Some(p) = clicked {
                        c.plot = Some(p);
                    }
                });
            });
        });
        if open_ia {
            self.show_opt = true;
        }
        if let Some(i) = load_i {
            self.cmpia_load(i);
        }
        if recompute {
            self.cmpia.job = Some(crate::compute::launch(crate::compute::Request::Solutions(self.cmpia.specs.clone(), self.st)));
            self.cmpia.replay = true;
        }
        if forget
            && let (Some(i), Some(h)) = (self.cmpia.rec, self.cmpia.history.as_mut())
            && i < h.len()
        {
            h.remove(i);
            save_history(h);
            self.cmpia.rec = None;
        }
        if export {
            let csv = self.cmpia_csv();
            ctx.output_mut(|o| o.copied_text = csv);
            self.status = t("Tableau copié dans le presse-papiers").into();
        }
        if let Some(k) = apply {
            if let (Some(i), Some(h)) = (self.cmpia.rec, self.cmpia.history.as_mut())
                && let Some(r) = h.get_mut(i)
            {
                r.chosen = Some(k);
                save_history(h);
            }
            if let Some(s) = self.cmpia.specs.get(k).cloned() {
                self.opt_live = false;
                self.spec = s;
                self.touched(ctx);
                self.status = if k == 0 { t("Moteur original conservé").to_string() } else { format!("{} : {}", t("Solution appliquée"), self.cmpia.names[k]) };
            }
            open = false;
        }
        self.cmpia.open = open;
    }
}

/// Traduction d'une clé connue seulement à l'exécution (objectif enregistré).
fn t_dyn(lang: crate::i18n::Lang, k: &str) -> String {
    crate::optimize::ALL_GOALS.iter().find(|g| g.name() == k).map(|g| T(lang, g.name()).to_string()).unwrap_or_else(|| k.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Heure de France : UTC+1 en hiver, UTC+2 en été (changements le dernier dimanche de mars et d'octobre).
    #[test]
    fn heure_de_france() {
        assert_eq!(date_fr(1_795_608_000), "25/11/2026 13:00"); // 25/11/2026 12:00 UTC
        assert_eq!(date_fr(1_782_907_200), "01/07/2026 14:00"); // 01/07/2026 12:00 UTC
        assert_eq!(date_fr(1_774_746_000), "29/03/2026 03:00"); // passage à l'heure d'été (01:00 UTC)
        assert_eq!(date_fr(1_774_745_999), "29/03/2026 01:59");
    }
}
