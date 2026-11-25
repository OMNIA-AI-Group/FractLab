//! Fenêtres : préférences, aide, développeur, SIMUMSAP, figures MATLAB, Ajustement I.A, licence, à propos.

use super::*;

impl App {
    // --------------------------- Fenêtres -----------------------------
    pub(super) fn prefs_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_prefs;
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        egui::Window::new(format!("⚙ {}", t("Paramètres")))
            .open(&mut open)
            .default_width(560.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.columns(2, |c| {
                    let ui = &mut c[0];
                    ui.label(RichText::new(t("Langue")).strong());
                    for l in [Lang::Fr, Lang::En, Lang::Es, Lang::De] {
                        if ui.radio_value(&mut self.cfg.lang, l, l.name()).clicked() {
                            self.cfg.save();
                        }
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new(t("Thème")).strong());
                    let before = self.cfg.theme;
                    ui.radio_value(&mut self.cfg.theme, Theme::Sombre, format!("🌙 {}", t("Sombre")));
                    ui.radio_value(&mut self.cfg.theme, Theme::Clair, format!("☀ {}", t("Clair")));
                    if before != self.cfg.theme {
                        apply_theme(ctx, self.cfg.theme);
                        self.cfg.save();
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new(t("Taille de l'interface")).strong());
                    let cur = self.cfg.ui_scale;
                    let label = |v: f32| if v == 0.0 { t("Auto (selon l'écran)").to_string() } else { format!("{:.0} %", v * 100.0) };
                    egui::ComboBox::from_id_source("ui_scale").selected_text(label(cur)).show_ui(ui, |ui| {
                        for v in [0.0f32, 0.8, 0.9, 1.0, 1.15, 1.3, 1.5] {
                            if ui.selectable_value(&mut self.cfg.ui_scale, v, label(v)).changed() {
                                self.cfg.save();
                            }
                        }
                    });
                    ui.label(RichText::new(format!("{} {:.0} %", t("actuelle :"), self.applied_zoom * 100.0)).small().weak());
                    let ui = &mut c[1];
                    ui.label(RichText::new(t("Unités")).strong());
                    egui::Grid::new("units").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
                        for qn in units::QUANTITIES {
                            ui.label(t(units::name_fr(qn)));
                            let opts = units::options(qn);
                            let cur = *self.cfg.units.pick_mut(qn);
                            let mut changed = false;
                            egui::ComboBox::from_id_source(("unit", units::name_fr(qn))).selected_text(opts[cur.min(opts.len() - 1)].0).show_ui(ui, |ui| {
                                for (k, (sym, _)) in opts.iter().enumerate() {
                                    changed |= ui.selectable_value(self.cfg.units.pick_mut(qn), k, *sym).changed();
                                }
                            });
                            if changed {
                                self.cfg.save();
                            }
                            ui.end_row();
                        }
                    });
                    if ui.button(t("Unités par défaut (SI usuel)")).clicked() {
                        self.cfg.units = Units::default();
                        self.cfg.save();
                    }
                });
            });
        self.show_prefs = open;
    }

    pub(super) fn help_window(&mut self, ctx: &egui::Context) {
        let sections = crate::help::sections();
        let lang = self.cfg.lang;
        let mut open = self.show_help;
        let sz = ctx.screen_rect().size();
        egui::Window::new(format!("📖 {}", T(lang, "Guide d'utilisation")))
            .open(&mut open)
            .default_size([(sz.x * 0.7).min(980.0), (sz.y * 0.75).min(680.0)])
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(egui::TextEdit::singleline(&mut self.help_search).hint_text(T(lang, "Rechercher dans l'aide")).desired_width(260.0));
                });
                ui.separator();
                let q = self.help_search.to_lowercase();
                let visible: Vec<usize> = (0..sections.len())
                    .filter(|&k| q.is_empty() || sections[k].title(lang).to_lowercase().contains(&q) || sections[k].body(lang).to_lowercase().contains(&q))
                    .collect();
                if !visible.contains(&self.help_sel)
                    && let Some(&f) = visible.first() {
                        self.help_sel = f;
                    }
                egui::SidePanel::left("help_toc").resizable(true).default_width(230.0).show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for &k in &visible {
                            let s = sections[k];
                            ui.selectable_value(&mut self.help_sel, k, format!("{} {}", s.icon, s.title(lang)));
                        }
                        if visible.is_empty() {
                            ui.label(RichText::new(T(lang, "Aucun résultat")).weak());
                        }
                    });
                });
                egui::TopBottomPanel::bottom("help_author").show_inside(ui, |ui| {
                    ui.add_space(4.0);
                    self.author_card(ui);
                });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let s = sections[self.help_sel.min(sections.len() - 1)];
                    ui.heading(RichText::new(format!("{} {}", s.icon, s.title(lang))).strong());
                    ui.add_space(6.0);
                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        for line in s.body(lang).lines() {
                            let l = line.trim();
                            if let Some(b) = l.strip_prefix('•') {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("  •").color(ORANGE));
                                    ui.add(egui::Label::new(b.trim()).wrap(true));
                                });
                            } else if l.ends_with(':') || l.ends_with('：') {
                                ui.add_space(4.0);
                                ui.label(RichText::new(l).strong().color(if view3d::is_dark() { CYAN } else { Color32::from_rgb(0, 110, 170) }));
                            } else if !l.is_empty() {
                                ui.add(egui::Label::new(l).wrap(true));
                                ui.add_space(4.0);
                            }
                        }
                    });
                });
            });
        self.show_help = open;
    }

    /// Son en direct : relance la lecture quand la vitesse change (> 3 %) ou
    /// quand la boucle précédente se termine.
    pub(super) fn live_sound_tick(&mut self) {
        let finished = match self.sound_child.as_mut() {
            Some(c) => c.finished(),
            None => true,
        };
        if !self.live_sound {
            if let Some(mut c) = self.sound_child.take() {
                c.stop();
            }
            return;
        }
        let changed = (self.anim_rpm / self.sound_rpm.max(1e-9) - 1.0).abs() > 0.03;
        if !(changed || finished) {
            return;
        }
        let Some(r) = self.res.as_ref() else { return };
        if let Some(mut c) = self.sound_child.take() {
            c.stop();
        }
        let secs = if self.ramp { 0.6 } else { 3.0 };
        let wav = crate::dynamics::synth_wav(r, self.anim_rpm, secs);
        self.sound_child = crate::platform::play_wav(&wav);
        self.sound_rpm = self.anim_rpm;
    }

    /// Fiche de l'auteur (photo intégrée, non modifiable).
    pub(super) fn author_card(&mut self, ui: &mut egui::Ui) {
        self.author_card_sized(ui, 120.0);
    }

    /// Carte de l'auteur avec une photo de hauteur `photo_h` (photo intégrée, non remplaçable).
    pub(super) fn author_card_sized(&mut self, ui: &mut egui::Ui, photo_h: f32) {
        if self.photo_tex.is_none() {
            self.photo_tex = load_png(ui.ctx(), "photo_auteur", include_bytes!("../../assets/auteur.jpg"));
        }
        egui::Frame::none().fill(ui.visuals().faint_bg_color).rounding(10.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(tex) = &self.photo_tex {
                    let sz = tex.size_vec2();
                    ui.add(egui::Image::new((tex.id(), sz * (photo_h / sz.y.max(1.0)))).rounding(10.0));
                }
                ui.vertical(|ui| {
                    ui.set_min_width(260.0);
                    ui.label(RichText::new(tr("Développeur de l'application")).weak());
                    ui.label(RichText::new("Alain VALETTE-CLARY").size(19.0).strong());
                    ui.label(RichText::new("ΩiA-Group").size(15.0).color(ORANGE));
                    ui.label(RichText::new("INPI DSO2025023838").monospace());
                    ui.hyperlink_to("🔗 LinkedIn", "https://www.linkedin.com/in/alain-valette-clary-524b28352/");
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("{} — Magnetic Engine Simulator · {} {}", crate::repartition::app_name(), tr("version"), env!("CARGO_PKG_VERSION"))).weak());
                    if ui.link(tr("Licence")).clicked() {
                        self.show_license = true;
                    }
                });
            });
        });
    }

    pub(super) fn simu_window(&mut self, ctx: &egui::Context) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let mut open = true;
        let (fields, ind, dir) = self.simu.clone().unwrap();
        egui::Window::new(format!("📂 {}", t("Projet SIMUMSAP"))).open(&mut open).default_size([560.0, 480.0]).resizable(true).show(ctx, |ui| {
            ui.label(RichText::new(format!("{:?} · {}N{}P", self.spec.topology, self.spec.n_slots, self.spec.n_poles)).strong());
            ui.horizontal_wrapped(|ui| {
                if ui.button(t("Hypothèses SIMUMSAP (encoches ouvertes)")).on_hover_text(t("Le modèle axial de SIMUMSAP (SolvSys_AFPMSM_OS) ignore les becs de dents : encoches ouvertes sur toute leur largeur")).clicked() {
                    self.spec.tip_h = 0.0;
                    self.spec.slot_open = self.spec.slot_w.max(self.spec.slot_open);
                    self.touched(ctx);
                }
                if ui.button(t("Charger les figures du dossier")).clicked() {
                    if crate::platform::WEB {
                        // navigateur : pas d'accès aux dossiers, l'utilisateur choisit les figures
                        Pick::Figures.ask("MATLAB", &["fig", "mat"]);
                    } else {
                        let mut v = Vec::new();
                        let mut stack = vec![dir.clone()];
                        while let Some(d) = stack.pop() {
                            if let Ok(rd) = std::fs::read_dir(&d) {
                                for e in rd.flatten() {
                                    let q = e.path();
                                    if q.is_dir() {
                                        stack.push(q);
                                    } else if q.extension().map(|x| x == "fig").unwrap_or(false) {
                                        v.push(q);
                                    }
                                }
                            }
                        }
                        v.sort();
                        let files = v.into_iter().filter_map(|p| std::fs::read(&p).ok().map(|b| (p.display().to_string(), b))).collect();
                        self.load_figures(files);
                    }
                }
            });
            if let (Some(l), Some(r)) = (&ind, &self.res) {
                let la = r.model.phase_flux(&r.model.solve_armature(&[1.0, 0.0, 0.0]));
                let per = geometry::gcd(r.spec.n_slots, r.spec.n_poles / 2) as f64;
                let g = if r.spec.topology == Topology::Axial { r.spec.axial.gaps as f64 } else { 1.0 };
                ui.label(format!(
                    "{} : L = {:.4} mH, M = {:.4} mH · FractLab ({}) : L = {:.4} mH, M = {:.4} mH",
                    t("Inductances SIMUMSAP"),
                    l[0][0] * 1e3,
                    l[0][1] * 1e3,
                    t("1 période, 1 entrefer, hors têtes de bobines"),
                    la[0] * 1e3 / per / g,
                    la[1] * 1e3 / per / g
                ));
            }
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("simu_f").striped(true).num_columns(2).show(ui, |ui| {
                    for (k, v) in &fields {
                        ui.label(RichText::new(k).small());
                        ui.label(RichText::new(v.lines().next().unwrap_or("").chars().take(60).collect::<String>()).small().monospace());
                        ui.end_row();
                    }
                });
            });
        });
        if !open {
            self.simu = None;
        }
    }

    pub(super) fn figs_window(&mut self, ctx: &egui::Context) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let mut open = self.show_figs;
        let mut add: Option<(String, Vec<[f64; 2]>, &'static str)> = None;
        egui::Window::new(format!("📈 {}", t("Figures MATLAB"))).open(&mut open).default_size([760.0, 560.0]).resizable(true).show(ctx, |ui| {
            ui.label(RichText::new(t("Courbes lues dans les fichiers MATLAB (sans MATLAB). « Superposer » les ajoute à la vue choisie pour comparer avec FractLab.")).weak());
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (fi, (name, axes)) in self.figs.iter().enumerate() {
                    for (ai, a) in axes.iter().enumerate() {
                        egui::CollapsingHeader::new(format!("{name} — {}", if a.ylabel.is_empty() { a.title.clone() } else { a.ylabel.clone() })).id_source((fi, ai)).show(ui, |ui| {
                            Plot::new(("fig", fi, ai)).height(180.0).legend(Legend::default()).x_axis_label(a.xlabel.clone()).y_axis_label(a.ylabel.clone()).show(ui, |p| {
                                for (k, c) in a.curves.iter().enumerate() {
                                    let pts: Vec<[f64; 2]> = c.x.iter().zip(&c.y).map(|(x, y)| [*x, *y]).collect();
                                    p.line(Line::new(PlotPoints::from(pts)).name(if c.name.is_empty() { format!("{} {}", t("courbe"), k + 1) } else { c.name.clone() }).color(SERIES[k % 6]));
                                }
                            });
                            ui.horizontal_wrapped(|ui| {
                                for (k, c) in a.curves.iter().enumerate() {
                                    for (kind, lbl) in [("cogging", t("Cogging")), ("fem", t("FEM")), ("champ", t("Champ")), ("forces", t("Forces")), ("charge", t("Charge"))] {
                                        if ui.small_button(format!("{} {} : {}", t("courbe"), k + 1, lbl)).clicked() {
                                            add = Some((format!("{name} #{}", k + 1), c.x.iter().zip(&c.y).map(|(x, y)| [*x, *y]).collect(), kind));
                                        }
                                    }
                                }
                            });
                        });
                    }
                }
            });
        });
        if let Some(a) = add {
            self.status = format!("{} {}", t("Superposé :"), a.0);
            self.overlays.push(a);
        }
        self.show_figs = open;
    }

    pub(super) fn poll_opt(&mut self, ctx: &egui::Context) {
        use crate::optimize::OptMsg;
        let mut done = false;
        let mut live = false;
        let mut trials = Vec::new();
        let mut events = Vec::new();
        if let Some(j) = &self.opt_job {
            while let Ok(m) = j.rx.try_recv() {
                match m {
                    OptMsg::Progress(n, b, top, h) => {
                        self.opt_prog = (n, b);
                        self.opt_top = top;
                        self.opt_hist = h;
                        live = true;
                    }
                    OptMsg::Trace(v) => trials.extend(v),
                    OptMsg::Event(e) => events.push(e),
                    OptMsg::Done => done = true,
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        // ordre conservé : les événements d'un envoi précèdent ou suivent ses essais
        for e in events {
            let after = matches!(e, crate::optimize::Event::End(..) | crate::optimize::Event::StartEnd(..));
            if after {
                for t in trials.drain(..) {
                    self.console_ia_push(super::console::IaLog::Tr(t));
                }
            }
            self.console_ia_push(super::console::IaLog::Ev(e));
        }
        for t in trials {
            self.console_ia_push(super::console::IaLog::Tr(t));
        }
        if live
            && self.opt_live
            && let (Some(best), Some(base)) = (self.opt_top.first(), self.opt_base.as_ref())
            && best.feasible
            && best.score < self.opt_live_score - 1e-12
        {
            self.opt_live_score = best.score;
            self.spec = crate::optimize::apply_candidate(base, &self.opt_vars_run, best);
            self.touched(ctx);
        }
        if done {
            self.opt_job = None;
            self.status = T(self.cfg.lang, "Ajustement I.A terminé").into();
            // la comparaison s'ouvre à la demande (bouton orange) ; ouverture
            // automatique seulement pour les essais (FRACTLAB_IA=compare)
            if std::env::var("FRACTLAB_IA").as_deref() == Ok("compare") && self.demo_inactive() && self.opt_base.is_some() && !self.opt_top.is_empty() {
                self.cmpia_start();
            }
        }
    }

    /// Onglet « Ajustement I.A ».
    pub(super) fn opt_tab(&mut self, ui: &mut egui::Ui) {
        let ctx = &ui.ctx().clone();
        use crate::optimize::{self as op, Goal};
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let u = self.cfg.units.clone();
        let mut apply: Option<usize> = None;
        egui::ScrollArea::vertical().id_source("opt_tab").auto_shrink([false, false]).show(ui, |ui| {
            ui.label(RichText::new(t("Recherche automatique des meilleurs réglages : chaque essai est un calcul complet du moteur. Les 5 meilleures solutions distinctes s'affichent en direct ; « Appliquer » les reporte dans le projet.")).weak());
            let running = self.opt_job.is_some();
            ui.add_enabled_ui(!running, |ui| {
                ui.horizontal_wrapped(|ui| {
                    lbl(ui, t("Objectif"));
                    let gn = |g: Goal| T(lang, g.name());
                    egui::ComboBox::from_id_source("goal").selected_text(gn(self.opt_goal)).show_ui(ui, |ui| {
                        for g in op::ALL_GOALS {
                            ui.selectable_value(&mut self.opt_goal, g, gn(g));
                        }
                    });
                    lbl(ui, t("Essais"));
                    ui.add(egui::DragValue::new(&mut self.opt_budget).clamp_range(20..=2000));
                    lbl(ui, t("FEM conservée ≥"));
                    ui.add(egui::DragValue::new(&mut self.opt_emf_keep).clamp_range(0.5..=1.0).speed(0.01).max_decimals(2));
                    chk(ui, &mut self.opt_sat_limit, t("Culasse rotor non saturée"));
                });
                ui.horizontal_wrapped(|ui| {
                    let mut on = self.opt_joule_max.is_some();
                    if chk(ui, &mut on, t("Pertes Joule ≤ original +")).changed() {
                        self.opt_joule_max = on.then_some(10.0);
                    }
                    if let Some(x) = &mut self.opt_joule_max {
                        ui.add(egui::DragValue::new(x).clamp_range(0.0..=200.0).speed(0.5).max_decimals(1).suffix(" %"));
                    }
                    ui.separator();
                    let mut on = self.opt_torque_keep.is_some();
                    if chk(ui, &mut on, t("Couple moyen ≥")).changed() {
                        self.opt_torque_keep = on.then_some(0.95);
                    }
                    if let Some(x) = &mut self.opt_torque_keep {
                        let mut pct = *x * 100.0;
                        if ui.add(egui::DragValue::new(&mut pct).clamp_range(50.0..=150.0).speed(0.5).max_decimals(1).suffix(" %")).changed() {
                            *x = pct / 100.0;
                        }
                        lbl(ui, t("de l'original"));
                    }
                    if crate::repartition::uses_offsets() && self.opt_vars.iter().any(|(v, on)| *on && *v == op::Var::SkewAmp) {
                        lbl(ui, t("Amplitude des décalages ≥"));
                        ui.add(egui::DragValue::new(&mut self.opt_skew_min).clamp_range(0.0..=8.0).speed(0.05).max_decimals(2).suffix(" °"));
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    lbl(ui, t("Réglages libres :"));
                    for (v, on) in self.opt_vars.iter_mut() {
                        ui.checkbox(on, t(v.name()));
                    }
                });
            });
            ui.horizontal(|ui| {
                if !running {
                    if ui.add(egui::Button::new(RichText::new(format!("▶ {}", t("Lancer l'ajustement"))).strong().color(Color32::WHITE)).fill(ACTION)).clicked() {
                        let vars: Vec<op::Var> = self.opt_vars.iter().filter(|(_, on)| *on).map(|(v, _)| *v).collect();
                        if !vars.is_empty() {
                            self.opt_base = Some(self.spec.clone());
                            self.opt_vars_run = vars.clone();
                            self.opt_live_score = f64::INFINITY;
                            self.opt_top.clear();
                            self.opt_hist.clear();
                            self.console_ia_start(&vars);
                            self.opt_job = Some(op::run(self.spec.clone(), op::Setup { vars, goal: self.opt_goal, budget: self.opt_budget, emf_keep: self.opt_emf_keep, sat_limit: self.opt_sat_limit, skew_min: self.opt_skew_min, joule_max_pct: self.opt_joule_max, torque_keep: self.opt_torque_keep }));
                        }
                    }
                } else if ui.button(format!("⏹ {}", t("Arrêter"))).clicked()
                    && let Some(j) = &mut self.opt_job
                    && j.cancel()
                {
                    // navigateur : Worker arrêté, les meilleures propositions reçues restent
                    self.opt_job = None;
                }
                if self.opt_prog.1 > 0 {
                    ui.add(egui::ProgressBar::new(self.opt_prog.0 as f32 / self.opt_prog.1 as f32).text(format!("{} / {}", self.opt_prog.0, self.opt_prog.1)).desired_width(240.0));
                }
                if running {
                    ui.spinner();
                }
                if !running
                    && !self.opt_top.is_empty()
                    && self.opt_base.is_some()
                    && ui.add(egui::Button::new(RichText::new(format!("⚖ {}", t("Comparer les solutions"))).strong().color(Color32::WHITE)).fill(IA_COL)).on_hover_text(t("Original et propositions recalculés en calcul complet, métriques et graphiques côte à côte, puis choix de la solution")).clicked()
                {
                    self.cmpia_start();
                }
                if ui.button(format!("🕘 {}", t("Historique des comparaisons"))).clicked() {
                    self.cmpia.open = true;
                }
                if ui.button(format!("🖥 {}", t("Console du moteur I.A"))).clicked() {
                    self.console.open = true;
                    self.console.tab = super::console::ConsoleTab::Ia;
                }
            });
            ui.horizontal_wrapped(|ui| {
                chk(ui, &mut self.opt_live, t("Appliquer en direct la meilleure proposition : toutes les vues suivent"));
                if self.opt_base.is_some() && ui.button(format!("↺ {}", t("Revenir au projet initial"))).clicked() {
                    self.restore_base = true;
                }
            });
            ui.separator();
            let vars: Vec<op::Var> = self.opt_vars.iter().filter(|(_, on)| *on).map(|(v, _)| *v).collect();
            ui.label(RichText::new(format!("🏆 {}", t("5 meilleures propositions"))).strong());
            egui::Grid::new("opt_top").striped(true).num_columns(8 + vars.len()).spacing([10.0, 3.0]).show(ui, |ui| {
                for h in ["#", t("Score"), t("Cogging pp"), t("Force max"), t("FEM crête"), t("Ondulation"), t("culasse rotor"), t("Faisable")] {
                    ui.label(RichText::new(h).small().strong());
                }
                for v in &vars {
                    ui.label(RichText::new(t(v.name())).small().strong());
                }
                ui.label("");
                ui.end_row();
                for (k, c) in self.opt_top.iter().enumerate() {
                    ui.label(format!("{}", k + 1));
                    ui.label(RichText::new(format!("{:.4}", c.score)).monospace());
                    ui.label(format!("{} {}", fnum(u.show(Q::Torque, c.cogging_mnm * 1e-3)), u.sym(Q::Torque)));
                    ui.label(format!("{} {}", fnum(u.show(Q::Force, c.force_n)), u.sym(Q::Force)));
                    ui.label(format!("{:.3} V", c.emf_v));
                    ui.label(format!("{:.1} %", c.ripple_pct));
                    ui.label(format!("{:.2} T · {} mm", c.b_rotor_t, fnum(c.rotor_yoke_mm)));
                    ui.label(RichText::new(if c.feasible { "✔" } else { "✖" }).color(badge(c.feasible)));
                    for x in &c.values {
                        ui.label(RichText::new(fnum(*x)).monospace());
                    }
                    if ui.button(t("Appliquer")).clicked() {
                        apply = Some(k);
                    }
                    ui.end_row();
                }
            });
            if !self.opt_hist.is_empty() {
                let pts: Vec<[f64; 2]> = self.opt_hist.iter().enumerate().map(|(i, v)| [i as f64, *v]).collect();
                Plot::new("opt_hist").height(ui.available_height().max(100.0)).x_axis_label(t("essai")).y_axis_label(t("meilleur score")).show(ui, |p| {
                    p.line(Line::new(PlotPoints::from(pts)).color(ORANGE).width(2.0f32));
                });
            }
        });
        if self.restore_base {
            self.restore_base = false;
            if let Some(b) = self.opt_base.clone() {
                self.spec = b;
                self.opt_live = false;
                self.touched(ctx);
                self.status = t("Projet initial rétabli").into();
            }
        }
        if let (Some(k), Some(base)) = (apply, self.opt_base.clone()) {
            let vars = self.opt_vars_run.clone();
            if let Some(c) = self.opt_top.get(k) {
                self.opt_live = false;
                self.spec = op::apply_candidate(&base, &vars, c);
                self.touched(ctx);
                self.status = format!("{} #{}", t("Proposition appliquée"), k + 1);
            }
        }
    }

    pub(super) fn license_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_license;
        egui::Window::new(format!("⚖ {}", tr("Licence"))).open(&mut open).default_size([640.0, 520.0]).resizable(true).show(ctx, |ui| {
            ui.label(RichText::new("FractLab — © Alain VALETTE-CLARY · ΩiA-Group · INPI DSO2025023838").strong());
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(include_str!("../../LICENSE")).monospace().small()).wrap(true));
            });
        });
        self.show_license = open;
    }

    /// Fenêtre « Développeur de l'application » (clic sur un logo, menu Aide).
    pub(super) fn author_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_author;
        egui::Window::new(format!("👤 {}", tr("Développeur de l'application")))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .min_width(520.0)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                self.author_card_sized(ui, 300.0);
            });
        self.show_author = open;
    }

    pub(super) fn about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_about;
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        egui::Window::new(t("À propos et hypothèses du modèle")).open(&mut open).default_width(620.0).show(ctx, |ui| {
            if let Some(l) = self.logo_tex.clone() {
                if ui.add(egui::Image::new((l.id(), Vec2::new(580.0, 323.0))).rounding(10.0).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.show_author = true;
                }
            }
            self.author_card(ui);
            ui.separator();
            for k in [
                "Modèle : solution formelle des équations de Maxwell en magnétostatique 2D, par sous-domaines en coordonnées polaires exactes (encoches, entrefer, aimants), séries de Fourier sur 360° complets : toute répartition d'aimants est admise, y compris apériodique.",
                "Hypothèses : fer linéaire de perméabilité infinie, aimants de perméabilité uniforme, champ 2D par tronçon (multi-tranches pour le flux axial), inductances hors têtes de bobines.",
                "Grandeurs : induction d'entrefer, couple de détente (tenseur de Maxwell), FEM par bobinage concentré déterminé par l'étoile des encoches, force radiale résultante, métriques spectrales du couple de détente.",
                "Validation : solveur volumes finis polaire indépendant, écart typique de 1 à 3 % sur le couple de détente.",
                "Couvert : rotor extérieur ou intérieur, flux axial, encoches semi-fermées, bobinage général, fonctionnement en charge, inductances, excentricité, simulation dynamique avec alimentations, charges et défauts, optimisation.",
            ] {
                ui.label(t(k));
                ui.add_space(4.0);
            }
        });
        self.show_about = open;
    }
}
