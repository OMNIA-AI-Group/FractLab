//! Bandeau des critères : un clic ouvre un tableau qui
//! explique, pour chaque critère non tenu, la valeur, le seuil, la cause et
//! le remède (avec un bouton d'action quand une correction directe existe).
//! Dans la Synthèse, le bandeau reprend tous les critères ; dans les autres
//! onglets, seulement ceux qui les concernent.

use super::*;

/// Correction directe proposée.
#[derive(Clone, Copy, PartialEq)]
enum Action {
    /// Réduire l'arc des aimants au maximum sans chevauchement.
    AdjustArc,
    /// Épaissir la culasse du rotor à l'épaisseur donnée [mm].
    RotorYoke(f64),
    /// Épaissir la culasse du stator à l'épaisseur donnée [mm].
    StatorYoke(f64),
    /// Élargir les dents à la largeur donnée [mm].
    ToothWidth(f64),
    /// Ouvrir l'Ajustement I.A.
    OpenIa,
    /// Ouvrir l'onglet Comparaison.
    Compare,
}

/// Un critère non tenu.
struct Crit {
    tab: Tab,
    name: &'static str,
    value: String,
    threshold: String,
    why: &'static str,
    todo: &'static str,
    action: Option<(Action, String)>,
}

impl App {
    /// Critères non tenus du projet courant.
    fn failed_criteria(&self) -> Vec<Crit> {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let Some(r) = &self.res else { return Vec::new() };
        let rep = &r.report;
        let u = &self.cfg.units;
        let mut v = Vec::new();
        let gap = rep.min_magnet_gap_deg;
        if gap < 0.0 {
            v.push(Crit {
                tab: Tab::Geometrie,
                name: "Faisabilité des aimants",
                value: format!("{gap:+.3}°"),
                threshold: "≥ 0°".into(),
                why: "Deux aimants voisins se chevauchent : le rotor ne peut pas être monté et les résultats ne sont pas physiques dans ces zones.",
                todo: "Réduire l'arc des aimants ou l'écart des aimants à leur position régulière.",
                action: Some((Action::AdjustArc, t("Ajuster l'arc").into())),
            });
        }
        let crit = crate::repartition::criteria();
        let b0 = crit.cogging_target.and_then(|(n, _)| self.cmp.iter().find(|c| c.name == n)).map(|c| c.report.cogging.cogging_pp_mnm);
        if let (Some(b), Some((n, pct))) = (b0, crit.cogging_target)
            && rep.cogging.cogging_pp_mnm > (1.0 + pct / 100.0) * b
        {
            v.push(Crit {
                tab: Tab::Cogging,
                name: "Cogging crête-à-crête",
                value: format!("{:+.1} % vs {n}", 100.0 * (rep.cogging.cogging_pp_mnm / b - 1.0)),
                threshold: format!("≤ {pct:+.0} % vs {n}"),
                why: "La répartition des aimants ne réduit pas assez le couple de détente par rapport au rotor de référence.",
                todo: "Essayer une autre variante, ou l'Ajustement I.A avec l'objectif « couple de détente minimal ».",
                action: Some((Action::OpenIa, t("Ajustement I.A").into())),
            });
        }
        let axial = r.spec.topology == crate::geometry::Topology::Axial;
        if let Some(fm) = crit.force_max_n
            && rep.force_max_n >= fm
        {
            v.push(Crit {
                tab: Tab::Forces,
                name: if axial { "Force transverse max" } else { "Force radiale max" },
                value: format!("{} {}", fnum(u.show(Q::Force, rep.force_max_n)), u.sym(Q::Force)),
                threshold: format!("< {} {}", fnum(u.show(Q::Force, fm)), u.sym(Q::Force)),
                why: "Une répartition non périodique casse la symétrie du rotor : les forces magnétiques ne s'annulent plus sur le tour. Cette force fait vibrer le rotor et use les roulements.",
                todo: "Rapprocher les aimants de leur position régulière, ou l'Ajustement I.A avec l'objectif « force radiale minimale ». Comparer les répartitions.",
                action: Some((Action::Compare, t("Comparaison").into())),
            });
        }
        if let Some(bm) = crit.balourd_max_mg_cm
            && rep.balourd_mg_cm >= bm
        {
            v.push(Crit {
                tab: Tab::Geometrie,
                name: "Balourd",
                value: format!("{} {}", fnum(u.show(Q::Unbalance, rep.balourd_mg_cm)), u.sym(Q::Unbalance)),
                threshold: format!("< {} {}", fnum(u.show(Q::Unbalance, bm)), u.sym(Q::Unbalance)),
                why: "La masse des aimants n'est pas répartie également autour de l'axe : le rotor vibre à chaque tour.",
                todo: "Positions personnalisées, puis « Rééquilibrer ».",
                action: None,
            });
        }
        if let (Some(a), Some((am, plancher))) = (rep.autocorr_max, crit.autocorr_max)
            && a >= am
        {
            v.push(Crit {
                tab: Tab::Geometrie,
                name: "Autocorrélation des écarts",
                value: format!("{a:.3}"),
                threshold: format!("< {} ({} ≈ {})", fnum(am), t("plancher"), fnum(plancher)),
                why: "Les écarts entre aimants se répètent trop : la répartition n'est pas assez apériodique.",
                todo: "Critère indicatif : avec peu de pôles, le plancher atteignable est proche du seuil. Plus de pôles l'abaisse.",
                action: None,
            });
        }
        if let Some(tm) = crit.thd_max_pct
            && rep.emf_thd_pct > tm
        {
            v.push(Crit {
                tab: Tab::Fem,
                name: "THD de la FEM",
                value: format!("{:.2} %", rep.emf_thd_pct),
                threshold: format!("≤ {} %", fnum(tm)),
                why: "La FEM n'est pas assez sinusoïdale : la forme des aimants et les encoches ajoutent des harmoniques (bruit, ondulation de couple).",
                todo: "Ajuster l'arc des aimants (≈ 0,7 à 0,8) ou l'ouverture d'encoche ; l'aimantation Halbach rend la FEM plus sinusoïdale.",
                action: None,
            });
        }
        let sat = &r.sat.at_load;
        let (kst, kro) = (r.sat.knee, r.spec.saturation.rotor.knee());
        let s = &r.spec;
        // pire partie : on propose l'épaisseur qui ramène l'induction à 1,5 T
        let parts = [
            (sat.b_rotor - kro, sat.b_rotor, Action::RotorYoke(s.rotor_yoke_th * sat.b_rotor / 1.5), "Épaissir la culasse du rotor à"),
            (sat.b_yoke - kst, sat.b_yoke, Action::StatorYoke(s.yoke_th * sat.b_yoke / 1.5), "Épaissir la culasse du stator à"),
            (sat.b_tooth - kst, sat.b_tooth, Action::ToothWidth(s.tooth_w * sat.b_tooth / 1.5), "Élargir les dents à"),
        ];
        if let Some(w) = parts.iter().filter(|p| p.0 > 0.1).max_by(|a, b| a.0.total_cmp(&b.0)) {
            let mm = match w.2 {
                Action::RotorYoke(x) | Action::StatorYoke(x) | Action::ToothWidth(x) => x,
                _ => 0.0,
            };
            v.push(Crit {
                tab: Tab::Charge,
                name: "Saturation du fer (en charge)",
                value: format!("{:.2} T", w.1),
                threshold: format!("≤ {:.1} T", kst.max(kro)),
                why: "L'induction dépasse le coude de la courbe B(H) : le fer ne transporte plus bien le flux, la FEM et le couple baissent, les pertes fer augmentent.",
                todo: "Épaissir la partie saturée (voir l'onglet Charge) ou choisir un matériau de meilleure qualité.",
                action: Some((w.2, format!("{} {:.2} mm", t(w.3), mm))),
            });
        }
        v
    }

    /// Bandeau des critères non tenus pour l'onglet `tab` (tous pour la Synthèse).
    pub(super) fn criteria_banner(&mut self, ui: &mut egui::Ui, tab: Tab) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let all = self.failed_criteria();
        let crits: Vec<&Crit> = all.iter().filter(|c| tab == Tab::Synthese || c.tab == tab).collect();
        if crits.is_empty() {
            return;
        }
        let names: Vec<&str> = crits.iter().map(|c| t(c.name)).collect();
        let mut act: Option<Action> = None;
        let mut goto: Option<Tab> = None;
        egui::Frame::none().fill(Color32::from_rgba_unmultiplied(255, 70, 70, 28)).rounding(8.0).inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let arrow = if self.alerts_open { "⏶" } else { "⏷" };
            let head = ui
                .add(egui::Label::new(RichText::new(format!("⚠ {} ({}) : {}   {arrow} {}", t("Critères non tenus"), crits.len(), names.join(" · "), if self.alerts_open { t("masquer") } else { t("pourquoi et que faire ?") })).color(Color32::from_rgb(255, 110, 100)).strong()).sense(egui::Sense::click()))
                .on_hover_text(t("Cliquez pour voir, critère par critère, la valeur, le seuil, la cause et la correction."))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if head.clicked() {
                self.alerts_open = !self.alerts_open;
            }
            if !self.alerts_open {
                return;
            }
            ui.add_space(4.0);
            ui.label(RichText::new(t("Ce bandeau rappelle les critères que le projet actuel ne tient pas. Il se met à jour à chaque calcul et disparaît quand tous sont tenus.")).small());
            ui.add_space(4.0);
            // colonnes : nom, valeur, seuil, pourquoi, que faire, action ; chaque
            // ligne prend la hauteur de sa cellule la plus haute
            let full = ui.available_width();
            let fixed = [170.0, 80.0, 70.0, 0.0, 0.0, 230.0];
            let wtxt = ((full - fixed.iter().sum::<f32>() - 6.0 * 10.0) / 2.0).max(160.0);
            let widths = [fixed[0], fixed[1], fixed[2], wtxt, wtxt, fixed[5]];
            let cell = |ui: &mut egui::Ui, w: f32, add: &mut dyn FnMut(&mut egui::Ui)| {
                ui.allocate_ui_with_layout(Vec2::new(w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_width(w);
                    add(ui);
                });
            };
            ui.horizontal_top(|ui| {
                for (k, h) in ["Critère", "Valeur", "Seuil", "Pourquoi", "Que faire", ""].iter().enumerate() {
                    cell(ui, widths[k], &mut |ui| {
                        ui.label(RichText::new(t(h)).strong());
                    });
                }
            });
            for c in &crits {
                ui.separator();
                ui.horizontal_top(|ui| {
                    cell(ui, widths[0], &mut |ui| {
                        if ui.link(RichText::new(t(c.name)).strong()).on_hover_text(t("Ouvrir l'onglet concerné")).clicked() {
                            goto = Some(c.tab);
                        }
                    });
                    cell(ui, widths[1], &mut |ui| {
                        ui.label(RichText::new(&c.value).color(Color32::from_rgb(255, 110, 100)).monospace());
                    });
                    cell(ui, widths[2], &mut |ui| {
                        ui.label(RichText::new(&c.threshold).monospace());
                    });
                    cell(ui, widths[3], &mut |ui| {
                        ui.add(egui::Label::new(t(c.why)).wrap(true));
                    });
                    cell(ui, widths[4], &mut |ui| {
                        ui.add(egui::Label::new(t(c.todo)).wrap(true));
                    });
                    cell(ui, widths[5], &mut |ui| {
                        if let Some((a, label)) = &c.action
                            && ui.add(egui::Button::new(label.as_str()).wrap(true)).clicked()
                        {
                            act = Some(*a);
                        }
                    });
                });
            }
        });
        ui.add_space(6.0);
        if let Some(g) = goto {
            self.tab = g;
            self.full = None;
        }
        if let Some(a) = act {
            match a {
                Action::AdjustArc => self.spec.magnet_arc_ratio = (geometry::max_arc_ratio(&self.spec, 0.1) * 1000.0).floor() / 1000.0,
                Action::RotorYoke(x) => self.spec.rotor_yoke_th = (x * 100.0).ceil() / 100.0,
                Action::StatorYoke(x) => self.spec.yoke_th = (x * 100.0).ceil() / 100.0,
                Action::ToothWidth(x) => self.spec.tooth_w = (x * 100.0).ceil() / 100.0,
                Action::OpenIa => self.show_opt = true,
                Action::Compare => self.tab = Tab::Comparaison,
            }
            if !matches!(a, Action::OpenIa | Action::Compare) {
                self.touched(ui.ctx());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// Toutes les explications du tableau (nom, pourquoi, que faire, en-têtes,
    /// libellés d'action) sont traduites dans les 4 langues.
    #[test]
    fn explications_traduites() {
        let src = include_str!("criteres.rs");
        let mut keys: Vec<String> = Vec::new();
        for pat in ["name: \"", "why: \"", "todo: \"", "\"Épaissir", "\"Élargir"] {
            let mut from = 0;
            while let Some(k) = src[from..].find(pat) {
                let start = from + k + pat.len() - if pat.starts_with('"') { pat.len() - 1 } else { 0 };
                let end = start + src[start..].find('"').unwrap();
                keys.push(src[start..end].to_string());
                from = end;
            }
        }
        keys.retain(|k| !k.contains('{') && !k.is_empty() && k != "Épaissir" && k != "Élargir");
        for h in ["Critère", "Valeur", "Seuil", "Pourquoi", "Que faire"] {
            keys.push(h.into());
        }
        let manq: Vec<&String> = keys.iter().filter(|k| !crate::i18n::TABLE.iter().any(|(f, _, _, _)| f == k)).collect();
        assert!(manq.is_empty(), "{manq:#?}");
    }
}
