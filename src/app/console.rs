//! Console : déroulé en direct du moteur I.A (chaque essai de l'Ajustement
//! I.A, étapes du simplexe, contraintes violées, nouveaux meilleurs, bilan de
//! la résolution) et console de script (commandes de `crate::script`).

use super::*;
use crate::optimize::{Candidate, Event, Trial, Var};
use crate::script::{Kind, ScriptMsg};

/// Une entrée du journal du moteur I.A.
pub(super) enum IaLog {
    /// Événement de la résolution.
    Ev(Event),
    /// Essai.
    Tr(Trial),
}

/// Onglet de la console.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConsoleTab {
    /// Moteur I.A.
    Ia,
    /// Script.
    Script,
}

/// Filtre du journal du moteur I.A.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum IaFilter {
    /// Tout.
    All,
    /// Nouveaux meilleurs et événements.
    Best,
    /// Essais rejetés (contraintes violées).
    Rejected,
}

/// État de la console.
pub(super) struct Console {
    pub open: bool,
    pub tab: ConsoleTab,
    pub ia: Vec<IaLog>,
    pub ia_vars: Vec<Var>,
    pub ia_ref: Option<Candidate>,
    pub ia_filter: IaFilter,
    pub follow: bool,
    pub src: String,
    pub cmd: String,
    pub hist: Vec<String>,
    pub hist_pos: usize,
    pub out: Vec<(Kind, String)>,
    pub job: Option<crate::script::ScriptJob>,
    /// graphe 3D des essais
    pub graph: super::graphe_ia::GraphView,
    /// projet de la ligne de commande (repris d'une commande à l'autre)
    pub session: Option<MotorSpec>,
}

const EXAMPLE: &str = "# Exemple : calculer le projet, décomposer l'ondulation, puis l'optimiser
calcul
ondulation
# effet des becs de dents
balaye becs 0 0,6 4
# Ajustement I.A tracé : cogging minimal, culasse rotor non saturée
ia cogging 120 arc,ouverture sat
calcul
# applique   ← enlever le « # » pour reporter le résultat dans le projet
";

/// Taille maximale des journaux (les plus anciennes lignes partent).
const MAX_LINES: usize = 20000;

impl Default for Console {
    fn default() -> Self {
        Console {
            open: std::env::var("FRACTLAB_CONSOLE").is_ok(),
            tab: if std::env::var("FRACTLAB_CONSOLE").as_deref() == Ok("script") { ConsoleTab::Script } else { ConsoleTab::Ia },
            ia: Vec::new(),
            ia_vars: Vec::new(),
            ia_ref: None,
            ia_filter: IaFilter::All,
            follow: true,
            src: crate::platform::load_kv("script").unwrap_or_else(|| EXAMPLE.to_string()),
            cmd: String::new(),
            hist: Vec::new(),
            hist_pos: 0,
            out: Vec::new(),
            job: None,
            session: None,
            graph: Default::default(),
        }
    }
}

fn kind_color(k: Kind, dark: bool) -> Color32 {
    match k {
        Kind::Cmd => if dark { CYAN } else { Color32::from_rgb(0, 100, 160) },
        Kind::Info => if dark { Color32::from_gray(190) } else { Color32::from_gray(70) },
        Kind::Result => if dark { Color32::from_rgb(140, 230, 140) } else { Color32::from_rgb(20, 120, 40) },
        Kind::Error => Color32::from_rgb(230, 70, 70),
        Kind::Ia => if dark { Color32::from_rgb(230, 170, 255) } else { Color32::from_rgb(120, 40, 160) },
    }
}

fn trim<T>(v: &mut Vec<T>) {
    if v.len() > MAX_LINES {
        let n = v.len() - MAX_LINES;
        v.drain(..n);
    }
}

impl App {
    /// Nouvel ajustement : le journal du moteur I.A repart de zéro.
    pub(super) fn console_ia_start(&mut self, vars: &[Var]) {
        self.console.ia.clear();
        self.console.ia_ref = None;
        self.console.ia_vars = vars.to_vec();
        self.console.graph = Default::default();
        // le graphe des possibilités s'ouvre à chaque lancement
        self.console.open = true;
        self.console.tab = ConsoleTab::Ia;
    }

    /// Reçoit les essais et événements de l'ajustement.
    pub(super) fn console_ia_push(&mut self, e: IaLog) {
        if let IaLog::Ev(Event::Reference(c)) = &e {
            self.console.ia_ref = Some(c.clone());
        }
        self.console.ia.push(e);
        trim(&mut self.console.ia);
    }

    /// Relève les messages du script en cours.
    pub(super) fn poll_script(&mut self, ctx: &egui::Context) {
        let mut done = false;
        let mut specs = Vec::new();
        if let Some(j) = &self.console.job {
            while let Ok(m) = j.rx.try_recv() {
                match m {
                    ScriptMsg::Line(k, s) => self.console.out.push((k, s)),
                    ScriptMsg::Spec(s) => specs.push(*s),
                    ScriptMsg::Done(s) => {
                        self.console.session = Some(*s);
                        done = true;
                    }
                }
            }
            trim(&mut self.console.out);
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        for s in specs {
            self.spec = s;
            self.touched(ctx);
        }
        if done {
            self.console.job = None;
        }
    }

    /// Lance un script : le script complet part du projet de l'interface,
    /// une commande isolée repart du projet laissé par la précédente.
    fn run_script(&mut self, src: String, whole: bool) {
        if self.console.job.is_some() {
            return;
        }
        crate::platform::save_kv("script", &self.console.src);
        let start = if whole { None } else { self.console.session.clone() }.unwrap_or_else(|| self.spec.clone());
        self.console.job = Some(crate::script::run(start, self.st, src));
    }

    pub(super) fn console_window(&mut self, ctx: &egui::Context) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let mut open = self.console.open;
        egui::Window::new(format!("🖥 {}", t("Console"))).open(&mut open).default_size([960.0, 780.0]).resizable(true).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let running_ia = self.opt_job.is_some();
                let ia_lbl = format!("🧠 {}{}", t("Moteur I.A"), if running_ia { " ●" } else { "" });
                ui.selectable_value(&mut self.console.tab, ConsoleTab::Ia, ia_lbl);
                let sc_lbl = format!("⌨ {}{}", t("Script"), if self.console.job.is_some() { " ●" } else { "" });
                ui.selectable_value(&mut self.console.tab, ConsoleTab::Script, sc_lbl);
            });
            ui.separator();
            match self.console.tab {
                ConsoleTab::Ia => {
                    egui::ScrollArea::vertical().id_source("ia_tab").auto_shrink([false, false]).show(ui, |ui| self.console_ia(ui));
                }
                ConsoleTab::Script => self.console_script(ui),
            }
        });
        self.console.open = open;
    }

    fn console_ia(&mut self, ui: &mut egui::Ui) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let dark = self.cfg.theme == Theme::Sombre;
        let running = self.opt_job.is_some();
        // en-tête : état de la résolution
        let trials: Vec<&Trial> = self.console.ia.iter().filter_map(|e| if let IaLog::Tr(t) = e { Some(t) } else { None }).collect();
        let n_rej = trials.iter().filter(|t| !t.violations.is_empty()).count();
        let ms = if trials.is_empty() { 0.0 } else { trials.iter().map(|t| t.ms).sum::<f64>() / trials.len() as f64 };
        let starts = self.console.ia.iter().filter(|e| matches!(e, IaLog::Ev(Event::Start(..)))).count();
        ui.horizontal_wrapped(|ui| {
            if running {
                ui.spinner();
                ui.label(RichText::new(t("Résolution en cours")).strong().color(ORANGE));
            } else if trials.is_empty() {
                ui.label(RichText::new(t("Aucune résolution : lancer l'Ajustement I.A ou la commande « ia » du script.")).weak());
            } else {
                ui.label(RichText::new(t("Résolution terminée")).strong());
            }
            if !trials.is_empty() {
                ui.label(format!("· {} {} / {} · {} {} · {} {} · {:.0} ms / {}", trials.len(), t("essais"), self.opt_prog.1.max(trials.len()), starts, t("départs"), n_rej, t("rejetés"), ms, t("essai")));
            }
        });
        // bilan : départ → meilleure solution faisable
        let best = trials.iter().filter(|t| t.cand.feasible).min_by(|a, b| a.cand.score.total_cmp(&b.cand.score));
        if let (Some(r), Some(b)) = (&self.console.ia_ref, best) {
            let c = &b.cand;
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(RichText::new(format!("🏁 {} (#{})", t("Meilleure solution faisable"), b.n)).strong());
                egui::Grid::new("ia_bilan").num_columns(4).spacing([18.0, 2.0]).show(ui, |ui| {
                    for h in [t("Grandeur"), t("Départ"), t("Meilleure"), t("Évolution")] {
                        ui.label(RichText::new(h).small().strong());
                    }
                    ui.end_row();
                    let rows: [(&str, f64, f64, &str, bool); 6] = [
                        ("Cogging pp", r.cogging_mnm, c.cogging_mnm, "mN·m", true),
                        ("Force max", r.force_n, c.force_n, "N", true),
                        ("FEM crête", r.emf_v, c.emf_v, "V", false),
                        ("Couple moyen", r.torque_nm, c.torque_nm, "N·m", false),
                        ("Ondulation", r.ripple_pct, c.ripple_pct, "%", true),
                        ("culasse rotor", r.b_rotor_t, c.b_rotor_t, "T", true),
                    ];
                    for (n, a, v, u, lower_better) in rows {
                        ui.label(t(n));
                        ui.label(RichText::new(format!("{} {u}", fnum(a))).monospace());
                        ui.label(RichText::new(format!("{} {u}", fnum(v))).monospace());
                        let d = if a.abs() > 1e-12 { 100.0 * (v - a) / a.abs() } else { 0.0 };
                        let good = if lower_better { d <= 0.0 } else { d >= 0.0 };
                        ui.label(RichText::new(format!("{d:+.1} %")).monospace().color(if d.abs() < 0.05 { Color32::GRAY } else { badge(good) }));
                        ui.end_row();
                    }
                });
                let mut vals: Vec<String> = self.console.ia_vars.iter().zip(&c.values).map(|(v, x)| format!("{} = {}", t(v.name()), fnum(*x))).collect();
                vals.push(format!("{} = {} mm", t("Culasse rotor"), fnum(c.rotor_yoke_mm)));
                ui.label(RichText::new(vals.join(" · ")).monospace());
            });
        }
        let has_trials = !trials.is_empty();
        drop(trials);
        // graphe 3D des possibilités explorées, au-dessus du journal
        if has_trials {
            // le journal déroulant reste visible sous le graphe
            let h = (ui.available_height() - 70.0).clamp(200.0, 460.0);
            if let Some(n) = self.ia_graph(ui, h)
                && let Some(base) = self.opt_base.clone()
                && let Some(tr) = self.console.ia.iter().find_map(|e| if let IaLog::Tr(t) = e && t.n == n { Some(t.cand.clone()) } else { None })
            {
                self.opt_live = false;
                self.spec = crate::optimize::apply_candidate(&base, &self.console.ia_vars, &tr);
                self.touched(ui.ctx());
                self.status = format!("{} #{n}", t("Essai appliqué"));
            }
        }
        // journal en menu déroulant
        let n_lines = self.console.ia.len();
        egui::CollapsingHeader::new(RichText::new(format!("📜 {} ({n_lines})", t("Journal des essais"))).strong()).id_source("ia_journal").default_open(!has_trials).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.radio_value(&mut self.console.ia_filter, IaFilter::All, t("Tous les essais"));
            ui.radio_value(&mut self.console.ia_filter, IaFilter::Best, t("Nouveaux meilleurs"));
            ui.radio_value(&mut self.console.ia_filter, IaFilter::Rejected, t("Essais rejetés"));
            ui.separator();
            ui.checkbox(&mut self.console.follow, t("Défilement auto"));
            if ui.button(format!("📋 {}", t("Copier"))).clicked() {
                let vars = self.console.ia_vars.clone();
                let txt: Vec<String> = self.console.ia.iter().map(|e| match e {
                    IaLog::Ev(e) => crate::script::event_line(e, &vars),
                    IaLog::Tr(t) => crate::script::trial_line(t, &vars),
                }).collect();
                ui.ctx().output_mut(|o| o.copied_text = txt.join("\n"));
            }
            if ui.add_enabled(!running, egui::Button::new(format!("🗑 {}", t("Effacer")))).clicked() {
                self.console.ia.clear();
                self.console.ia_ref = None;
                self.console.graph = Default::default();
            }
            if ui.button(format!("✨ {}", t("Ajustement I.A"))).clicked() {
                self.show_opt = true;
            }
        });
        let vars = self.console.ia_vars.clone();
        let filter = self.console.ia_filter;
        let lines: Vec<(Color32, String, bool)> = self.console.ia.iter().filter_map(|e| match e {
            IaLog::Ev(e) => (filter != IaFilter::Rejected).then(|| (if dark { CYAN } else { Color32::from_rgb(0, 100, 160) }, crate::script::event_line(e, &vars), true)),
            IaLog::Tr(tr) => {
                let keep = match filter {
                    IaFilter::All => true,
                    IaFilter::Best => tr.new_best,
                    IaFilter::Rejected => !tr.violations.is_empty(),
                };
                let col = if tr.new_best {
                    if dark { Color32::from_rgb(120, 240, 120) } else { Color32::from_rgb(20, 130, 40) }
                } else if !tr.violations.is_empty() {
                    Color32::from_rgb(220, 100, 90)
                } else if dark {
                    Color32::from_gray(185)
                } else {
                    Color32::from_gray(80)
                };
                keep.then(|| (col, crate::script::trial_line(tr, &vars), tr.new_best))
            }
        }).collect();
        let row_h = ui.text_style_height(&egui::TextStyle::Monospace);
        egui::ScrollArea::both().id_source("ia_log").max_height(260.0).auto_shrink([false, true]).stick_to_bottom(self.console.follow).show_rows(ui, row_h, lines.len(), |ui, range| {
            for (col, s, strong) in &lines[range] {
                let mut rt = RichText::new(s).monospace().color(*col);
                if *strong {
                    rt = rt.strong();
                }
                ui.add(egui::Label::new(rt).wrap(false));
            }
        });
        });
    }

    fn console_script(&mut self, ui: &mut egui::Ui) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let dark = self.cfg.theme == Theme::Sombre;
        let running = self.console.job.is_some();
        ui.label(RichText::new(t("Le script travaille sur une copie du projet ; « applique » reporte le résultat dans l'interface. Taper « aide » pour la liste des commandes.")).weak());
        ui.horizontal_wrapped(|ui| {
            if !running {
                if ui.add(egui::Button::new(RichText::new(format!("▶ {}", t("Exécuter le script"))).strong()).fill(Color32::from_rgb(0, 110, 160))).clicked() {
                    let src = self.console.src.clone();
                    self.run_script(src, true);
                }
            } else {
                ui.spinner();
                if ui.button(format!("⏹ {}", t("Arrêter"))).clicked()
                    && let Some(j) = &mut self.console.job
                    && j.cancel()
                {
                    self.console.job = None;
                }
            }
            if ui.button(format!("📄 {}", t("Exemple"))).clicked() {
                self.console.src = EXAMPLE.to_string();
            }
            if ui.button(format!("🗑 {}", t("Effacer la sortie"))).clicked() {
                self.console.out.clear();
            }
            if ui.button(format!("📋 {}", t("Copier"))).clicked() {
                let txt: Vec<&str> = self.console.out.iter().map(|(_, s)| s.as_str()).collect();
                ui.ctx().output_mut(|o| o.copied_text = txt.join("\n"));
            }
        });
        let h = (ui.available_height() * 0.35).clamp(80.0, 260.0);
        egui::ScrollArea::vertical().id_source("script_src").max_height(h).show(ui, |ui| {
            ui.add_enabled(!running, egui::TextEdit::multiline(&mut self.console.src).code_editor().desired_rows(8).desired_width(f32::INFINITY));
        });
        // ligne de commande : Entrée exécute, ↑ ↓ parcourent l'historique
        ui.horizontal(|ui| {
            ui.label(RichText::new("›").monospace().strong());
            let r = ui.add_enabled(!running, egui::TextEdit::singleline(&mut self.console.cmd).code_editor().hint_text(t("commande (Entrée)")).desired_width(f32::INFINITY));
            if r.has_focus() {
                let (up, down) = ui.input(|i| (i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowDown)));
                if up && self.console.hist_pos > 0 {
                    self.console.hist_pos -= 1;
                    self.console.cmd = self.console.hist[self.console.hist_pos].clone();
                }
                if down && self.console.hist_pos < self.console.hist.len() {
                    self.console.hist_pos += 1;
                    self.console.cmd = self.console.hist.get(self.console.hist_pos).cloned().unwrap_or_default();
                }
            }
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.console.cmd.trim().is_empty() {
                let c = std::mem::take(&mut self.console.cmd);
                self.console.hist.push(c.clone());
                self.console.hist_pos = self.console.hist.len();
                self.run_script(c, false);
                r.request_focus();
            }
        });
        ui.separator();
        let row_h = ui.text_style_height(&egui::TextStyle::Monospace);
        let n = self.console.out.len();
        egui::ScrollArea::both().id_source("script_out").auto_shrink([false, false]).stick_to_bottom(true).show_rows(ui, row_h, n, |ui, range| {
            for (k, s) in &self.console.out[range] {
                ui.add(egui::Label::new(RichText::new(s).monospace().color(kind_color(*k, dark))).wrap(false));
            }
        });
    }
}
