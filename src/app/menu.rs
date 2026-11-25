//! Barre de menus (fichier, calcul, paramètres, aide).

use super::*;

impl App {
    // ------------------------------------------------------------------
    // Barre de menus
    // ------------------------------------------------------------------
    pub(super) fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.toggle_value(&mut self.show_params, "☰").on_hover_text(self.t("Afficher / masquer le panneau des paramètres"));
                ui.menu_button(self.t("Fichier"), |ui| {
                    if ui.button(self.t("Nouveau projet")).clicked() {
                        self.spec = crate::repartition::default_project();
                        self.touched(ctx);
                        ui.close_menu();
                    }
                    if ui.button(self.t("Ouvrir un projet…")).clicked() {
                        ui.close_menu();
                        Pick::Project.ask("FractLab", &["json"]);
                    }
                    if ui.button(format!("📂 {}", self.t("Ouvrir un projet SIMUMSAP (MATLAB)…"))).clicked() {
                        ui.close_menu();
                        Pick::Simumsap.ask("SIMUMSAP", &["txt"]);
                    }
                    if ui.button(self.t("Enregistrer le projet…")).clicked() {
                        ui.close_menu();
                        self.save_project_dialog();
                    }
                    ui.separator();
                    ui.menu_button(format!("⬇ {}", self.t("Importer")), |ui| self.import_menu(ui, ctx));
                    ui.menu_button(format!("⬆ {}", self.t("Exporter")), |ui| self.export_menu(ui));
                    // navigateur : on ferme l'onglet, pas l'application
                    if !crate::platform::WEB {
                        ui.separator();
                        if ui.button(self.t("Quitter")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.menu_button(self.t("Calcul"), |ui| {
                    if ui.button(self.t("Calculer maintenant")).clicked() {
                        self.launch();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(format!("✨ {}…", self.t("Ajustement I.A"))).clicked() {
                        self.show_opt = true;
                        ui.close_menu();
                    }
                    if ui.button(format!("⚖ {}…", self.t("Comparaison des solutions"))).on_hover_text(self.t("Après un Ajustement I.A : original et propositions côte à côte ; sinon l'historique des comparaisons")).clicked() {
                        // résultats d'un ajustement pas encore comparés : nouvelle comparaison ; sinon historique
                        let fresh = self.opt_job.is_none() && self.opt_base.is_some() && !self.opt_top.is_empty() && self.cmpia.is_empty();
                        if fresh {
                            self.cmpia_start();
                        } else {
                            self.cmpia.open = true;
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if crate::repartition::variants().len() > 1 && ui.button(self.t("Comparer toutes les variantes")).clicked() {
                        self.cmp_job = Some(compute::run_compare(self.spec.clone(), self.st));
                        self.tab = Tab::Comparaison;
                        ui.close_menu();
                    }
                    if ui.button(self.t("Valider contre les volumes finis")).clicked() {
                        self.val_job = Some(compute::run_validation(self.spec.clone(), self.st, 5760, 8));
                        self.tab = Tab::Validation;
                        ui.close_menu();
                    }
                });
                ui.menu_button(self.t("Paramètres"), |ui| {
                    if ui.button(self.t("Préférences et unités…")).clicked() {
                        self.show_prefs = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label(self.t("Langue"));
                    for l in [Lang::Fr, Lang::En, Lang::Es, Lang::De] {
                        if ui.radio_value(&mut self.cfg.lang, l, l.name()).clicked() {
                            self.cfg.save();
                        }
                    }
                    ui.separator();
                    ui.label(self.t("Thème"));
                    let before = self.cfg.theme;
                    ui.radio_value(&mut self.cfg.theme, Theme::Sombre, tr("Sombre"));
                    ui.radio_value(&mut self.cfg.theme, Theme::Clair, tr("Clair"));
                    if before != self.cfg.theme {
                        apply_theme(ctx, self.cfg.theme);
                        self.cfg.save();
                    }
                });
                ui.menu_button(self.t("Aide"), |ui| {
                    if ui.button(RichText::new(format!("▶ {}…", self.t("Démo"))).strong()).clicked() {
                        self.demo.confirm = true;
                        ui.close_menu();
                    }
                    if ui.button(format!("📖 {}  (F1)", self.t("Guide d'utilisation"))).clicked() {
                        self.show_help = true;
                        ui.close_menu();
                    }
                    if ui.button(format!("⚖ {}", self.t("Licence"))).clicked() {
                        self.show_license = true;
                        ui.close_menu();
                    }
                    if ui.button(format!("👤 {}", self.t("Développeur de l'application"))).clicked() {
                        self.show_author = true;
                        ui.close_menu();
                    }
                    if ui.button(self.t("À propos et hypothèses du modèle")).clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let icon = if self.cfg.theme == Theme::Sombre { "☀" } else { "🌙" };
                    if ui.button(icon).on_hover_text(self.t("Basculer clair / sombre")).clicked() {
                        self.cfg.theme = if self.cfg.theme == Theme::Sombre { Theme::Clair } else { Theme::Sombre };
                        apply_theme(ctx, self.cfg.theme);
                        self.cfg.save();
                    }
                    egui::ComboBox::from_id_source("lang_quick")
                        .width(60.0)
                        .selected_text(self.cfg.lang.code())
                        .show_ui(ui, |ui| {
                            for l in [Lang::Fr, Lang::En, Lang::Es, Lang::De] {
                                if ui.selectable_value(&mut self.cfg.lang, l, l.name()).clicked() {
                                    self.cfg.save();
                                }
                            }
                        });
                });
            });
        });
    }
}

impl App {
    /// Enregistre le projet courant (dialogue de fichier).
    pub(super) fn save_project_dialog(&mut self) {
        let json = serde_json::to_string_pretty(&self.spec).unwrap_or_default();
        match crate::platform::save(&format!("{}.json", self.spec.name), "FractLab", "json", json.as_bytes()) {
            Some(Ok(p)) => {
                if !crate::platform::WEB {
                    self.cfg.last_project = Some(p.clone());
                    self.cfg.save();
                }
                self.status = format!("{} {}", self.t("Enregistré :"), p);
            }
            Some(Err(e)) => self.err = Some(e),
            None => {}
        }
    }
}
