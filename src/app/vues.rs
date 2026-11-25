//! Onglets, panneau central et plein écran.

use super::*;

impl App {
    // ------------------------------------------------------------------
    // Onglets
    // ------------------------------------------------------------------
    pub(super) fn tabs(&mut self, ui: &mut egui::Ui) {
        let dark = self.cfg.theme == Theme::Sombre;
        let accent = if dark { CYAN } else { Color32::from_rgb(0, 110, 170) };
        let pulse = pulse(ui);
        // la comparaison des variantes n'a de sens qu'avec plusieurs répartitions proposées
        let compare = crate::repartition::variants().len() > 1;
        let visible = |t: Tab| t != Tab::Comparaison || compare;
        let cur_step = STEPS.iter().position(|s| s.2.contains(&self.tab)).unwrap_or(0);
        // ---- étapes de l'étude
        ui.horizontal_wrapped(|ui| {
            for (k, (name, icon, tabs)) in STEPS.iter().enumerate() {
                let sel = k == cur_step;
                let fresh = tabs.iter().any(|t| self.fresh.contains(t) && *t != self.tab);
                let txt = RichText::new(format!("{icon} {}", self.t(name))).size(15.5).strong();
                let txt = if sel { txt.color(Color32::WHITE) } else { txt };
                let mut b = egui::Button::new(txt).min_size(Vec2::new(0.0, 28.0));
                if sel {
                    b = b.fill(accent.gamma_multiply(0.55));
                }
                let r = ui.add(b);
                if fresh {
                    let c = r.rect.right_top() + Vec2::new(-5.0, 5.0);
                    ui.painter().circle_filled(c, 3.5, ORANGE.gamma_multiply(0.5 + 0.5 * pulse));
                }
                if r.clicked() && !sel {
                    // dernier onglet ouvert dans cette étape, sinon le premier
                    let t = self.step_tab.get(k).copied().flatten().filter(|t| tabs.contains(t) && visible(*t)).or_else(|| tabs.iter().copied().find(|t| visible(*t)));
                    if let Some(t) = t {
                        self.tab = t;
                    }
                }
                if k + 1 < STEPS.len() {
                    ui.label(RichText::new("›").weak());
                }
            }
        });
        // ---- onglets de l'étape
        ui.horizontal_wrapped(|ui| {
            for tab in STEPS[cur_step].2.iter().copied().filter(|t| visible(*t)) {
                let (_, ic, k) = ALL_TABS.iter().find(|x| x.0 == tab).copied().unwrap();
                let sel = self.tab == tab;
                let txt = RichText::new(format!("{ic} {}", self.t(k))).size(14.5);
                let txt = if sel { txt.strong().color(accent) } else { txt };
                let r = with_tip(ui.selectable_label(sel, txt), self.t(k));
                if tab == Tab::Moteur3d && !self.cfg.seen_field3d {
                    let c = r.rect.right_top() + Vec2::new(-4.0, 4.0);
                    ui.painter().circle_filled(c, 5.0, champ3d::FIELD3D_COL.gamma_multiply(0.4 + 0.6 * pulse));
                    ui.painter().text(c, egui::Align2::CENTER_CENTER, "👁", egui::FontId::proportional(8.0), Color32::WHITE);
                }
                if self.fresh.contains(&tab) && !sel {
                    let c = r.rect.right_top() + Vec2::new(-5.0, 5.0);
                    ui.painter().circle_filled(c, 3.0, ORANGE.gamma_multiply(0.75));
                }
                if (tab == Tab::Ia && self.opt_job.is_some()) || (tab == Tab::Solutions && self.cmpia_busy()) {
                    let c = r.rect.right_top() + Vec2::new(-5.0, 5.0);
                    ui.painter().circle_filled(c, 3.5, accent.gamma_multiply(0.4 + 0.6 * pulse));
                }
                if r.clicked() {
                    self.tab = tab;
                }
            }
        });
        if let Some(k) = STEPS.iter().position(|s| s.2.contains(&self.tab))
            && let Some(slot) = self.step_tab.get_mut(k)
        {
            *slot = Some(self.tab);
        }
        ui.separator();
    }

    pub(super) fn rotor_slider(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(self.t("Position du rotor θr"));
            ui.add(egui::Slider::new(&mut self.theta_r, 0.0..=360.0).suffix("°").max_decimals(3));
            ui.toggle_value(&mut self.animate, format!("⟳ {}", tr("Animer")));
        });
    }

    pub(super) fn field(&mut self) -> Option<(&Field, &GapField)> {
        let r = self.res.as_ref()?;
        let fresh = matches!(&self.field_cache, Some((a, _, _)) if (*a - self.theta_r).abs() < 1e-12);
        if !fresh {
            let f = r.model.solve(self.theta_r.to_radians());
            let g = analysis::gap_field(&r.model, &f, 1440);
            self.field_cache = Some((self.theta_r, f, g));
        }
        self.field_cache.as_ref().map(|(_, f, g)| (f, g))
    }

    pub(super) fn central(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.job.is_some() || self.cmp_job.is_some() || self.surf_job.is_some() || self.val_job.is_some() {
                    ui.spinner();
                }
                ui.label(RichText::new(&self.status).small());
                if let Some(e) = &self.err {
                    ui.label(RichText::new(format!("⚠ {e}")).color(Color32::from_rgb(255, 90, 90)));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(RichText::new("🔗 LinkedIn").small(), "https://www.linkedin.com/in/alain-valette-clary-524b28352/");
                    ui.label(RichText::new("INPI DSO2025023838").small().monospace());
                    let who = ui.add(egui::Label::new(RichText::new(format!("{} : Alain VALETTE-CLARY · ΩiA-Group", tr("Développeur de l'application"))).small().strong()).sense(egui::Sense::click()));
                    if who.on_hover_text(tr("À propos")).clicked() {
                        self.show_about = true;
                    }
                    if self.photo_tex.is_none() {
                        self.photo_tex = load_png(ui.ctx(), "photo_auteur", include_bytes!("../../assets/auteur.jpg"));
                    }
                    if let Some(tex) = &self.photo_tex {
                        let sz = tex.size_vec2();
                        ui.add(egui::Image::new((tex.id(), sz * (22.0 / sz.y.max(1.0)))).rounding(11.0));
                    }
                });
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.full = None;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(f) = self.full {
                ui.horizontal(|ui| {
                    if ui.button(RichText::new(format!("◀ {}", tr("Retour (Échap)"))).strong()).clicked() {
                        self.full = None;
                    }
                    let (_, ic, k) = ALL_TABS.iter().find(|x| x.0 == f).copied().unwrap();
                    ui.heading(RichText::new(format!("{ic} {}", tr(k))).strong());
                    ui.label(RichText::new(tr("plein écran")).weak());
                });
                ui.separator();
                self.fresh.remove(&f);
                if self.res.is_some() {
                    self.show_tab(f, ui);
                }
                return;
            }
            self.tabs(ui);
            self.fresh.remove(&self.tab);
            if self.res.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        if let Some(l) = self.logo_tex.clone() {
                            if ui.add(egui::Image::new((l.id(), Vec2::new(560.0, 312.0))).rounding(12.0).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                self.show_author = true;
                            }
                        }
                        ui.add_space(10.0);
                        ui.spinner();
                    });
                });
                return;
            }
            self.show_tab(self.tab, ui);
        });
    }

    pub(super) fn show_tab(&mut self, tab: Tab, ui: &mut egui::Ui) {
        // bandeau des critères : tous dans la Synthèse, ceux de l'onglet ailleurs
        if self.res.is_some() {
            self.criteria_banner(ui, tab);
        }
        {
            match tab {
                Tab::Synthese => self.tab_synthese(ui),
                Tab::Geometrie => self.tab_geometrie(ui),
                Tab::Champ => self.tab_champ(ui),
                Tab::Cogging => self.tab_cogging(ui),
                Tab::Fem => self.tab_fem(ui),
                Tab::Forces => self.tab_forces(ui),
                Tab::Charge => self.tab_charge(ui),
                Tab::Dynamique => self.tab_dynamique(ui),
                Tab::Comparaison => self.tab_comparaison(ui),
                Tab::Moteur3d => self.tab_moteur3d(ui),
                Tab::Carte3d => self.tab_carte3d(ui),
                Tab::Validation => self.tab_validation(ui),
                Tab::Parametrique => self.tab_parametrique(ui),
                Tab::Ia => self.opt_tab(ui),
                Tab::Console => self.console_tab(ui),
                Tab::Solutions => self.cmpia_tab(ui),
            }
        }
    }
}
