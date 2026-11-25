//! Onglet « Paramétrique » : calcul en lot sur un paramètre, courbes de
//! toutes les grandeurs clés, tableau, export CSV et application d'un point.

use super::*;
use crate::sweep::{self, OUTPUTS};

impl App {
    /// Lance (ou relance) l'étude paramétrique sur le projet courant.
    pub(super) fn start_sweep(&mut self) {
        // les anciens points restent affichés jusqu'à l'arrivée du premier nouveau
        self.sweep_replace = true;
        self.sweep_job = Some(compute::run_sweep(self.spec.clone(), self.st, self.sweep_var, self.sweep_from, self.sweep_to, self.sweep_n));
    }

    pub(super) fn tab_parametrique(&mut self, ui: &mut egui::Ui) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        ui.label(RichText::new(t("Un paramètre varie entre deux bornes : le moteur complet est recalculé à chaque valeur. L'étude se relance seule quand le projet change.")).weak());
        let running = self.sweep_job.is_some();
        let mut apply: Option<f64> = None;
        ui.horizontal_wrapped(|ui| {
            lbl(ui, t("Paramètre"));
            let before = self.sweep_var;
            egui::ComboBox::from_id_source("sweep_var").selected_text(t(self.sweep_var.name())).width(240.0).show_ui(ui, |ui| {
                for v in sweep::ALL {
                    ui.selectable_value(&mut self.sweep_var, v, t(v.name()));
                }
            });
            if before != self.sweep_var {
                (self.sweep_from, self.sweep_to) = self.sweep_var.default_range(&self.spec);
                self.sweep_rows.clear();
            }
            let unit = self.sweep_var.unit();
            lbl(ui, t("de"));
            ui.add(egui::DragValue::new(&mut self.sweep_from).speed(0.01).max_decimals(4).suffix(format!(" {unit}")));
            lbl(ui, t("à"));
            ui.add(egui::DragValue::new(&mut self.sweep_to).speed(0.01).max_decimals(4).suffix(format!(" {unit}")));
            lbl(ui, t("points"));
            ui.add(egui::DragValue::new(&mut self.sweep_n).clamp_range(3..=101));
            if ui.add_enabled(!running, egui::Button::new(RichText::new(format!("▶ {}", t("Lancer l'étude"))).strong())).clicked() {
                self.start_sweep();
            }
            if running {
                ui.spinner();
                ui.label(format!("{} / {}", self.sweep_rows.len(), self.sweep_n));
            }
            if !self.sweep_rows.is_empty() && ui.button(format!("⬇ {}", t("Exporter CSV"))).clicked() {
                let csv = sweep::csv(self.sweep_var, &self.sweep_rows);
                self.export_one("etude_parametrique.csv".into(), "CSV", "csv", csv.as_bytes());
            }
        });
        ui.horizontal_wrapped(|ui| {
            lbl(ui, t("Grandeurs tracées :"));
            for (k, (n, _)) in OUTPUTS.iter().enumerate() {
                ui.checkbox(&mut self.sweep_show[k], t(n));
            }
        });
        ui.separator();
        if self.sweep_rows.is_empty() {
            lbl(ui, t("Choisissez un paramètre et sa plage, puis « Lancer l'étude »."));
            return;
        }
        let cur = self.sweep_var.get(&self.spec);
        let shown: Vec<usize> = (0..OUTPUTS.len()).filter(|k| self.sweep_show[*k]).collect();
        let unit = self.sweep_var.unit();
        let pname = if unit.is_empty() { t(self.sweep_var.name()).to_string() } else { format!("{} [{unit}]", t(self.sweep_var.name())) };
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let ncol = if ui.available_width() > 900.0 { 2 } else { 1 };
            let w = (ui.available_width() - 12.0 * (ncol as f32 - 1.0)) / ncol as f32;
            egui::Grid::new("sweep_plots").num_columns(ncol).spacing([12.0, 12.0]).show(ui, |ui| {
                for (i, k) in shown.iter().enumerate() {
                    let (n, u) = OUTPUTS[*k];
                    let pts: Vec<[f64; 2]> = self.sweep_rows.iter().map(|(v, o)| [*v, o[*k]]).collect();
                    ui.vertical(|ui| {
                        ui.set_width(w);
                        ui.label(RichText::new(format!("{} [{u}]", t(n))).strong());
                        Plot::new(("sweep", *k)).height(190.0).width(w).x_axis_label(pname.clone()).allow_scroll(false).show(ui, |p| {
                            p.line(Line::new(PlotPoints::from(pts.clone())).color(SERIES[i % 6]).width(2.0f32));
                            p.points(Points::new(PlotPoints::from(pts)).radius(3.0f32).color(SERIES[i % 6]));
                            p.vline(VLine::new(cur).color(ORANGE).width(1.0f32));
                        });
                    });
                    if i % ncol == ncol - 1 {
                        ui.end_row();
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(RichText::new(t("Tableau des résultats (la ligne orange verticale des courbes = valeur actuelle du projet)")).strong());
            egui::Grid::new("sweep_tab").striped(true).num_columns(OUTPUTS.len() + 2).spacing([10.0, 2.0]).show(ui, |ui| {
                ui.label(RichText::new(&pname).small().strong());
                for (n, u) in OUTPUTS {
                    ui.label(RichText::new(format!("{} [{u}]", t(n))).small().strong());
                }
                ui.label("");
                ui.end_row();
                for (v, o) in &self.sweep_rows {
                    ui.label(RichText::new(fnum(*v)).monospace());
                    for x in o {
                        ui.label(RichText::new(fnum(*x)).monospace().small());
                    }
                    if ui.small_button(t("Appliquer")).clicked() {
                        apply = Some(*v);
                    }
                    ui.end_row();
                }
            });
        });
        if let Some(v) = apply {
            self.sweep_var.set(&mut self.spec, v);
            self.touched(ui.ctx());
            self.status = format!("{} = {}", t(self.sweep_var.name()), fnum(v));
        }
        if running {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(150));
        }
    }
}
