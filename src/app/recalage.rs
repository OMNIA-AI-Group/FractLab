//! Section « Recalage sur mesures » de l'onglet Validation.

use super::*;
use crate::recalage::{self as rc, MeasKind};

impl App {
    pub(super) fn recalage_ui(&mut self, ui: &mut egui::Ui) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let open = !self.recal_meas.is_empty();
        egui::CollapsingHeader::new(RichText::new(format!("🎯 {}", t("Recalage sur mesures du prototype"))).strong()).default_open(open).show(ui, |ui| {
            ui.label(RichText::new(t("Importez une FEM ou un couple de détente mesurés : Br réelle, entrefer réel et longueur effective (effets 3D) sont ajustés pour que le calcul colle à la mesure. L'origine angulaire de la mesure est recalée automatiquement.")).weak());
            let running = self.recal_job.is_some();
            ui.horizontal_wrapped(|ui| {
                lbl(ui, t("Mesure"));
                rad(ui, &mut self.recal_kind, MeasKind::Emf, t("FEM de phase à vide [V]"));
                rad(ui, &mut self.recal_kind, MeasKind::Cogging, t("Couple de détente [N·m]"));
                ui.separator();
                chk(ui, &mut self.recal_time_x, t("1re colonne en secondes (sinon en degrés)"));
                lbl(ui, t("vitesse de la mesure"));
                ui.add(egui::DragValue::new(&mut self.recal_rpm).clamp_range(1.0..=100000.0).speed(10.0).suffix(" tr/min"));
                if ui.button(format!("📂 {}", t("Importer la mesure (CSV)…"))).clicked() {
                    Pick::Measure.ask("CSV", &["csv", "txt"]);
                }
                if !self.recal_file.is_empty() {
                    ui.label(RichText::new(format!("{} ({} {})", self.recal_file, self.recal_meas.len(), t("points"))).monospace());
                }
            });
            ui.horizontal_wrapped(|ui| {
                lbl(ui, t("Paramètres ajustés :"));
                let mut changed = None;
                for (k, (v, on)) in self.recal_vars.iter_mut().enumerate() {
                    let tip = if *v == rc::FitVar::Airgap { t("l'entrefer change aussi la forme d'onde : il se distingue de Br") } else { t("Br et la longueur ne font que changer l'amplitude : un seul des deux est identifiable, l'autre doit rester fixé") };
                    if ui.checkbox(on, t(v.name())).on_hover_text(tip).changed() && *on {
                        changed = Some(k);
                    }
                }
                // Br et longueur sont indiscernables (seul leur produit compte) : exclusion mutuelle
                if let Some(k) = changed {
                    let v = self.recal_vars[k].0;
                    for (w, on) in self.recal_vars.iter_mut() {
                        if (v == rc::FitVar::Br && *w == rc::FitVar::Length) || (v == rc::FitVar::Length && *w == rc::FitVar::Br) {
                            *on = false;
                        }
                    }
                }
                let vars: Vec<rc::FitVar> = self.recal_vars.iter().filter(|(_, on)| *on).map(|(v, _)| *v).collect();
                if !running {
                    if ui.add_enabled(self.recal_meas.len() >= 4 && !vars.is_empty(), egui::Button::new(RichText::new(format!("▶ {}", t("Lancer le recalage"))).strong())).clicked() {
                        self.recal_vars_run = vars.clone();
                        let setup = rc::FitSetup { kind: self.recal_kind, meas: self.recal_meas.clone(), meas_rpm: self.recal_rpm, vars, budget: 60 };
                        self.recal_job = Some(rc::run(self.spec.clone(), setup));
                    }
                } else {
                    ui.spinner();
                    ui.label(format!("{} {} · {:.2} %", t("essai"), self.recal_prog.0, self.recal_prog.1));
                    if ui.button(format!("⏹ {}", t("Arrêter"))).clicked()
                        && let Some(j) = &mut self.recal_job
                        && j.cancel()
                    {
                        self.recal_job = None;
                    }
                }
            });
            if self.recal_meas.is_empty() {
                ui.label(RichText::new(t("Format : deux colonnes (angle du rotor en degrés ou temps en secondes ; valeur), séparateur « ; », « , » ou tabulation, entête facultative.")).small().weak());
                return;
            }
            let mut apply = false;
            if let Some(r) = &self.recal_res {
                ui.horizontal_wrapped(|ui| {
                    chip(ui, t("écart avant"), &format!("{:.2} %", r.err_initial));
                    chip(ui, t("écart après"), &format!("{:.2} %", r.err_fitted));
                    for (k, v) in self.recal_vars_run.iter().enumerate() {
                        chip(ui, t(v.name()), &format!("{} ➡ {} {}", fnum(r.initial[k]), fnum(r.fitted[k]), v.unit()));
                    }
                    if ui.add(egui::Button::new(RichText::new(format!("✔ {}", t("Appliquer le recalage au projet"))).strong())).clicked() {
                        apply = true;
                    }
                });
            }
            let meas = self.recal_meas.clone();
            let res = self.recal_res.clone();
            let ink = ui.visuals().text_color();
            Plot::new("recal").height(230.0).legend(Legend::default()).x_axis_label(t("angle du rotor [°]")).y_axis_label(if self.recal_kind == MeasKind::Emf { "V" } else { "N·m" }).show(ui, |p| {
                p.points(Points::new(PlotPoints::from(meas.clone())).radius(2.5f32).color(ink).name(t("mesure")));
                if let Some(r) = &res {
                    p.line(Line::new(PlotPoints::from(r.curve_initial.clone())).color(SERIES[0]).style(egui_plot::LineStyle::dashed_loose()).name(t("calcul initial")));
                    p.line(Line::new(PlotPoints::from(r.curve_fitted.clone())).color(ORANGE).width(2.0f32).name(t("calcul recalé")));
                }
            });
            if apply && let Some(r) = res {
                for (v, x) in self.recal_vars_run.clone().iter().zip(&r.fitted) {
                    v.set(&mut self.spec, *x);
                }
                self.touched(ui.ctx());
                self.status = t("Recalage appliqué au projet").into();
            }
        });
    }
}
