//! Onglets d'étude : dynamique, comparaison, moteur 3D animé, cartes 3D, validation.

use super::*;

impl App {
    // ----------------------------- Dynamique ---------------------------
    pub(super) fn tab_dynamique(&mut self, ui: &mut egui::Ui) {
        use crate::dynsim::{Fault, Load, Supply};
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        if self.sim_rx.is_some() {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }
        let st = &mut self.sim_setup;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t("Alimentation")).strong());
            let sn = |s: &Supply| match s {
                Supply::Grid { .. } => t("Réseau sinusoïdal"),
                Supply::InverterSine { .. } => t("Onduleur MLI autopiloté"),
                Supply::SixStep { .. } => t("Commutation 120° (ESC)"),
                Supply::DiodeRectifier { .. } => t("Redresseur à diodes (génératrice)"),
            };
            egui::ComboBox::from_id_source("supply").selected_text(sn(&st.supply)).show_ui(ui, |ui| {
                for c in [
                    Supply::Grid { v_ll: 12.0, f_hz: 500.0 },
                    Supply::InverterSine { vdc: 16.0, m: 0.8, advance_deg: 0.0 },
                    Supply::SixStep { vdc: 16.0, duty: 0.6, advance_deg: 0.0 },
                    Supply::DiodeRectifier { c_bus: 1e-3, r_load: 10.0 },
                ] {
                    let same = std::mem::discriminant(&c) == std::mem::discriminant(&st.supply);
                    if ui.selectable_label(same, sn(&c)).clicked() && !same {
                        st.supply = c;
                    }
                }
            });
            match &mut st.supply {
                Supply::Grid { v_ll, f_hz } => {
                    ui.add(egui::DragValue::new(v_ll).speed(0.1).suffix(" V"));
                    ui.add(egui::DragValue::new(f_hz).speed(1.0).suffix(" Hz"));
                }
                Supply::InverterSine { vdc, m, advance_deg } => {
                    ui.add(egui::DragValue::new(vdc).speed(0.1).suffix(" V"));
                    ui.add(egui::DragValue::new(m).speed(0.01).clamp_range(0.0..=1.15).prefix("m = "));
                    ui.add(egui::DragValue::new(advance_deg).speed(0.5).suffix(" °él"));
                }
                Supply::SixStep { vdc, duty, advance_deg } => {
                    ui.add(egui::DragValue::new(vdc).speed(0.1).suffix(" V"));
                    ui.add(egui::DragValue::new(duty).speed(0.01).clamp_range(0.0..=1.0).prefix("d = "));
                    ui.add(egui::DragValue::new(advance_deg).speed(0.5).suffix(" °él"));
                }
                Supply::DiodeRectifier { c_bus, r_load } => {
                    let mut c = *c_bus * 1e6;
                    if ui.add(egui::DragValue::new(&mut c).speed(10.0).suffix(" µF")).changed() {
                        *c_bus = c * 1e-6;
                    }
                    ui.add(egui::DragValue::new(r_load).speed(0.1).suffix(" Ω"));
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t("Charge mécanique")).strong());
            let ln = |l: &Load| match l {
                Load::Free => t("Aucune"),
                Load::Constant(_) => t("Couple constant"),
                Load::Quadratic(_) => t("Hélice / ventilateur / pompe (k·ω²)"),
                Load::Conveyor { .. } => t("Convoyeur"),
                Load::WindTurbine { .. } => t("Éolienne (entraînement)"),
            };
            egui::ComboBox::from_id_source("load").selected_text(ln(&st.load)).show_ui(ui, |ui| {
                for c in [Load::Free, Load::Constant(0.05), Load::Quadratic(2e-8), Load::Conveyor { t0: 0.05, visc: 1e-5 }, Load::WindTurbine { radius: 0.12, wind: 12.0 }] {
                    let same = std::mem::discriminant(&c) == std::mem::discriminant(&st.load);
                    if ui.selectable_label(same, ln(&c)).clicked() && !same {
                        st.load = c;
                    }
                }
            });
            match &mut st.load {
                Load::Constant(c) => {
                    ui.add(egui::DragValue::new(c).speed(0.001).suffix(" N·m"));
                }
                Load::Quadratic(k) => {
                    let mut v = *k * 1e9;
                    if ui.add(egui::DragValue::new(&mut v).speed(0.1).prefix("k = ").suffix(" e-9 N·m·s²")).changed() {
                        *k = v * 1e-9;
                    }
                }
                Load::Conveyor { t0, visc } => {
                    ui.add(egui::DragValue::new(t0).speed(0.001).suffix(" N·m"));
                    let mut v = *visc * 1e6;
                    if ui.add(egui::DragValue::new(&mut v).speed(0.1).suffix(" µN·m·s")).changed() {
                        *visc = v * 1e-6;
                    }
                }
                Load::WindTurbine { radius, wind } => {
                    ui.add(egui::DragValue::new(radius).speed(0.01).suffix(" m"));
                    ui.add(egui::DragValue::new(wind).speed(0.1).suffix(" m/s"));
                }
                Load::Free => {}
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t("Défaut")).strong());
            let fname = |f: &Fault| match f {
                Fault::None => t("Machine saine"),
                Fault::OpenPhase(_) => t("Phase ouverte"),
                Fault::InterTurn { .. } => t("Court-circuit entre spires (phase A)"),
            };
            egui::ComboBox::from_id_source("fault").selected_text(fname(&st.fault)).show_ui(ui, |ui| {
                for c in [Fault::None, Fault::OpenPhase(0), Fault::InterTurn { mu: 0.1, r_f: 0.01 }] {
                    let same = std::mem::discriminant(&c) == std::mem::discriminant(&st.fault);
                    if ui.selectable_label(same, fname(&c)).clicked() && !same {
                        st.fault = c;
                    }
                }
            });
            match &mut st.fault {
                Fault::OpenPhase(k) => {
                    egui::ComboBox::from_id_source("fph").width(40.0).selected_text(["A", "B", "C"][*k]).show_ui(ui, |ui| {
                        for i in 0..3 {
                            ui.selectable_value(k, i, ["A", "B", "C"][i]);
                        }
                    });
                }
                Fault::InterTurn { mu, r_f } => {
                    ui.add(egui::DragValue::new(mu).speed(0.005).clamp_range(0.0..=1.0).prefix("μ = "));
                    ui.add(egui::DragValue::new(r_f).speed(0.001).suffix(" Ω"));
                }
                Fault::None => {}
            }
            if st.fault != Fault::None {
                ui.add(egui::DragValue::new(&mut st.fault_t).speed(0.01).prefix(format!("{} ", t("à t ="))).suffix(" s"));
            }
            ui.separator();
            lbl(ui, t("Durée"));
            ui.add(egui::DragValue::new(&mut st.duration).speed(0.01).clamp_range(0.01..=10.0).suffix(" s"));
            lbl(ui, t("Vitesse initiale"));
            ui.add(egui::DragValue::new(&mut st.rpm0).speed(10.0).suffix(" tr/min"));
        });
        ui.horizontal(|ui| {
            let busy = self.sim_rx.is_some();
            if ui.add_enabled(!busy && self.res.is_some(), egui::Button::new(RichText::new(format!("▶ {}", t("Simuler"))).strong())).clicked() {
                self.start_sim();
            }
            if busy {
                ui.spinner();
                lbl(ui, t("Simulation en cours…"));
            }
            ui.label(RichText::new(t("Modèle à inductance synchrone constante et flux d'aimants issus du calcul de champ ; défauts en modèle simplifié.")).small().weak());
        });
        let Some(o) = &self.sim_res else {
            lbl(ui, t("Réglez l'alimentation, la charge et un éventuel défaut, puis « Simuler »."));
            return;
        };
        let n = o.t.len();
        if n < 2 {
            return;
        }
        let last = |v: &Vec<f64>| v[n - 1];
        let tail = n * 9 / 10;
        let mean = |v: &Vec<f64>| v[tail..].iter().sum::<f64>() / (n - tail) as f64;
        let ipk = (0..3).map(|k| o.i[k][tail..].iter().fold(0.0f64, |a, v| a.max(v.abs()))).fold(0.0, f64::max);
        ui.horizontal_wrapped(|ui| {
            chip(ui, t("vitesse finale"), &format!("{:.0} tr/min", last(&o.rpm)));
            chip(ui, t("couple moyen"), &format!("{:.4} N·m", mean(&o.te)));
            chip(ui, t("puissance électrique"), &format!("{:.2} W", mean(&o.p_elec)));
            chip(ui, t("courant crête"), &format!("{ipk:.2} A"));
            if o.vdc.iter().any(|v| *v != 0.0) {
                chip(ui, t("bus continu"), &format!("{:.2} V", last(&o.vdc)));
            }
            if o.i_fault.iter().any(|v| *v != 0.0) {
                chip(ui, t("courant de défaut"), &format!("{:.2} A", o.i_fault[tail..].iter().fold(0.0f64, |a, v| a.max(v.abs()))));
            }
            chip(ui, t("pas de calcul"), &format!("{:.2} µs", o.dt * 1e6));
        });
        let tt = |v: &Vec<f64>| -> Vec<[f64; 2]> { o.t.iter().zip(v).map(|(a, b)| [*a, *b]).collect() };
        let h = (ui.available_height() / 2.0 - 8.0).max(110.0);
        ui.columns(2, |c| {
            Plot::new("dyn_w").height(h).legend(Legend::default()).x_axis_label("t [s]").y_axis_label("tr/min").show(&mut c[0], |p| {
                p.line(Line::new(PlotPoints::from(tt(&o.rpm))).color(CYAN).name(t("vitesse")));
            });
            Plot::new("dyn_t").height(h).legend(Legend::default()).x_axis_label("t [s]").y_axis_label("N·m").show(&mut c[1], |p| {
                p.line(Line::new(PlotPoints::from(tt(&o.te))).color(ORANGE).name(t("couple électromagnétique")));
            });
        });
        ui.columns(2, |c| {
            Plot::new("dyn_i").height(h).legend(Legend::default()).x_axis_label("t [s]").y_axis_label("A").show(&mut c[0], |p| {
                for k in 0..3 {
                    p.line(Line::new(PlotPoints::from(tt(&o.i[k]))).color(SERIES[k]).name(format!("i{}", ["A", "B", "C"][k])));
                }
                if o.i_fault.iter().any(|v| *v != 0.0) {
                    p.line(Line::new(PlotPoints::from(tt(&o.i_fault))).color(SERIES[3]).name(t("courant de défaut")));
                }
            });
            Plot::new("dyn_v").height(h).legend(Legend::default()).x_axis_label("t [s]").y_axis_label("V").show(&mut c[1], |p| {
                p.line(Line::new(PlotPoints::from(tt(&o.va))).color(SERIES[4]).name("vA"));
                if o.vdc.iter().any(|v| *v != 0.0) {
                    p.line(Line::new(PlotPoints::from(tt(&o.vdc))).color(SERIES[2]).name(t("bus continu")));
                }
            });
        });
    }

    // ---------------------------- Comparaison -------------------------
    pub(super) fn tab_comparaison(&mut self, ui: &mut egui::Ui) {
        let u = self.cfg.units.clone();
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        ui.horizontal(|ui| {
            if ui.add_enabled(self.cmp_job.is_none(), egui::Button::new(RichText::new(format!("📊 {}", t("Comparer toutes les variantes"))).strong())).clicked() {
                self.cmp_job = Some(compute::run_compare(self.spec.clone(), self.st));
            }
            ui.label(RichText::new(t("mêmes dimensions, même fenêtre de cogging (360°/Q), seule la répartition change")).weak());
        });
        if self.cmp.is_empty() {
            lbl(ui, t("Aucune comparaison calculée."));
            return;
        }
        let ft = u.factor(Q::Torque);
        let series: Vec<(String, Vec<[f64; 2]>, Color32)> = self
            .cmp
            .iter()
            .enumerate()
            .map(|(k, v)| (v.name.clone(), v.cog.angles_deg.iter().zip(&v.cog.torque_nm).map(|(a, y)| [*a, y * ft]).collect(), SERIES[k % 6]))
            .collect();
        let refname = crate::repartition::reference_name();
        let b0 = self.cmp.iter().find(|v| v.name == refname).map(|v| v.report.cogging.cogging_pp_mnm);
        let h = ui.available_height() * 0.45;
        let mut cur = *self.cursors.entry("cmp").or_default();
        curve_plot(ui, "cmp", &series, "θr [°]", &format!("{} [{}]", t("couple de détente"), u.sym(Q::Torque)), h, &mut cur);
        self.cursors.insert("cmp", cur);
        ui.columns(2, |c| {
            let bars: Vec<Bar> = self
                .cmp
                .iter()
                .enumerate()
                .map(|(k, v)| Bar::new(k as f64, u.show(Q::Torque, v.report.cogging.cogging_pp_mnm * 1e-3)).width(0.6).fill(SERIES[k % 6]).name(&v.name))
                .collect();
            let names: Vec<String> = self.cmp.iter().map(|v| v.name.clone()).collect();
            let sym = u.sym(Q::Torque);
            Plot::new("cmp_bars")
                .height(c[0].available_height())
                .y_axis_label(format!("{} [{}]", t("cogging crête-à-crête"), sym))
                .x_axis_formatter(move |m, _, _| names.get(m.value.round() as usize).cloned().unwrap_or_default())
                .show(&mut c[0], |p| p.bar_chart(BarChart::new(bars).element_formatter(Box::new(move |b, _| format!("{} : {:.4} {sym}", b.name, b.value)))));
            egui::ScrollArea::both().id_source("cmp_tab").show(&mut c[1], |ui| {
                egui::Grid::new("cmp_grid").striped(true).num_columns(8).spacing([12.0, 4.0]).show(ui, |ui| {
                    let vs_ref = format!("{} {}", t("vs"), crate::repartition::reference_name());
                    for h in [t("Variante"), t("Cogging pp"), vs_ref.as_str(), t("Part dom."), t("Entropie"), t("Force max"), "THD", t("Autocorr.")] {
                        ui.label(RichText::new(h).strong());
                    }
                    ui.end_row();
                    for v in &self.cmp {
                        let r = &v.report;
                        ui.label(RichText::new(&v.name).strong());
                        ui.label(format!("{} {}", fnum(u.show(Q::Torque, r.cogging.cogging_pp_mnm * 1e-3)), u.sym(Q::Torque)));
                        match b0 {
                            Some(b) => {
                                let d = 100.0 * (r.cogging.cogging_pp_mnm / b - 1.0);
                                ui.label(RichText::new(format!("{d:+.1} %")).color(badge(d <= -30.0)))
                            }
                            None => ui.label("—"),
                        };
                        ui.label(format!("{:.1} %", r.cogging.dominant_share_pct));
                        ui.label(format!("{:.3}", r.cogging.spectral_entropy));
                        ui.label(RichText::new(format!("{} {}", fnum(u.show(Q::Force, r.force_max_n)), u.sym(Q::Force))).color(badge(r.force_max_n < 0.5)));
                        ui.label(RichText::new(format!("{:.2} %", r.emf_thd_pct)).color(badge(r.emf_thd_pct <= 5.0)));
                        ui.label(r.autocorr_max.map(|a| format!("{a:.3}")).unwrap_or("—".into()));
                        ui.end_row();
                    }
                });
            });
        });
    }

    // ----------------------------- Moteur 3D --------------------------
    pub(super) fn tab_moteur3d(&mut self, ui: &mut egui::Ui) {
        // fonction phare : bien en vue, en tête de l'onglet
        ui.horizontal(|ui| {
            self.field3d_button(ui, true);
        });
        ui.add_space(4.0);
        use crate::dynamics as dy;
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let u = self.cfg.units.clone();
        // --- commandes d'animation ---
        ui.horizontal_wrapped(|ui| {
            let play_lbl = if self.anim3d { format!("⏸ {}", t("Pause")) } else { format!("▶ {}", t("Lecture")) };
            if ui.add(egui::Button::new(RichText::new(play_lbl).strong()).fill(Color32::from_rgb(0, 110, 160))).clicked() {
                self.anim3d = !self.anim3d;
            }
            ui.separator();
            lbl(ui, t("Vitesse du moteur"));
            let fs = u.factor(Q::Speed);
            let mut v = self.anim_rpm * fs;
            if ui.add(egui::Slider::new(&mut v, 10.0 * fs..=60000.0 * fs).logarithmic(true).max_decimals(1).suffix(format!(" {}", u.sym(Q::Speed)))).changed() {
                self.anim_rpm = v / fs;
            }
            ui.separator();
            lbl(ui, t("Ralenti"));
            ui.add(egui::Slider::new(&mut self.slowmo, 1e-5..=1.0).logarithmic(true).custom_formatter(|v, _| if v >= 1.0 { "×1".into() } else { format!("×1/{:.0}", 1.0 / v) }).show_value(true));
            if ui.small_button("÷2").on_hover_text(t("Ralentir")).clicked() {
                self.slowmo = (self.slowmo / 2.0).max(1e-5);
            }
            if ui.small_button("×2").on_hover_text(t("Accélérer")).clicked() {
                self.slowmo = (self.slowmo * 2.0).min(1.0);
            }
            let shown = self.anim_rpm / 60.0 * self.slowmo;
            ui.label(RichText::new(format!("1 s {} = {} ms {} · {:.3} {}", t("à l'écran"), fnum(self.slowmo * 1000.0), t("réelles"), shown, t("tr/s affichés"))).weak().small());
        });
        ui.horizontal_wrapped(|ui| {
            lbl(ui, t("Position du rotor θr"));
            ui.add(egui::Slider::new(&mut self.theta_r, 0.0..=360.0).suffix("°").max_decimals(2));
            chk(ui, &mut self.teeth_by_b, t("Dents colorées par l'induction"));
            chk(ui, &mut self.show_ribbon, t("Ruban de l'induction d'entrefer (couleur = Br)"));
            chk(ui, &mut self.show_force, t("Vecteur de la force radiale"));
        });
        ui.horizontal_wrapped(|ui| {
            chk(ui, &mut self.vis_transparent, t("Vue transparente"));
            if self.vis_transparent {
                lbl(ui, t("opacité"));
                ui.add(egui::Slider::new(&mut self.vis_opacity, 0.05..=1.0).max_decimals(2));
            }
            ui.separator();
            lbl(ui, t("Champ :"));
            chk(ui, &mut self.vis_field_map, t("carte de l'induction"));
            let axial = self.spec.topology == Topology::Axial;
            ui.add_enabled_ui(!axial, |ui| {
                chk(ui, &mut self.vis_field_lines, t("lignes de flux")).on_disabled_hover_text(t("Flux axial : le champ est montré par la carte et les vecteurs Bz dans chaque entrefer"));
            });
            chk(ui, &mut self.vis_field_vec, t("vecteurs B"));
        });
        ui.horizontal_wrapped(|ui| {
            lbl(ui, t("Amplification de la vibration"));
            ui.add(egui::Slider::new(&mut self.vib_amp, 0.0..=5.0).max_decimals(2));
            ui.separator();
            ui.toggle_value(&mut self.live_sound, format!("🔊 {}", t("Son en direct")))
                .on_hover_text(t("Le son suit la vitesse choisie (régénéré à chaque changement)"));
            ui.toggle_value(&mut self.ramp, format!("📈 {}", t("Rampe de vitesse")))
                .on_hover_text(t("Balaye automatiquement la vitesse entre deux bornes : toutes les grandeurs évoluent en direct"));
            if self.ramp {
                ui.add(egui::DragValue::new(&mut self.ramp_lo).clamp_range(10.0..=60000.0).speed(50.0).suffix(" tr/min"));
                ui.label("–");
                ui.add(egui::DragValue::new(&mut self.ramp_hi).clamp_range(10.0..=60000.0).speed(50.0).suffix(" tr/min"));
                ui.add(egui::DragValue::new(&mut self.ramp_period).clamp_range(2.0..=300.0).speed(0.5).suffix(" s"));
            }
        });
        let Some(r) = self.res.as_ref() else { return };
        let spec = r.spec.clone();
        let p = self.spec.dynamics.clone();
        let m = dy::masses(&spec);
        let ic = dy::iron_coeffs(&r.iron, &m, &spec, &p);
        let st = dy::evaluate(r, &ic, &m, &p, self.anim_rpm);
        let k_br = st.br_ratio; // effet de la température sur les aimants
        let vib = dy::vibration(r, &m, &p, self.anim_rpm, k_br);
        let th = self.theta_r;
        let th_r = th.to_radians();
        // valeurs instantanées
        let span = r.cog.metrics.span_deg;
        let w_ref = spec.rpm.max(1e-9);
        let emf_k = self.anim_rpm / w_ref * k_br;
        let inst_emf: Vec<f64> = (0..3).map(|ph| dy::interp_periodic(&r.emf.angles_deg, &r.emf.emf[ph], 360.0, th) * emf_k).collect();
        let inst_cog = dy::interp_periodic(&r.cog.angles_deg, &r.cog.torque_nm, span, th) * k_br * k_br;
        let fx = dy::interp_periodic(&r.forces.angles_deg, &r.forces.fx, 360.0, th) * k_br * k_br;
        let fy = dy::interp_periodic(&r.forces.angles_deg, &r.forces.fy, 360.0, th) * k_br * k_br;
        let q = spec.n_slots;
        let b_teeth: Vec<f64> = (0..q).map(|j| dy::interp_periodic(&r.iron.angles_deg, &r.iron.b_tooth[j], 360.0, th) * k_br).collect();
        let b_tmax = b_teeth.iter().fold(0.0f64, |a, v| a.max(v.abs()));
        let ribbon = if self.show_ribbon {
            let f = r.model.solve(th_r);
            Some(r.model.airgap_b(&f, r.model.r_mid_gap(), 720).0)
        } else {
            None
        };
        let r = self.res.as_ref().unwrap();
        // --- scène 3D ---
        // La forme de la machine ne demande aucun calcul : elle est dessinée
        // tout de suite d'après les réglages en cours, même pendant que le
        // calcul de la nouvelle machine tourne (sinon l'ancienne machine
        // restait affichée plusieurs secondes après un changement de type).
        let stale = serde_json::to_string(&self.spec).ok() != serde_json::to_string(&r.spec).ok();
        let busy3d = self.job.is_some();
        let gspec = if stale { self.spec.clone() } else { spec.clone() };
        let gq = gspec.n_slots;
        let teeth_by_b = self.teeth_by_b && !stale;
        let rg = gspec.rings();
        let r_out = rg.r_max;
        let s = 1.0 / r_out;
        let hz = (gspec.length() * s).min(1.2) as f32 / 2.0;
        let mut quads = Vec::new();
        let iron = Color32::from_rgb(140, 150, 170);
        let coils = winding::layout(&gspec);
        let phase_col = [Color32::from_rgb(220, 160, 60), Color32::from_rgb(90, 200, 120), Color32::from_rgb(170, 120, 255)];
        let bscale = ic.b_tooth_peak.max(1e-6);
        let tooth_col = |tooth: usize| -> Color32 {
            if teeth_by_b {
                view3d::colormap((0.5 + 0.5 * b_teeth[tooth] / bscale) as f32)
            } else {
                let pc = phase_col[coils[tooth].phase];
                Color32::from_rgb(((iron.r() as u16 + pc.r() as u16) / 2) as u8, ((iron.g() as u16 + pc.g() as u16) / 2) as u8, ((iron.b() as u16 + pc.b() as u16) / 2) as u8)
            }
        };
        let axial = gspec.topology == Topology::Axial;
        if axial {
            // flux axial : disques empilés le long de l'axe (stator au centre)
            quads = Self::axial_quads(&gspec, th_r, &tooth_col);
        } else {
            for i in 0..gq {
                let a0 = (i as f64 + 0.5) * 2.0 * PI / gq as f64 + rg.beta_body / 2.0;
                let a1 = (i as f64 + 1.5) * 2.0 * PI / gq as f64 - rg.beta_body / 2.0;
                let o0 = (i as f64 + 0.5) * 2.0 * PI / gq as f64 + rg.beta_open / 2.0;
                let o1 = (i as f64 + 1.5) * 2.0 * PI / gq as f64 - rg.beta_open / 2.0;
                let tooth = (i + 1) % gq;
                let col = if teeth_by_b {
                    view3d::colormap((0.5 + 0.5 * b_teeth[tooth] / bscale) as f32)
                } else {
                    let pc = phase_col[coils[tooth].phase];
                    Color32::from_rgb(((iron.r() as u16 + pc.r() as u16) / 2) as u8, ((iron.g() as u16 + pc.g() as u16) / 2) as u8, ((iron.b() as u16 + pc.b() as u16) / 2) as u8)
                };
                ring_sector(&mut quads, rg.sb * s, rg.tip * s, a0, a1, -hz, hz, col, 8, 1000 + tooth as u32);
                if (rg.tip - rg.bore).abs() > 1e-9 {
                    ring_sector(&mut quads, rg.tip * s, rg.bore * s, o0, o1, -hz, hz, col, 8, 1000 + tooth as u32);
                }
            }
            ring_sector(&mut quads, rg.yoke.0 * s, rg.yoke.1 * s, 0.0, 2.0 * PI, -hz, hz, iron, 72, 2000);
            for (k, pc) in magnets::pieces(&gspec).iter().enumerate() {
                let pos = match pc.kind {
                    PieceKind::Radial { s } => s > 0.0,
                    PieceKind::Fixed { psi } => (0.5 * (pc.t1 + pc.t2) - psi).cos() > 0.0,
                };
                let col = if pos { Color32::from_rgb(230, 70, 60) } else { Color32::from_rgb(50, 120, 235) };
                ring_sector(&mut quads, gspec.r_magnet_in() * s, gspec.r_magnet_out() * s, pc.t1 + th_r, pc.t2 + th_r, -hz, hz, col, 6, k as u32);
            }
            ring_sector(&mut quads, rg.rotor_yoke.0 * s, rg.rotor_yoke.1 * s, PI * 0.5 + th_r, 2.0 * PI + th_r, -hz, hz, Color32::from_rgb(110, 118, 135), 54, 3000);
        }
        if let Some(br) = ribbon.as_ref().filter(|_| !axial && !stale) {
            let n = br.len();
            let rr = 0.5 * (gspec.r_bore() + gspec.r_magnet_in()) * s;
            for k in 0..n {
                let a0 = 2.0 * PI * k as f64 / n as f64;
                let a1 = 2.0 * PI * (k + 1) as f64 / n as f64;
                let col = view3d::colormap((0.5 + br[k] / 4.0) as f32);
                let pp = |a: f64, z: f32| [(rr * a.cos()) as f32, (rr * a.sin()) as f32, z];
                quads.push(Quad { p: [pp(a0, hz * 1.15), pp(a1, hz * 1.15), pp(a1, hz * 1.35), pp(a0, hz * 1.35)], col, tag: 4000 });
            }
        }
        // champ magnétique dans la scène (calculé à la position courante du rotor)
        let mut field = champ3d::Field3d::default();
        let want_field = !stale && (self.vis_field_map || self.vis_field_vec || (self.vis_field_lines && !axial));
        if self.vis_transparent {
            champ3d::make_transparent(&mut quads, self.vis_opacity, want_field);
        }
        if want_field {
            let f = r.model.solve(th_r);
            field = if axial {
                champ3d::axial(&r.model, &f, &spec, self.vis_field_map, self.vis_field_vec)
            } else {
                // plan médian en vue transparente, sinon sur la face avant
                let zf = if self.vis_transparent { 0.0 } else { hz * 1.02 };
                champ3d::radial(&r.model, &f, r_out, zf, self.vis_field_map, self.vis_field_lines, self.vis_field_vec)
            };
            quads.extend(std::mem::take(&mut field.quads));
        }
        // vibration : le rotor (aimants, cloche) est décalé de son déplacement, amplifié
        let (vx, vy) = vib.disp_at(th);
        // amplification automatique : l'orbite crête vaut 5 % du rayon × curseur
        let amp = if vib.disp_peak_um > 0.0 { self.vib_amp * 0.05 * r_out / (vib.disp_peak_um * 1e-6) } else { 0.0 };
        let (ox, oy) = ((vx * amp * s) as f32, (vy * amp * s) as f32);
        for qd in quads.iter_mut().filter(|qd| qd.tag < 1000 || qd.tag == 3000) {
            for v in qd.p.iter_mut() {
                v[0] += ox;
                v[1] += oy;
            }
        }
        *view3d::HINT.write().unwrap() = t("glisser : orbite · maj/clic droit : déplacer · molette : zoom · double-clic : vue initiale").into();
        let angles = spec.magnet_angles_deg();
        let pitch = 360.0 / angles.len() as f64;
        let fmax = r.forces.max_n.max(1e-9);
        let show_force = self.show_force;
        let fsym = u.sym(Q::Force);
        let fval = u.show(Q::Force, fx.hypot(fy));
        let h = ui.available_height();
        let mut hovered = None;
        ui.horizontal_top(|ui| {
            let w3 = (ui.available_width() * 0.62).max(300.0);
            ui.allocate_ui(Vec2::new(w3, h * 0.66), |ui| {
                let r3 = ui.max_rect();
                hovered = view3d::quads_ui(ui, &mut self.cam_motor, &quads, |painter, cam, rect| {
                    if stale {
                        let msg = if busy3d {
                            format!("⏳ {}", T(lang, "Nouvelle machine : calcul en cours… la forme est déjà à jour, les grandeurs le seront à la fin du calcul"))
                        } else {
                            format!("⚠ {}", T(lang, "Réglages non calculés : la forme est à jour, les grandeurs sont celles du dernier calcul réussi (voir la barre d'état)"))
                        };
                        let g = painter.layout(msg, egui::FontId::proportional(14.0), Color32::WHITE, rect.width() - 40.0);
                        let bx = egui::Rect::from_min_size(egui::pos2(rect.left() + 12.0, rect.bottom() - g.size().y - 40.0), g.size() + Vec2::new(16.0, 10.0));
                        painter.rect_filled(bx, 6.0, ORANGE.gamma_multiply(0.85));
                        painter.galley(bx.min + Vec2::new(8.0, 5.0), g, Color32::WHITE);
                    }
                    for (a, b, col, w) in &field.segs {
                        let (pa, _) = cam.project(*a, rect);
                        let (pb, _) = cam.project(*b, rect);
                        painter.line_segment([pa, pb], egui::Stroke::new(*w, *col));
                    }
                    if want_field {
                        // légende des couleurs du champ
                        let (x0, y0) = (rect.right() - 26.0, rect.top() + 34.0);
                        let hh = (rect.height() * 0.35).max(80.0);
                        for k in 0..60 {
                            let tt = k as f32 / 60.0;
                            let y = y0 + hh * (1.0 - tt);
                            painter.rect_filled(egui::Rect::from_min_max(egui::pos2(x0, y - hh / 60.0 - 0.5), egui::pos2(x0 + 12.0, y)), 0.0, view3d::colormap(tt));
                        }
                        let (lo, hi, name) = if field.signed { (-field.bmax, field.bmax, "Bz") } else { (0.0, field.bmax, "|B|") };
                        painter.text(egui::pos2(x0 + 6.0, y0 - 4.0), egui::Align2::CENTER_BOTTOM, format!("{name} {hi:+.2} T"), egui::FontId::proportional(10.0), Color32::GRAY);
                        painter.text(egui::pos2(x0 + 6.0, y0 + hh + 4.0), egui::Align2::CENTER_TOP, format!("{lo:+.2} T"), egui::FontId::proportional(10.0), Color32::GRAY);
                    }
                    if show_force {
                        let z = hz * 1.6;
                        let k = 0.9 / fmax as f32;
                        let (p0, _) = cam.project([0.0, 0.0, z], rect);
                        let (p1, _) = cam.project([fx as f32 * k, fy as f32 * k, z], rect);
                        painter.arrow(p0, p1 - p0, egui::Stroke::new(3.0f32, ORANGE));
                        painter.text(p1, egui::Align2::LEFT_BOTTOM, format!(" F = {} {}", fnum(fval), fsym), egui::FontId::proportional(13.0), ORANGE);
                    }
                    painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), egui::Align2::LEFT_BOTTOM, view3d::HINT.read().unwrap().clone(), egui::FontId::proportional(11.0), Color32::GRAY);
                });
                // icônes posées sur la vue : transparence et champ, en un clic
                let tb = egui::Rect::from_min_size(r3.min + Vec2::new(8.0, 8.0), Vec2::new(r3.width() - 16.0, 30.0));
                ui.allocate_ui_at_rect(tb, |ui| {
                    ui.horizontal(|ui| {
                        let axial = self.spec.topology == Topology::Axial;
                        let icon = |ui: &mut egui::Ui, on: &mut bool, ic: &str, tip: &str, enabled: bool| {
                            let txt = RichText::new(ic).size(18.0).color(if *on { Color32::WHITE } else { Color32::LIGHT_GRAY });
                            let b = egui::Button::new(txt).fill(if *on { champ3d::FIELD3D_COL } else { Color32::from_black_alpha(140) }).min_size(Vec2::splat(30.0));
                            if ui.add_enabled(enabled, b).on_hover_text(tip).clicked() {
                                *on = !*on;
                            }
                        };
                        icon(ui, &mut self.vis_transparent, "👁", t("Vue transparente"), true);
                        icon(ui, &mut self.vis_field_map, "🌈", t("carte de l'induction"), true);
                        icon(ui, &mut self.vis_field_lines, "🌀", t("lignes de flux"), !axial);
                        icon(ui, &mut self.vis_field_vec, "➡", t("vecteurs B"), true);
                    });
                });
            });
            // --- tableau de bord en direct ---
            ui.vertical(|ui| {
                ui.set_max_height(h * 0.66);
                egui::ScrollArea::vertical().id_source("hud3d").show(ui, |ui| {
                    let head = |ui: &mut egui::Ui, s: &str| {
                        ui.add_space(4.0);
                        ui.label(RichText::new(s).strong().color(if view3d::is_dark() { CYAN } else { Color32::from_rgb(0, 110, 170) }));
                    };
                    let row = |ui: &mut egui::Ui, k: &str, v: String| {
                        ui.label(RichText::new(k).weak());
                        ui.label(RichText::new(v).monospace());
                        ui.end_row();
                    };
                    // courbes en fonction de la vitesse (repère = vitesse courante)
                    let key = format!("{}|{}", r.seconds, serde_json::to_string(&p).unwrap_or_default());
                    if self.speed_curves.as_ref().map(|(k, _)| k != &key).unwrap_or(true) {
                        let pts: Vec<[f64; 5]> = (0..36)
                            .map(|k| {
                                let rpm = 100.0 * (600.0f64).powf(k as f64 / 35.0);
                                let sx = dy::evaluate(r, &ic, &m, &p, rpm);
                                let vx = dy::vibration(r, &m, &p, rpm, sx.br_ratio);
                                [rpm, sx.p_total, sx.t_steady, vx.vel_rms_mm_s, sx.la_index_db.max(-20.0)]
                            })
                            .collect();
                        self.speed_curves = Some((key, pts));
                    }
                    if let Some((_, pts)) = &self.speed_curves {
                        head(ui, t("En fonction de la vitesse"));
                        let cur = self.anim_rpm;
                        let mini = |ui: &mut egui::Ui, id: &str, col: usize, play_lbl: &str, c: Color32| {
                            ui.label(RichText::new(play_lbl).small().weak());
                            Plot::new(id).height(70.0).show_axes([false, true]).allow_drag(false).allow_zoom(false).allow_scroll(false).show_background(false)
                                .label_formatter(|_, v| format!("{:.0} tr/min\n{:.4}", 10f64.powf(v.x), v.y))
                                .show(ui, |pl| {
                                    pl.line(Line::new(PlotPoints::from(pts.iter().map(|q| [q[0].log10(), q[col]]).collect::<Vec<_>>())).color(c).width(1.5f32));
                                    pl.vline(VLine::new(cur.max(1.0).log10()).color(ORANGE).width(1.5f32));
                                });
                        };
                        mini(ui, "vs_p", 1, t("pertes à vide [W] (échelle log. de la vitesse)"), CYAN);
                        mini(ui, "vs_t", 2, t("température [°C]"), Color32::from_rgb(255, 110, 70));
                        mini(ui, "vs_v", 3, t("vitesse vibratoire [mm/s]"), SERIES[4]);
                        mini(ui, "vs_s", 4, t("indice sonore [dB(A)]"), SERIES[2]);
                    }
                    head(ui, t("Instantané"));
                    egui::Grid::new("hud_inst").num_columns(2).spacing([10.0, 2.0]).show(ui, |ui| {
                        row(ui, "θr", format!("{th:.2}°"));
                        row(ui, t("temps réel simulé"), format!("{} ms", fnum(self.sim_time * 1e3)));
                        row(ui, t("fréquence électrique"), format!("{:.1} Hz", st.f_elec_hz));
                        row(ui, t("couple de détente"), format!("{} {}", fnum(u.show(Q::Torque, inst_cog)), u.sym(Q::Torque)));
                        for (k, e) in inst_emf.iter().enumerate() {
                            row(ui, &format!("{} {}", t("FEM phase"), ["A", "B", "C"][k]), format!("{} {}", fnum(u.show(Q::Voltage, *e)), u.sym(Q::Voltage)));
                        }
                        row(ui, t("force radiale"), format!("{} {}", fnum(fval), fsym));
                        row(ui, t("induction max dans les dents"), format!("{} {}", fnum(u.show(Q::Induction, b_tmax)), u.sym(Q::Induction)));
                    });
                    head(ui, t("Pertes et frottements (régime établi)"));
                    egui::Grid::new("hud_loss").num_columns(2).spacing([10.0, 2.0]).show(ui, |ui| {
                        row(ui, t("fer : hystérésis"), format!("{:.3} W", st.p_hyst));
                        row(ui, t("fer : courants de Foucault"), format!("{:.3} W", st.p_eddy));
                        row(ui, t("roulements"), format!("{:.3} W", st.p_bearing));
                        row(ui, t("ventilation (air)"), format!("{:.3} W", st.p_windage));
                        row(ui, t("total à vide"), format!("{:.3} W", st.p_total));
                        row(ui, t("couple résistant"), format!("{} {}", fnum(u.show(Q::Torque, st.drag_torque)), u.sym(Q::Torque)));
                        row(ui, t("charge des roulements"), format!("{} {}", fnum(u.show(Q::Force, st.bearing_load)), fsym));
                        row(ui, t("force due au balourd"), format!("{} {}", fnum(u.show(Q::Force, st.unbalance_force)), fsym));
                    });
                    head(ui, t("Chaleur"));
                    egui::Grid::new("hud_th").num_columns(2).spacing([10.0, 2.0]).show(ui, |ui| {
                        row(ui, t("température en régime établi"), format!("{:.1} °C", st.t_steady));
                        row(ui, t("constante de temps thermique"), format!("{:.0} s", st.tau_s));
                        row(ui, t("convection"), format!("{:.1} W/(m²·K)", st.h_conv));
                        row(ui, t("Br à chaud / Br à 20 °C"), format!("{:.2} %", 100.0 * k_br));
                    });
                    head(ui, t("Vibration"));
                    egui::Grid::new("hud_vib").num_columns(2).spacing([10.0, 2.0]).show(ui, |ui| {
                        row(ui, t("déplacement crête du rotor"), format!("{} µm", fnum(vib.disp_peak_um)));
                        row(ui, t("amplification affichée"), format!("×{}", fnum(amp)));
                        ui.label(RichText::new(t("vitesse vibratoire efficace")).weak());
                        let zc = match vib.iso_zone { 'A' => Color32::from_rgb(70, 200, 110), 'B' => Color32::from_rgb(170, 210, 80), 'C' => ORANGE, _ => Color32::from_rgb(245, 85, 75) };
                        ui.label(RichText::new(format!("{:.3} mm/s · {} {}", vib.vel_rms_mm_s, t("zone ISO"), vib.iso_zone)).monospace().color(zc))
                            .on_hover_text(t("ISO 10816-1 classe I : A ≤ 0,71 · B ≤ 1,8 · C ≤ 4,5 mm/s · D au-delà (repère indicatif pour un petit moteur)"));
                        ui.end_row();
                        row(ui, t("accélération crête"), format!("{:.2} g", vib.acc_peak_g));
                        let near = (self.anim_rpm / vib.critical_rpm - 1.0).abs() < 0.2;
                        ui.label(RichText::new(t("vitesse critique (1×)")).weak());
                        ui.label(RichText::new(format!("{:.0} tr/min", vib.critical_rpm)).monospace().color(if near { Color32::from_rgb(245, 85, 75) } else { Color32::GRAY }));
                        ui.end_row();
                        row(ui, t("ondulation de vitesse (torsion)"), format!("{:.3} tr/min", vib.speed_ripple_rpm));
                        row(ui, t("accélération angulaire crête"), format!("{} rad/s²", fnum(vib.ang_acc_peak)));
                    });
                    if !vib.resonances.is_empty() {
                        ui.label(RichText::new(t("Résonances excitées par les harmoniques de force :")).small().weak());
                        for &(k, a, n) in &vib.resonances {
                            let near = (self.anim_rpm / n - 1.0).abs() < 0.15;
                            ui.label(RichText::new(format!("  {} {:+} : {} {} : {:.0} tr/min", t("ordre"), k, fnum(u.show(Q::Force, a)), u.sym(Q::Force), n)).small().monospace().color(if near { Color32::from_rgb(245, 85, 75) } else { Color32::GRAY }));
                        }
                    }
                    head(ui, t("Son (excitations magnétiques)"));
                    let audible = st.tones.iter().any(|t| t.freq_hz <= 20000.0);
                    ui.label(if audible { format!("{} : {:.1} dB(A) {}", t("indice global"), st.la_index_db, t("rel. 1 mN·m")) } else { t("aucune raie audible (toutes au-delà de 20 kHz)").to_string() });
                    egui::Grid::new("hud_son").num_columns(3).striped(true).spacing([10.0, 2.0]).show(ui, |ui| {
                        for tn in st.tones.iter().take(6) {
                            let aud = tn.freq_hz <= 20000.0;
                            ui.label(RichText::new(format!("{:.0} Hz", tn.freq_hz)).monospace().color(if aud { ORANGE } else { Color32::GRAY }));
                            ui.label(RichText::new(t(if tn.source == "couple" { "couple" } else { "force" })).weak());
                            ui.label(RichText::new(if aud { format!("{:.1} dB(A)", tn.level_a_db) } else { t("inaudible").into() }).monospace());
                            ui.end_row();
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button(format!("🔊 {}", t("Écouter (2 s)"))).on_hover_text(t("Excitation magnétique synthétisée à la vitesse choisie (couple de détente + force radiale)")).clicked() {
                            let wav = dy::synth_wav(r, self.anim_rpm, 2.0);
                            match crate::platform::play_wav(&wav) {
                                Some(s) => {
                                    if let Some(mut old) = self.sound_child.replace(s) {
                                        old.stop();
                                    }
                                }
                                None => self.err = Some(t("aucun lecteur audio trouvé").into()),
                            }
                        }
                        if ui.button(t("Exporter le son (WAV)…")).clicked() {
                            let _ = crate::platform::save(&format!("{}_{:.0}rpm.wav", spec.name, self.anim_rpm), "WAV", "wav", &dy::synth_wav(r, self.anim_rpm, 5.0));
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(RichText::new(t("Indicateurs d'ingénierie : pertes de Bertotti sur l'induction réelle, roulements de Palmgren, ventilation, convection forcée, Br(T). Le son est l'excitation magnétique, pas un niveau mesuré au micro. À vide et sans saturation, la forme des signaux ne dépend pas de la vitesse : seuls leur fréquence, leur amplitude (température) et les pertes changent.")).small().weak());
                });
            });
        });
        // --- oscilloscope : 3 périodes électriques en temps réel ---
        let pp = (spec.n_poles / 2).max(1) as f64;
        let te = 60.0 / (self.anim_rpm.max(1e-6) * pp);
        let win = 3.0 * te;
        let n = 400;
        let deg_s = self.anim_rpm * 6.0;
        let mut emf_series: Vec<(String, Vec<[f64; 2]>, Color32)> = (0..3).map(|ph| (format!("{} {}", t("FEM phase"), ["A", "B", "C"][ph]), Vec::with_capacity(n), SERIES[ph])).collect();
        let mut cog_pts = Vec::with_capacity(n);
        let mut f_pts = Vec::with_capacity(n);
        let fv = u.factor(Q::Voltage);
        let ft = u.factor(Q::Torque);
        let ff = u.factor(Q::Force);
        for k in 0..n {
            let tt = -win + win * k as f64 / (n - 1) as f64;
            let a = th + deg_s * tt;
            for ph in 0..3 {
                emf_series[ph].1.push([tt * 1e3, dy::interp_periodic(&r.emf.angles_deg, &r.emf.emf[ph], 360.0, a) * emf_k * fv]);
            }
            cog_pts.push([tt * 1e3, dy::interp_periodic(&r.cog.angles_deg, &r.cog.torque_nm, span, a) * k_br * k_br * ft]);
            let fxx = dy::interp_periodic(&r.forces.angles_deg, &r.forces.fx, 360.0, a);
            let fyy = dy::interp_periodic(&r.forces.angles_deg, &r.forces.fy, 360.0, a);
            f_pts.push([tt * 1e3, fxx.hypot(fyy) * k_br * k_br * ff]);
        }
        let orbit: Vec<[f64; 2]> = (0..=360).map(|j| { let (x, y) = vib.disp_at(j as f64); [x * 1e6, y * 1e6] }).collect();
        let cur = [vx * 1e6, vy * 1e6];
        ui.columns(3, |c| {
            let hh = c[0].available_height().max(120.0);
            Plot::new("orbit_vib").height(hh).data_aspect(1.0).x_axis_label("x [µm]").y_axis_label("y [µm]").allow_drag(false).show(&mut c[2], |p| {
                p.line(Line::new(PlotPoints::from(orbit)).color(SERIES[4]).width(1.5f32).name(t("orbite du rotor")));
                p.points(Points::new(vec![cur]).radius(5.0f32).color(ORANGE));
            });
            Plot::new("osc_emf").height(hh).legend(Legend::default().position(Corner::LeftTop)).x_axis_label(t("temps réel [ms]")).y_axis_label(format!("{} [{}]", t("FEM"), u.sym(Q::Voltage))).allow_drag(false).show(&mut c[0], |p| {
                for (nme, pts, col) in &emf_series {
                    p.line(Line::new(PlotPoints::from(pts.clone())).name(nme).color(*col).width(1.5f32));
                }
                p.vline(VLine::new(0.0).color(Color32::WHITE).width(1.0f32));
            });
            Plot::new("osc_cog").height(hh).legend(Legend::default().position(Corner::LeftTop)).x_axis_label(t("temps réel [ms]")).allow_drag(false).show(&mut c[1], |p| {
                p.line(Line::new(PlotPoints::from(cog_pts)).name(format!("{} [{}]", t("couple de détente"), u.sym(Q::Torque))).color(ORANGE).width(1.5f32));
                p.line(Line::new(PlotPoints::from(f_pts)).name(format!("|F| [{}]", fsym)).color(SERIES[3]).width(1.5f32));
                p.vline(VLine::new(0.0).color(Color32::WHITE).width(1.0f32));
            });
        });
        if let Some(tag) = hovered {
            let txt = match tag {
                k if (k as usize) < angles.len() => format!("{} {} ({}) — {:.4}° ({:+.4}° {})", t("Aimant"), k + 1, if k % 2 == 0 { "N" } else { "S" }, angles[k as usize], angles[k as usize] - k as f64 * pitch, t("vs régulier")),
                k if (1000..2000).contains(&k) => {
                    let j = (k - 1000) as usize;
                    let c = &coils[j];
                    format!("{} {} — {} {}{} — B = {} {}", t("Dent"), j, t("bobine phase"), winding::PHASES[c.phase], if c.sign < 0.0 { " (−)" } else { "" }, fnum(u.show(Q::Induction, b_teeth[j])), u.sym(Q::Induction))
                }
                2000 => t("Culasse stator").into(),
                3000 => t("Culasse rotor (cloche)").into(),
                _ => t("Induction d'entrefer Br").into(),
            };
            egui::show_tooltip_text(ui.ctx(), egui::Id::new("tip3d"), txt);
        }
    }

    // ----------------------------- Cartes 3D --------------------------
    pub(super) fn tab_carte3d(&mut self, ui: &mut egui::Ui) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        ui.horizontal(|ui| {
            if crate::repartition::uses_offsets() {
                ui.selectable_value(&mut self.surf_kind, "cogging_amplitude".to_string(), t("Cogging × amplitude des décalages"));
            }
            ui.selectable_value(&mut self.surf_kind, "br".to_string(), t("Br(θ, θr) d'entrefer"));
            ui.separator();
            if self.surf_kind == "cogging_amplitude" {
                lbl(ui, t("amplitude max"));
                ui.add(egui::DragValue::new(&mut self.amp_max).clamp_range(0.5..=15.0).suffix("°").speed(0.1));
            }
            if ui.add_enabled(self.surf_job.is_none(), egui::Button::new(RichText::new(format!("🗻 {}", t("Calculer la carte"))).strong())).clicked() {
                let k = self.surf_kind.clone();
                self.launch_surface(&k);
            }
            chk(ui, &mut self.wire, t("maillage"));
        });
        *view3d::HINT.write().unwrap() = t("glisser : orbite · maj/clic droit : déplacer · molette : zoom · double-clic : vue initiale").into();
        match self.surfaces.get(&self.surf_kind) {
            Some(s) => {
                let s_ptr = s as *const Surface;
                // SAFETY : `s` reste valide pendant l'appel (pas de mutation de self.surfaces)
                let s = unsafe { &*s_ptr };
                let hit = view3d::surface_ui(ui, &mut self.cam_surf, s, self.wire);
                if let Some((x, y, z)) = hit {
                    egui::show_tooltip_text(ui.ctx(), egui::Id::new("tip_surf"), format!("{} = {:.4}\n{} = {:.4}\n{} = {:.6}", s.x_label, x, s.y_label, y, s.z_label, z));
                }
            }
            None => {
                lbl(ui, t("Cliquez sur « Calculer la carte » pour générer la surface."));
            }
        }
    }

    // ----------------------------- Validation -------------------------
    pub(super) fn tab_validation(&mut self, ui: &mut egui::Ui) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        self.recalage_ui(ui);
        ui.separator();
        ui.label(RichText::new(format!("✔ {}", t("Validation par un solveur indépendant"))).strong());
        lbl(ui, t("Contrôle croisé du modèle analytique par un solveur volumes finis polaire indépendant (fer μr = 10⁵ maillé, aimantation moyennée par cellule, couple d'Arkkio)."));
        if self.spec.topology == Topology::Axial {
            ui.label(RichText::new(t("Flux axial : la validation porte sur la tranche de référence du modèle multi-tranches, résolue en géométrie plane (circonférence développée) ; couple d'une tranche.")).small().weak());
        }
        if ui.add_enabled(self.val_job.is_none(), egui::Button::new(RichText::new(format!("✔ {}", t("Lancer la validation (≈ 30 s)"))).strong())).clicked() {
            self.val_job = Some(compute::run_validation(self.spec.clone(), self.st, 5760, 8));
        }
        let Some((rows, open)) = &self.validation else { return };
        let u = self.cfg.units.clone();
        let ft = u.factor(Q::Torque);
        let maxabs = rows.iter().map(|r| r.1.abs()).fold(0.0, f64::max).max(1e-12);
        let err = rows.iter().map(|r| (r.1 - r.2).abs()).fold(0.0, f64::max) / maxabs * 100.0;
        ui.label(RichText::new(format!("{} {:.2} % ({} {:.4} mm)", t("Écart max rapporté à l'amplitude :"), err, t("ouverture alignée sur la grille :"), open)).strong().color(badge(err < 5.0)));
        let an: Vec<[f64; 2]> = rows.iter().map(|r| [r.0, r.1 * 1e-3 * ft]).collect();
        let fd: Vec<[f64; 2]> = rows.iter().map(|r| [r.0, r.2 * 1e-3 * ft]).collect();
        let mut cur = *self.cursors.entry("val").or_default();
        let h = ui.available_height() * 0.6;
        curve_plot(ui, "val", &[(t("analytique (sous-domaines)").into(), an, CYAN), (t("numérique (volumes finis)").into(), fd, ORANGE)], "θr [°]", &format!("T [{}]", u.sym(Q::Torque)), h, &mut cur);
        self.cursors.insert("val", cur);
        egui::Grid::new("val_grid").striped(true).num_columns(4).show(ui, |ui| {
            for h in ["θr [°]", t("analytique"), t("numérique"), t("écart")] {
                ui.label(RichText::new(h).strong());
            }
            ui.end_row();
            for r in rows {
                ui.label(format!("{:.4}", r.0));
                ui.label(format!("{:+.5}", u.show(Q::Torque, r.1 * 1e-3)));
                ui.label(format!("{:+.5}", u.show(Q::Torque, r.2 * 1e-3)));
                ui.label(format!("{:+.2} %", 100.0 * (r.2 - r.1) / maxabs));
                ui.end_row();
            }
        });
    }
}
