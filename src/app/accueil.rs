//! Accueil du premier lancement : les étapes d'une étude et trois façons
//! de commencer (démo guidée, ouvrir un projet, partir du projet affiché).

use super::*;

impl App {
    /// Fenêtre d'accueil (premier lancement, ou FRACTLAB_ACCUEIL=1).
    pub(super) fn welcome_window(&mut self, ctx: &egui::Context) {
        if !self.welcome {
            return;
        }
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let mut close = false;
        egui::Window::new(format!("👋 {}", t("Bienvenue dans FractLab"))).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.set_max_width(560.0);
            ui.label(RichText::new(t("FractLab calcule le champ, le couple, la FEM, les forces et les pertes d'une machine à aimants permanents, puis l'optimise. Une étude suit quatre étapes :")).size(15.0));
            ui.add_space(6.0);
            egui::Grid::new("accueil").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                for (icon, name, what) in [
                    ("✏", "Conception", "régler la machine dans le panneau de gauche ; la Synthèse et la jauge disent ce qui va ou non"),
                    ("🔬", "Analyse", "champ, couple de détente, FEM, forces, charge, dynamique"),
                    ("✨", "Optimisation", "Ajustement I.A, puis comparaison des solutions pour choisir"),
                    ("✔", "Validation", "contrôle par un solveur volumes finis indépendant"),
                ] {
                    ui.label(RichText::new(format!("{icon} {}", t(name))).strong().color(CYAN));
                    ui.label(t(what));
                    ui.end_row();
                }
            });
            ui.add_space(8.0);
            ui.label(RichText::new(t("Ctrl+Z annule, F1 ouvre le guide, la loupe du panneau retrouve un paramètre.")).weak());
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                if ui.add(egui::Button::new(RichText::new(format!("▶ {}", t("Démo guidée (recommandé)"))).strong().color(Color32::WHITE)).fill(ACTION).min_size(Vec2::new(0.0, 32.0))).clicked() {
                    self.demo.confirm = true;
                    close = true;
                }
                if !crate::platform::WEB && ui.add(egui::Button::new(format!("📂 {}", t("Ouvrir un projet…"))).min_size(Vec2::new(0.0, 32.0))).clicked() {
                    Pick::Project.ask("FractLab", &["json"]);
                    close = true;
                }
                if ui.add(egui::Button::new(format!("✏ {}", t("Commencer avec ce projet"))).min_size(Vec2::new(0.0, 32.0))).clicked() {
                    close = true;
                }
            });
            ui.add_space(4.0);
            ui.checkbox(&mut self.welcome_never, t("Ne plus afficher au démarrage"));
        });
        if close {
            self.welcome = false;
            if self.welcome_never {
                self.cfg.welcomed = true;
                self.cfg.save();
            }
        }
    }
}
