//! Onglets de mesure : synthèse, géométrie, champ, cogging, FEM, forces, charge.

use super::*;

impl App {
    // ---------------------------- Synthèse ----------------------------
    pub(super) fn tab_synthese(&mut self, ui: &mut egui::Ui) {
        let u = self.u().clone();
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let br_spark: Vec<f64> = self.field().map(|(_, g)| g.br.iter().step_by(4).cloned().collect()).unwrap_or_default();
        let quality = self.quality_info();
        let r = self.res.as_ref().unwrap();
        let rep = &r.report;
        let crit = crate::repartition::criteria();
        let refname = crate::repartition::reference_name();
        let b0 = self.cmp.iter().find(|v| v.name == refname).map(|v| v.report.cogging.cogging_pp_mnm);
        let seuil = |s: String| format!("{} {s} ({})", t("seuil"), t(crit.source));
        let mut open_full: Option<Tab> = None;
        // aperçus des vues (vignettes)
        let spec_off: Vec<f64> = {
            let a = r.spec.magnet_angles_deg();
            let p = 360.0 / a.len() as f64;
            a.iter().enumerate().map(|(k, x)| x - k as f64 * p).collect()
        };
        let fmag: Vec<f64> = r.forces.fx.iter().zip(&r.forces.fy).map(|(x, y)| x.hypot(*y)).collect();
        let surf_row: Vec<f64> = self
            .surfaces
            .get("cogging_amplitude")
            .map(|s| {
                let nx = s.xs.len();
                s.z[(s.ys.len() - 1) * nx..].to_vec()
            })
            .unwrap_or_default();
        let cmp_pp: Vec<f64> = self.cmp.iter().map(|v| v.report.cogging.cogging_pp_mnm).collect();
        let val: Vec<f64> = self.validation.as_ref().map(|(v, _)| v.iter().map(|x| x.2).collect()).unwrap_or_default();
        let previews: Vec<(Tab, Vec<f64>, bool)> = vec![
            (Tab::Geometrie, spec_off, true),
            (Tab::Champ, br_spark, false),
            (Tab::Cogging, r.cog.torque_nm.clone(), false),
            (Tab::Fem, r.emf.emf[0].clone(), false),
            (Tab::Forces, fmag, false),
            (Tab::Comparaison, cmp_pp, true),
            (Tab::Moteur3d, r.cog.torque_nm.iter().step_by(8).cloned().collect(), false),
            (Tab::Carte3d, surf_row, false),
            (Tab::Validation, val, false),
        ];
        let pulse_v = pulse(ui);
        let mut go3d = false;
        let fresh = self.fresh.clone();
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(l) = self.logo_tex.clone().filter(|_| ui.available_width() > 700.0) {
                    if ui.add(egui::Image::new((l.id(), Vec2::new(240.0, 134.0))).rounding(8.0).sense(egui::Sense::click())).on_hover_text(tr("Développeur de l'application")).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.show_author = true;
                    }
                }
                if let Some((ok, total, tip)) = &quality {
                    super::criteres::draw_quality(ui, *ok, *total, tip);
                }
                ui.vertical(|ui| {
                    ui.heading(RichText::new(format!("{} — {}", rep.variant, t("tableau de bord"))).strong());
                    ui.label(format!(
                        "{} {}N{}P · {} {} · {} {:.3} s",
                        t("Moteur"),
                        r.spec.n_slots,
                        r.spec.n_poles,
                        t("bobinage"),
                        rep.winding,
                        t("calcul"),
                        r.seconds
                    ));
                    ui.label(RichText::new(t("Survolez une carte pour le détail ; cliquez sur un graphe pour poser les curseurs A (gauche) et B (droit).")).weak());
                });
            });
            ui.add_space(6.0);
            ui.label(RichText::new(t("Vues — cliquez pour ouvrir en plein écran (une vignette clignote quand un nouveau résultat est prêt)")).weak());
            ui.horizontal_wrapped(|ui| {
                for (tab, data, bars) in &previews {
                    if *tab == Tab::Comparaison && crate::repartition::variants().len() < 2 {
                        continue;
                    }
                    let (_, ic, k) = ALL_TABS.iter().find(|x| x.0 == *tab).copied().unwrap();
                    let is_new = fresh.contains(tab);
                    if vignette(ui, &format!("{ic} {}", t(k)), data, *bars, is_new, pulse_v).clicked() {
                        open_full = Some(*tab);
                    }
                }
            });
            ui.add_space(6.0);
            // fonction phare : bandeau d'accès direct au champ 3D
            egui::Frame::none().fill(champ3d::FIELD3D_COL.gamma_multiply(0.22)).rounding(8.0).inner_margin(egui::Margin::symmetric(10.0, 6.0)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("👁").size(22.0));
                    ui.label(RichText::new(t("Voir le champ magnétique à travers le moteur")).strong());
                    ui.label(RichText::new(t("vue 3D transparente, carte de l'induction, lignes de flux et vecteurs B, en mouvement")).weak());
                    if ui.add(egui::Button::new(RichText::new(format!("{} ➡", t("Ouvrir"))).strong().color(Color32::WHITE)).fill(champ3d::FIELD3D_COL)).clicked() {
                        go3d = true;
                    }
                });
            });
            ui.add_space(8.0);
            let tq = |v_mnm: f64| format!("{} {}", fnum(u.show(Q::Torque, v_mnm * 1e-3)), u.sym(Q::Torque));
            let card_tab = [Tab::Geometrie, Tab::Cogging, Tab::Cogging, Tab::Cogging, Tab::Cogging, Tab::Cogging, Tab::Cogging, Tab::Forces, Tab::Geometrie, Tab::Geometrie, Tab::Fem, Tab::Fem, Tab::Fem, Tab::Charge];
            let sat = &r.sat.at_load;
            let (kst, kro) = (r.sat.knee, r.spec.saturation.rotor.knee());
            let gap = rep.min_magnet_gap_deg;
            let cards: Vec<(String, String, Option<bool>, String)> = vec![
                (
                    t("Faisabilité des aimants").into(),
                    format!("{gap:+.3}°"),
                    Some(gap >= 0.0),
                    if gap >= 0.0 { t("écart min entre aimants voisins : aucun chevauchement").into() } else { t("⚠ des aimants se chevauchent : résultats non physiques dans ces zones, réduire l'arc ou l'amplitude").into() },
                ),
                (
                    t("Cogging crête-à-crête").into(),
                    tq(rep.cogging.cogging_pp_mnm),
                    crit.cogging_target.and(b0).map(|b| rep.cogging.cogging_pp_mnm <= (1.0 + crit.cogging_target.map_or(0.0, |c| c.1) / 100.0) * b),
                    match (crit.cogging_target, b0) {
                        (Some((n, pct)), Some(b)) => format!("{} {n} : {:+.1} % ({} {pct:+.0} %)", t("vs"), 100.0 * (rep.cogging.cogging_pp_mnm / b - 1.0), t("objectif")),
                        (Some((n, _)), None) => format!("{} {n}", t("Lancez « Comparer » pour situer par rapport à")),
                        _ => t("crête-à-crête sur la fenêtre de calcul").into(),
                    },
                ),
                (t("Cogging efficace").into(), tq(rep.cogging.cogging_rms_mnm), None, t("valeur RMS sur la fenêtre").into()),
                (
                    t("Harmonique dominante").into(),
                    format!("{} @ {:.0}/{}", tq(rep.cogging.dominant_mnm), rep.cogging.dominant_order_per_rev, t("tour")),
                    None,
                    t("ordre mécanique par tour").into(),
                ),
                (
                    t("Part de la dominante").into(),
                    format!("{:.1} %", rep.cogging.dominant_share_pct),
                    None,
                    t("énergie spectrale portée par une seule raie (tonalité)").into(),
                ),
                // au-delà de 80 dB, le plancher du spectre n'est que du bruit
                // numérique : la valeur n'a pas de sens physique
                if rep.cogging.tonal_peak_db <= 80.0 {
                    (t("Pic tonal").into(), format!("{:.1} dB", rep.cogging.tonal_peak_db), None, t("dominante / plancher médian du spectre").into())
                } else {
                    (t("Pic tonal").into(), t("non mesurable").into(), None, t("plancher du spectre = bruit numérique (calcul sans bruit de mesure) ; se fier à la part de la dominante").into())
                },
                (t("Entropie spectrale").into(), format!("{:.3}", rep.cogging.spectral_entropy), None, t("plus haut = spectre plus étalé").into()),
                (
                    t(if r.spec.topology == Topology::Axial { "Force transverse max" } else { "Force radiale max" }).into(),
                    format!("{} {}", fnum(u.show(Q::Force, rep.force_max_n)), u.sym(Q::Force)),
                    crit.force_max_n.map(|m| rep.force_max_n < m),
                    match crit.force_max_n {
                        Some(m) => seuil(format!("< {} {}", fnum(u.show(Q::Force, m)), u.sym(Q::Force))),
                        None => t("résultante sur un tour").into(),
                    },
                ),
                (
                    t("Balourd").into(),
                    format!("{} {}", fnum(u.show(Q::Unbalance, rep.balourd_mg_cm)), u.sym(Q::Unbalance)),
                    crit.balourd_max_mg_cm.map(|m| rep.balourd_mg_cm < m),
                    match crit.balourd_max_mg_cm {
                        Some(m) => seuil(format!("< {} {}", fnum(u.show(Q::Unbalance, m)), u.sym(Q::Unbalance))),
                        None => t("masse des aimants répartie autour de l'axe").into(),
                    },
                ),
                (
                    t("Autocorrélation des écarts").into(),
                    rep.autocorr_max.map(|a| format!("{a:.3}")).unwrap_or("—".into()),
                    crit.autocorr_max.and(rep.autocorr_max).map(|a| a < crit.autocorr_max.map_or(1.0, |c| c.0)),
                    match crit.autocorr_max {
                        Some((m, plancher)) => seuil(format!("< {} ; {} ≈ {}", fnum(m), t("plancher"), fnum(plancher))),
                        None => t("plus bas = répartition plus apériodique").into(),
                    },
                ),
                (
                    t("THD de la FEM").into(),
                    format!("{:.2} %", rep.emf_thd_pct),
                    crit.thd_max_pct.map(|m| rep.emf_thd_pct <= m),
                    crit.thd_max_pct.map(|m| seuil(format!("≤ {} %", fnum(m)))).unwrap_or_default(),
                ),
                (
                    t("Interharmoniques FEM").into(),
                    format!("{:.2} %", rep.emf_interharm_pct),
                    None,
                    t("raies hors multiples de p : signature de l'apériodicité").into(),
                ),
                (
                    t("KV estimé (2D)").into(),
                    format!("{:.0} tr/min/V", rep.kv_estime),
                    None,
                    format!("Ke = {:.3} V/krpm {}", rep.ke_ll_v_per_krpm, t("(ligne-ligne crête, sans effets d'extrémité)")),
                ),
                (
                    t("Saturation du fer (en charge)").into(),
                    format!("{:.2} / {:.2} / {:.2} T", sat.b_tooth, sat.b_yoke, sat.b_rotor),
                    Some(sat.b_tooth <= kst + 0.1 && sat.b_yoke <= kst + 0.1 && sat.b_rotor <= kro + 0.1),
                    format!("{} · {} {:+.1} %", t("dents / culasse stator / culasse rotor"), t("flux"), 100.0 * (sat.s - 1.0)),
                ),
            ];
            // alertes d'abord : critères non tenus, puis tenus, puis informatifs
            let mut cards: Vec<(Tab, (String, String, Option<bool>, String))> = card_tab.iter().copied().zip(cards).collect();
            cards.sort_by_key(|(_, c)| match c.2 {
                Some(false) => 0,
                Some(true) => 1,
                None => 2,
            });
            let (card_tab, cards): (Vec<Tab>, Vec<_>) = cards.into_iter().unzip();
            // largeur réellement visible (la zone défilante peut annoncer davantage)
            let aw = (ui.available_width().min(ui.clip_rect().right() - ui.cursor().left()) - 24.0).max(160.0);
            // place insuffisante pour 3 colonnes de cartes complètes : vignettes compactes
            let compact = aw < 3.0 * 270.0;
            let cw = if compact { 190.0 } else { 270.0 };
            let ncol = (((aw + 12.0) / cw).floor() as usize).clamp(1, 6);
            let w = ((aw - 12.0 * (ncol as f32 - 1.0)) / ncol as f32).max(150.0);
            egui::Grid::new("cards").num_columns(ncol).spacing([12.0, 12.0]).show(ui, |ui| {
                for (i, (title, val, ok, tip)) in cards.iter().enumerate() {
                    if card(ui, w, title, val, *ok, tip, self.cfg.theme, compact).clicked() {
                        open_full = Some(card_tab[i]);
                    }
                    if i % ncol == ncol - 1 {
                        ui.end_row();
                    }
                }
            });
            ui.add_space(10.0);
            ui.label(RichText::new(t("Hypothèses : modèle 2D en sous-domaines exacts (multi-tranches en flux axial), fer idéal (μ = ∞) avec correction de saturation par facteur de saturation, inductances avec têtes de bobines estimées. Validé contre un solveur volumes finis indépendant (onglet Validation).")).small().weak());
        });
        if open_full.is_some() {
            self.full = open_full;
        }
        if go3d {
            self.enable_field3d();
        }
        if !self.fresh.is_empty() {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    // ---------------------------- Géométrie ---------------------------
    pub(super) fn tab_geometrie(&mut self, ui: &mut egui::Ui) {
        self.rotor_slider(ui);
        let spec = self.res.as_ref().unwrap().spec.clone();
        // flux axial : vue de face (stator et rotor superposés) + coupe axiale
        if spec.topology == Topology::Axial {
            self.geometrie_axiale(ui, &spec);
            return;
        }
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        ui.columns(2, |cols| {
            let ui = &mut cols[0];
            let size = Vec2::new(ui.available_width(), ui.available_height().max(300.0));
            let (resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
            pan_zoom(ui, &resp, &mut self.geo_view);
            let rect = resp.rect;
            painter.rect_filled(rect, 6.0, view3d::bg());
            let rg = spec.rings();
            let r_out = rg.r_max as f32;
            let sc = rect.width().min(rect.height()) * 0.46 / r_out * self.geo_view.zoom;
            let c = rect.center() + self.geo_view.pan;
            let to = |r: f64, th: f64| c + Vec2::new((r * th.cos()) as f32 * sc, -(r * th.sin()) as f32 * sc);
            let iron = if view3d::canvas_dark() { Color32::from_rgb(85, 95, 115) } else { Color32::from_rgb(150, 160, 175) };
            let q = spec.n_slots;
            let mut mesh = egui::epaint::Mesh::default();
            stator_mesh(&mut mesh, &to, &rg, q, iron);
            sector(&mut mesh, &to, rg.rotor_yoke.0, rg.rotor_yoke.1, 0.0, 2.0 * PI, iron, 180);
            let th_r = self.theta_r.to_radians();
            let pieces = magnets::pieces(&spec);
            let mut hovered_txt = None;
            let hp = resp.hover_pos();
            for p in &pieces {
                let col = match p.kind {
                    PieceKind::Radial { s } => if s > 0.0 { Color32::from_rgb(230, 60, 50) } else { Color32::from_rgb(40, 110, 230) },
                    PieceKind::Fixed { psi } => {
                        let mr = (0.5 * (p.t1 + p.t2) - psi).cos();
                        if mr > 0.0 { Color32::from_rgb(230, 60, 50) } else { Color32::from_rgb(40, 110, 230) }
                    }
                };
                sector(&mut mesh, &to, spec.r_magnet_in(), spec.r_magnet_out(), p.t1 + th_r, p.t2 + th_r, col, 16);
            }
            painter.add(egui::Shape::mesh(mesh));
            // flèches d'aimantation
            for p in &pieces {
                let tc = 0.5 * (p.t1 + p.t2) + th_r;
                let rc = 0.5 * (spec.r_magnet_in() + spec.r_magnet_out());
                let dir = match p.kind {
                    PieceKind::Radial { s } => tc + if s > 0.0 { 0.0 } else { PI },
                    PieceKind::Fixed { psi } => psi + th_r,
                };
                let a = to(rc, tc);
                let len = (spec.magnet_th * 1e-3 * 0.4) as f32 * sc;
                let d = Vec2::new(dir.cos() as f32, -dir.sin() as f32) * len;
                painter.arrow(a - d * 0.5, d, egui::Stroke::new(1.5f32, Color32::WHITE));
            }
            // numéros d'aimants + survol
            let angles = spec.magnet_angles_deg();
            let pitch = 360.0 / angles.len() as f64;
            for (k, a) in angles.iter().enumerate() {
                let th = a.to_radians() + th_r;
                let pos = to(rg.rotor_yoke.1 + (rg.rotor_yoke.1 - rg.rotor_yoke.0) * 0.8, th);
                painter.text(pos, egui::Align2::CENTER_CENTER, format!("{}", k + 1), egui::FontId::proportional(11.0), if view3d::canvas_dark() { Color32::LIGHT_GRAY } else { Color32::DARK_GRAY });
                if let Some(h) = hp
                    && h.distance(to(0.5 * (spec.r_magnet_in() + spec.r_magnet_out()), th)) < (spec.magnet_th * 1e-3) as f32 * sc {
                        hovered_txt = Some(format!(
                            "{} {} ({})\n{} {:.4}°\n{} {:+.4}°",
                            t("Aimant"),
                            k + 1,
                            if k % 2 == 0 { "N" } else { "S" },
                            t("centre"),
                            a,
                            t("écart / régulier"),
                            a - k as f64 * pitch
                        ));
                    }
            }
            if let Some(h) = hp {
                let v = (h - c) / sc;
                let r = (v.x * v.x + v.y * v.y).sqrt() as f64;
                let th = (-v.y as f64).atan2(v.x as f64).to_degrees().rem_euclid(360.0);
                let u = self.cfg.units.clone();
                let txt = hovered_txt.unwrap_or_else(|| format!("r = {} {}\nθ = {:.3}°", fnum(u.show(Q::Length, r * 1e3)), u.sym(Q::Length), th));
                painter.text(h + Vec2::new(14.0, 14.0), egui::Align2::LEFT_TOP, txt, egui::FontId::monospace(12.0), ORANGE);
            }
            painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), egui::Align2::LEFT_BOTTOM, t("molette : zoom · glisser : déplacer · double-clic : recentrer"), egui::FontId::proportional(11.0), Color32::GRAY);

            // colonne droite : écarts angulaires
            let ui = &mut cols[1];
            ui.label(RichText::new(t("Écarts des aimants par rapport au pas régulier")).strong());
            let bars: Vec<Bar> = angles
                .iter()
                .enumerate()
                .map(|(k, a)| Bar::new((k + 1) as f64, a - k as f64 * pitch).width(0.7).fill(if k % 2 == 0 { Color32::from_rgb(230, 60, 50) } else { Color32::from_rgb(40, 110, 230) }))
                .collect();
            Plot::new("ecarts")
                .height(ui.available_height() * 0.55)
                .x_axis_label(t("aimant n°"))
                .y_axis_label(t("écart [°]"))
                .show(ui, |p| p.bar_chart(BarChart::new(bars).element_formatter(Box::new(|b, _| format!("#{:.0} : {:+.4}°", b.argument, b.value)))));
            egui::Grid::new("geo_info").num_columns(2).striped(true).show(ui, |ui| {
                let u = self.cfg.units.clone();
                let l = |v: f64| format!("{} {}", fnum(u.show(Q::Length, v)), u.sym(Q::Length));
                lbl(ui, t("Période de cogging (rotor régulier)"));
                ui.label(format!("{:.4}°", spec.cogging_period_regular_deg()));
                ui.end_row();
                lbl(ui, t("Période réelle de ce rotor"));
                ui.label(format!("{:.4}°", spec.cogging_period_deg()));
                ui.end_row();
                lbl(ui, t("Rayon extérieur rotor"));
                ui.label(l((spec.r_magnet_out() * 1e3) + spec.rotor_yoke_th));
                ui.end_row();
                lbl(ui, t("Profondeur d'encoche"));
                ui.label(l(spec.stator_or - spec.stator_ir - spec.yoke_th));
                ui.end_row();
            });
        });
    }

    // ------------------------------ Champ -----------------------------
    pub(super) fn tab_champ(&mut self, ui: &mut egui::Ui) {
        self.rotor_slider(ui);
        // flux axial : tranches, carte de la tranche développée, entrefer sur tout le disque
        if self.res.as_ref().map(|r| r.spec.topology == Topology::Axial).unwrap_or(false) {
            self.champ_axial(ui);
            return;
        }
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.map_mode_lines, false, tr("Courbes d'entrefer"));
            ui.selectable_value(&mut self.map_mode_lines, true, tr("Carte du champ"));
        });
        let u = self.cfg.units.clone();
        let fb = u.factor(Q::Induction);
        if !self.map_mode_lines {
            let lang = self.cfg.lang;
            let t = |k: &'static str| T(lang, k);
            let db = self.spectrum_db;
            let Some((_, g)) = self.field() else { return };
            let br: Vec<[f64; 2]> = g.theta_deg.iter().zip(&g.br).map(|(x, y)| [*x, y * fb]).collect();
            let bt: Vec<[f64; 2]> = g.theta_deg.iter().zip(&g.bt).map(|(x, y)| [*x, y * fb]).collect();
            let sp = g.spectrum_br.clone();
            let h = ui.available_height() * 0.55;
            let mut cur = *self.cursors.entry("champ").or_default();
            curve_plot(ui, "champ", &[(t("Br radiale").into(), br, CYAN), (t("Bθ tangentielle").into(), bt, ORANGE)], "θ [°]", &format!("B [{}]", u.sym(Q::Induction)), h, &mut cur);
            self.cursors.insert("champ", cur);
            chk(ui, &mut self.spectrum_db, t("spectre en dB"));
            spectrum_plot(ui, "sp_br", &sp, 1.0, fb, t("rang harmonique (par tour)"), &format!("|Br| [{}]", u.sym(Q::Induction)), db, CYAN);
        } else {
            self.field_map_ui(ui);
        }
    }

    pub(super) fn field_map_ui(&mut self, ui: &mut egui::Ui) {
        let fresh = matches!(&self.map_cache, Some((a, _)) if (*a - self.theta_r).abs() < 1e-12);
        if !fresh {
            let r = self.res.as_ref().unwrap();
            let f = r.model.solve(self.theta_r.to_radians());
            let radii = analysis::map_radii(&r.model, 36, 10, 22);
            self.map_cache = Some((self.theta_r, analysis::field_map(&r.model, &f, radii, 1080)));
        }
        let spec = self.res.as_ref().unwrap().spec.clone();
        let (_, fm) = self.map_cache.as_ref().unwrap();
        let u = self.cfg.units.clone();
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let size = Vec2::new(ui.available_width(), ui.available_height());
        let (resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
        pan_zoom(ui, &resp, &mut self.map_view);
        let rect = resp.rect;
        painter.rect_filled(rect, 6.0, view3d::bg());
        let rg = spec.rings();
        let r_out = rg.r_max as f32;
        let sc = (rect.width() - 90.0).min(rect.height()) * 0.47 / r_out * self.map_view.zoom;
        let c = rect.center() - Vec2::new(40.0, 0.0) + self.map_view.pan;
        let to = |r: f64, th: f64| c + Vec2::new((r * th.cos()) as f32 * sc, -(r * th.sin()) as f32 * sc);
        let bmax = 2.5f64;
        let iron = if view3d::canvas_dark() { Color32::from_rgb(60, 66, 80) } else { Color32::from_rgb(175, 182, 195) };
        let mut mesh = egui::epaint::Mesh::default();
        sector(&mut mesh, &to, rg.yoke.0, rg.yoke.1, 0.0, 2.0 * PI, iron, 180);
        sector(&mut mesh, &to, rg.rotor_yoke.0, rg.rotor_yoke.1, 0.0, 2.0 * PI, iron, 180);
        let (nr, nt) = (fm.nr, fm.nt);
        let dth = 2.0 * PI / nt as f64;
        let rb = |i: usize| -> (f64, f64) {
            let lo = if i == 0 { spec.r_slot_bottom() } else { 0.5 * (fm.r[i - 1] + fm.r[i]) };
            let hi = if i + 1 == nr { spec.r_magnet_out() } else { 0.5 * (fm.r[i] + fm.r[i + 1]) };
            (lo, hi)
        };
        for i in 0..nr {
            let (lo, hi) = rb(i);
            for j in 0..nt {
                let col = match fm.b[i * nt + j] {
                    Some(b) => view3d::colormap((b / bmax) as f32),
                    None => iron,
                };
                let th0 = fm.th[j] - dth / 2.0;
                let p = [to(lo, th0), to(hi, th0), to(hi, th0 + dth), to(lo, th0 + dth)];
                let base = mesh.vertices.len() as u32;
                for v in p {
                    mesh.colored_vertex(v, col);
                }
                mesh.add_triangle(base, base + 1, base + 2);
                mesh.add_triangle(base, base + 2, base + 3);
            }
        }
        painter.add(egui::Shape::mesh(mesh));
        // lignes de flux : isovaleurs de A (carrés marchants)
        let (amin, amax) = fm.a.iter().flatten().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        let nlev = 18;
        let stroke = egui::Stroke::new(0.7f32, if view3d::canvas_dark() { Color32::from_white_alpha(110) } else { Color32::from_black_alpha(120) });
        for lv in 1..nlev {
            let level = amin + (amax - amin) * lv as f64 / nlev as f64;
            for i in 0..nr - 1 {
                for j in 0..nt {
                    let jn = (j + 1) % nt;
                    let v = [fm.a[i * nt + j], fm.a[i * nt + jn], fm.a[(i + 1) * nt + jn], fm.a[(i + 1) * nt + j]];
                    if v.iter().any(|x| x.is_none()) {
                        continue;
                    }
                    let v: Vec<f64> = v.iter().map(|x| x.unwrap()).collect();
                    let pos = [(fm.r[i], fm.th[j]), (fm.r[i], fm.th[j] + dth), (fm.r[i + 1], fm.th[j] + dth), (fm.r[i + 1], fm.th[j])];
                    let mut pts = Vec::new();
                    for e in 0..4 {
                        let (a, b) = (v[e], v[(e + 1) % 4]);
                        if (a - level) * (b - level) < 0.0 {
                            let s = (level - a) / (b - a);
                            let (pa, pb) = (pos[e], pos[(e + 1) % 4]);
                            pts.push(to(pa.0 + s * (pb.0 - pa.0), pa.1 + s * (pb.1 - pa.1)));
                        }
                    }
                    if pts.len() >= 2 {
                        painter.line_segment([pts[0], pts[1]], stroke);
                        if pts.len() == 4 {
                            painter.line_segment([pts[2], pts[3]], stroke);
                        }
                    }
                }
            }
        }
        // barre de couleurs
        let bx = rect.right() - 70.0;
        let (by0, by1) = (rect.top() + 30.0, rect.bottom() - 40.0);
        for k in 0..100 {
            let tt = k as f32 / 100.0;
            let y = by1 - (by1 - by0) * tt;
            painter.rect_filled(egui::Rect::from_min_max(egui::pos2(bx, y - (by1 - by0) / 100.0 - 0.5), egui::pos2(bx + 16.0, y)), 0.0, view3d::colormap(tt));
        }
        for k in 0..=5 {
            let v = bmax * k as f64 / 5.0;
            let y = by1 - (by1 - by0) * k as f32 / 5.0;
            painter.text(egui::pos2(bx + 20.0, y), egui::Align2::LEFT_CENTER, fnum(u.show(Q::Induction, v)), egui::FontId::proportional(11.0), Color32::GRAY);
        }
        painter.text(egui::pos2(bx, by0 - 8.0), egui::Align2::LEFT_BOTTOM, format!("|B| [{}]", u.sym(Q::Induction)), egui::FontId::proportional(12.0), Color32::GRAY);
        // survol : lecture exacte
        if let Some(h) = resp.hover_pos() {
            let v = (h - c) / sc;
            let r = (v.x * v.x + v.y * v.y).sqrt() as f64;
            let th = (-v.y as f64).atan2(v.x as f64).rem_euclid(2.0 * PI);
            let model = &self.res.as_ref().unwrap().model;
            let f = model.solve(self.theta_r.to_radians());
            let (_, rs, rm, _) = model.radii();
            let zone = if r < spec.r_slot_bottom() || r > spec.r_magnet_out() {
                t("fer")
            } else if r < rs {
                t("encoche / dent")
            } else if r < rm {
                t("entrefer")
            } else {
                t("aimant")
            };
            let a = model.a_at(&f, r, th);
            let d = 1e-7;
            let b = match (model.a_at(&f, r, th + d / r), model.a_at(&f, r, th - d / r), model.a_at(&f, r + d, th), model.a_at(&f, r - d, th)) {
                (Some(a1), Some(a2), Some(a3), Some(a4)) => Some((((a1 - a2) / (2.0 * d)), -(a3 - a4) / (2.0 * d))),
                _ => None,
            };
            let mut txt = format!("{zone}\nr = {} {}\nθ = {:.3}°", fnum(u.show(Q::Length, r * 1e3)), u.sym(Q::Length), th.to_degrees());
            if let Some((br, bt)) = b {
                txt += &format!(
                    "\nBr = {} {s}\nBθ = {} {s}\n|B| = {} {s}",
                    fnum(u.show(Q::Induction, br)),
                    fnum(u.show(Q::Induction, bt)),
                    fnum(u.show(Q::Induction, br.hypot(bt))),
                    s = u.sym(Q::Induction)
                );
            }
            if let Some(a) = a {
                txt += &format!("\nA = {:.6e} Wb/m", a);
            }
            painter.text(h + Vec2::new(14.0, 14.0), egui::Align2::LEFT_TOP, txt, egui::FontId::monospace(12.0), ORANGE);
        }
        painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), egui::Align2::LEFT_BOTTOM, t("molette : zoom (entrefer) · glisser : déplacer · double-clic : recentrer"), egui::FontId::proportional(11.0), Color32::GRAY);
    }

    // ----------------------------- Cogging ----------------------------
    pub(super) fn tab_cogging(&mut self, ui: &mut egui::Ui) {
        let u = self.cfg.units.clone();
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let r = self.res.as_ref().unwrap();
        let ft = u.factor(Q::Torque);
        let pts: Vec<[f64; 2]> = r.cog.angles_deg.iter().zip(&r.cog.torque_nm).map(|(a, v)| [*a, v * ft]).collect();
        let span = r.cog.metrics.span_deg;
        let sp = r.cog.spectrum.clone();
        let m = r.cog.metrics.clone();
        let name = r.spec.name.clone();
        ui.horizontal_wrapped(|ui| {
            for (k, v) in [
                (t("crête-à-crête"), format!("{} {}", fnum(u.show(Q::Torque, m.cogging_pp_mnm * 1e-3)), u.sym(Q::Torque))),
                (t("efficace"), format!("{} {}", fnum(u.show(Q::Torque, m.cogging_rms_mnm * 1e-3)), u.sym(Q::Torque))),
                (t("dominante"), format!("{:.0}/{}", m.dominant_order_per_rev, t("tour"))),
                (t("part dominante"), format!("{:.1} %", m.dominant_share_pct)),
                (t("entropie"), format!("{:.3}", m.spectral_entropy)),
                (t("fenêtre"), format!("{span:.4}°")),
            ] {
                chip(ui, k, &v);
            }
        });
        let h = ui.available_height() * 0.55;
        let mut cur = *self.cursors.entry("cog").or_default();
        let mut series = vec![(name, pts, CYAN)];
        for (k, (n, p, kind)) in self.overlays.iter().enumerate() {
            if *kind == "cogging" {
                series.push((n.clone(), p.iter().map(|q| [q[0], q[1] * ft]).collect(), SERIES[(k + 1) % 6]));
            }
        }
        curve_plot(ui, "cog", &series, "θr [°]", &format!("{} [{}]", t("couple de détente"), u.sym(Q::Torque)), h, &mut cur);
        self.cursors.insert("cog", cur);
        chk(ui, &mut self.spectrum_db, t("spectre en dB"));
        spectrum_plot(ui, "sp_cog", &sp, 360.0 / span, ft, t("ordre (par tour mécanique)"), &format!("|T| [{}]", u.sym(Q::Torque)), self.spectrum_db, ORANGE);
    }

    // ------------------------------- FEM ------------------------------
    pub(super) fn tab_fem(&mut self, ui: &mut egui::Ui) {
        let u = self.cfg.units.clone();
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let r = self.res.as_ref().unwrap();
        let fv = u.factor(Q::Voltage);
        let e = &r.emf;
        let mk = |v: &Vec<f64>, f: f64| -> Vec<[f64; 2]> { e.angles_deg.iter().zip(v).map(|(a, y)| [*a, y * f]).collect() };
        let ll: Vec<f64> = (0..e.emf[0].len()).map(|k| e.emf[0][k] - e.emf[1][k]).collect();
        let series = vec![
            (format!("{} A", t("phase")), mk(&e.emf[0], fv), SERIES[0]),
            (format!("{} B", t("phase")), mk(&e.emf[1], fv), SERIES[1]),
            (format!("{} C", t("phase")), mk(&e.emf[2], fv), SERIES[2]),
            ("A−B".to_string(), mk(&ll, fv), SERIES[4]),
        ];
        let ff = u.factor(Q::Flux);
        let flux = vec![
            ("ψA".to_string(), mk(&e.flux[0], ff), SERIES[0]),
            ("ψB".to_string(), mk(&e.flux[1], ff), SERIES[1]),
            ("ψC".to_string(), mk(&e.flux[2], ff), SERIES[2]),
        ];
        let sp = e.spectrum_a.clone();
        let rpm = r.spec.rpm;
        ui.horizontal_wrapped(|ui| {
            chip(ui, t("crête phase"), &format!("{} {}", fnum(u.show(Q::Voltage, e.emf_peak_v)), u.sym(Q::Voltage)));
            chip(ui, "THD", &format!("{:.2} %", e.thd_pct));
            chip(ui, t("interharmoniques"), &format!("{:.2} %", e.interharm_pct));
            chip(ui, t("déséquilibre"), &format!("{:.2} %", e.unbalance_pct));
            chip(ui, "Ke", &format!("{:.4} V/krpm", e.ke_ll_v_per_krpm));
            chip(ui, "KV", &format!("{:.0}", e.kv_estime));
            chip(ui, t("vitesse"), &format!("{} {}", fnum(u.show(Q::Speed, rpm)), u.sym(Q::Speed)));
        });
        let h = ui.available_height() * 0.42;
        let mut cur = *self.cursors.entry("fem").or_default();
        let mut series = series;
        for (k, (n, p, kind)) in self.overlays.iter().enumerate() {
            if *kind == "fem" {
                series.push((n.clone(), p.iter().map(|q| [q[0], q[1] * fv]).collect(), SERIES[(k + 3) % 6]));
            }
        }
        curve_plot(ui, "fem", &series, "θr [°]", &format!("{} [{}]", t("FEM"), u.sym(Q::Voltage)), h, &mut cur);
        self.cursors.insert("fem", cur);
        ui.columns(2, |c| {
            let mut cur2 = Cursors::default();
            let hh = c[0].available_height();
            curve_plot(&mut c[0], "flux", &flux, "θr [°]", &format!("{} [{}]", t("flux totalisé"), u.sym(Q::Flux)), hh, &mut cur2);
            spectrum_plot(&mut c[1], "sp_fem", &sp, 1.0, fv, t("ordre (par tour mécanique)"), &format!("|eA| [{}]", u.sym(Q::Voltage)), self.spectrum_db, CYAN);
        });
    }

    // ------------------------------ Forces ----------------------------
    pub(super) fn tab_forces(&mut self, ui: &mut egui::Ui) {
        let u = self.cfg.units.clone();
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let r = self.res.as_ref().unwrap();
        let ff = u.factor(Q::Force);
        let f = &r.forces;
        let mk = |v: &Vec<f64>| -> Vec<[f64; 2]> { f.angles_deg.iter().zip(v).map(|(a, y)| [*a, y * ff]).collect() };
        let mag: Vec<f64> = f.fx.iter().zip(&f.fy).map(|(x, y)| x.hypot(*y)).collect();
        let series = vec![("Fx".to_string(), mk(&f.fx), SERIES[0]), ("Fy".to_string(), mk(&f.fy), SERIES[1]), ("|F|".to_string(), mk(&mag), SERIES[3])];
        let mut series = series;
        if let Some(e) = &self.ecc_curve {
            let em: Vec<f64> = e.fx.iter().zip(&e.fy).map(|(x, y)| x.hypot(*y)).collect();
            series.push((t("|F| avec excentricité").to_string(), e.angles_deg.iter().zip(&em).map(|(a, y)| [*a, y * ff]).collect(), ORANGE));
        }
        let mut orbit: Vec<[f64; 2]> = f.fx.iter().zip(&f.fy).map(|(x, y)| [x * ff, y * ff]).collect();
        if let Some(e) = &self.ecc_curve {
            orbit = e.fx.iter().zip(&e.fy).map(|(x, y)| [x * ff, y * ff]).collect();
        }
        if r.spec.topology == Topology::Axial {
            // flux axial : force transverse (dans le plan du disque) et défauts d'alignement du disque
            ui.label(RichText::new(t("Flux axial : Fx, Fy = force transverse dans le plan du disque (vibration) ; l'attraction axiale de chaque disque est calculée ci-dessous.")).small().weak());
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(t("Défauts d'alignement du rotor")).strong());
                lbl(ui, t("décalage axial"));
                ui.add(egui::Slider::new(&mut self.ax_offset, 0.0..=0.5).custom_formatter(|v, _| format!("{:.0} %", v * 100.0)));
                lbl(ui, t("inclinaison"));
                ui.add(egui::Slider::new(&mut self.ax_tilt, 0.0..=0.5).custom_formatter(|v, _| format!("{:.0} %", v * 100.0)));
                if ui.button(t("Calculer")).on_hover_text(t("Méthode de perméance tranche par tranche : l'entrefer varie de ±δ ; pression magnétique Bn²/2μ0")).clicked()
                    && let Some(rr) = &self.res
                {
                    self.ax_def = Some(crate::analysis::axial_defects(&rr.model, self.ax_offset, self.ax_tilt, 8));
                }
            });
            if let Some(d) = self.ax_def.clone() {
                ui.horizontal_wrapped(|ui| {
                    chip(ui, t("attraction d'un disque"), &format!("{} {}", fnum(u.show(Q::Force, d.attraction_n)), u.sym(Q::Force)));
                    chip(ui, t("force axiale nette"), &format!("{} {}", fnum(u.show(Q::Force, d.offset_force_n)), u.sym(Q::Force)));
                    chip(ui, t("raideur axiale"), &format!("{:.3} N/µm", d.offset_stiffness_n_per_um));
                    chip(ui, t("moment de basculement"), &format!("{:.3} N·m ({:.3} mrad)", d.tilt_moment_nm, d.tilt_mrad));
                    chip(ui, t("raideur angulaire"), &format!("{:.4} N·m/mrad", d.tilt_stiffness_nm_per_mrad));
                    if ui.small_button("✖").clicked() {
                        self.ax_def = None;
                    }
                });
                ui.label(RichText::new(t("Raideurs négatives : le rotor est attiré vers le côté où l'entrefer diminue ; les roulements et la butée doivent être plus raides que ces valeurs.")).small().weak());
            }
        } else {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(t("Excentricité du rotor")).strong());
                ui.add(egui::Slider::new(&mut self.ecc_eps, 0.0..=0.5).custom_formatter(|v, _| format!("{:.0} %", v * 100.0)));
                rad(ui, &mut self.ecc_dyn, false, t("statique"));
                rad(ui, &mut self.ecc_dyn, true, t("dynamique"));
                if ui.button(t("Calculer l'attraction magnétique déséquilibrée")).on_hover_text(t("Méthode de perméance (petite excentricité) : l'entrefer varie de ±ε·g autour du rotor")).clicked()
                    && let Some(rr) = &self.res {
                        self.ecc_curve = Some(crate::analysis::forces_eccentric(&rr.model, self.ecc_eps, self.ecc_dyn, 120));
                    }
                if let Some(e) = &self.ecc_curve {
                    let g = self.spec.airgap;
                    ui.label(RichText::new(format!("{} {} {} · {} {:.3} N/µm", t("max"), fnum(u.show(Q::Force, e.max_n)), u.sym(Q::Force), t("raideur magnétique"), e.mean_n / (self.ecc_eps * g * 1e3).max(1e-9))).color(ORANGE));
                    if ui.small_button("✖").clicked() {
                        self.ecc_curve = None;
                    }
                }
            });
        }
        let r = self.res.as_ref().unwrap();
        let f = &r.forces;
        ui.horizontal_wrapped(|ui| {
            chip(ui, t("max"), &format!("{} {}", fnum(u.show(Q::Force, f.max_n)), u.sym(Q::Force)));
            chip(ui, t("moyenne"), &format!("{} {}", fnum(u.show(Q::Force, f.mean_n)), u.sym(Q::Force)));
            if let Some(m) = crate::repartition::criteria().force_max_n {
                let ok = f.max_n < m;
                let lim = format!("< {} {}", fnum(u.show(Q::Force, m)), u.sym(Q::Force));
                ui.label(RichText::new(if ok { format!("✔ {} ({lim})", t("conforme")) } else { format!("✖ {} ({lim})", t("hors seuil")) }).color(badge(ok)).strong());
            }
        });
        ui.columns(2, |c| {
            let mut cur = *self.cursors.entry("forces").or_default();
            let h = c[0].available_height();
            curve_plot(&mut c[0], "forces", &series, "θr [°]", &format!("{} [{}]", t("force sur le rotor"), u.sym(Q::Force)), h, &mut cur);
            self.cursors.insert("forces", cur);
            c[1].label(RichText::new(t("Orbite de la force (1 tour)")).strong());
            Plot::new("orbit")
                .data_aspect(1.0)
                .x_axis_label(format!("Fx [{}]", u.sym(Q::Force)))
                .y_axis_label(format!("Fy [{}]", u.sym(Q::Force)))
                .show(&mut c[1], |p| {
                    p.line(Line::new(PlotPoints::from(orbit.clone())).color(ORANGE).width(1.8f32).name("F(θr)"));
                    p.points(Points::new(vec![[0.0, 0.0]]).radius(4.0f32).color(CYAN));
                    if let Some(m) = crate::repartition::criteria().force_max_n {
                        let lim = m * ff;
                        let circ: Vec<[f64; 2]> = (0..=120).map(|k| { let a = k as f64 * PI / 60.0; [lim * a.cos(), lim * a.sin()] }).collect();
                        p.line(Line::new(PlotPoints::from(circ)).color(Color32::from_rgb(90, 200, 120)).style(egui_plot::LineStyle::dashed_loose()).name(t("seuil")));
                    }
                });
        });
    }

    // ------------------------------ Charge ----------------------------
    pub(super) fn tab_charge(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| self.tab_charge_body(ui));
    }

    fn tab_charge_body(&mut self, ui: &mut egui::Ui) {
        let u = self.cfg.units.clone();
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let r = self.res.as_ref().unwrap();
        let l = &r.load;
        let ft = u.factor(Q::Torque);
        let st = {
            let p = &self.spec.dynamics;
            let m = crate::dynamics::masses(&r.spec);
            let ic = crate::dynamics::iron_coeffs(&r.iron, &m, &r.spec, p);
            crate::dynamics::evaluate(r, &ic, &m, p, r.spec.rpm)
        };
        let eff = if l.p_mech > 0.0 { 100.0 * (l.p_mech - st.p_total) / (l.p_mech + l.p_joule) } else { 0.0 };
        ui.horizontal_wrapped(|ui| {
            chip(ui, t("courant crête"), &format!("{:.2} A", r.spec.load.i_peak));
            chip(ui, t("couple moyen"), &format!("{} {}", fnum(u.show(Q::Torque, l.avg_torque)), u.sym(Q::Torque)));
            chip(ui, t("ondulation"), &format!("{:.2} %", l.ripple_pct));
            chip(ui, "Kt", &format!("{} {}/A", fnum(u.show(Q::Torque, l.kt)), u.sym(Q::Torque)));
            chip(ui, t("puissance"), &format!("{:.2} W", l.p_mech));
            chip(ui, "R", &format!("{} Ω", fnum(l.r_phase)));
            chip(ui, "Ls", &format!("{} mH", fnum(l.ls * 1e3)));
            chip(ui, t("dont têtes de bobines"), &format!("{} mH", fnum(l.l_end * 1e3)));
            chip(ui, t("pertes Joule"), &format!("{:.3} W", l.p_joule));
            chip(ui, t("rendement estimé"), &format!("{eff:.1} %"));
        });
        let pts: Vec<[f64; 2]> = l.angles_deg.iter().zip(&l.torque).map(|(a, v)| [*a, v * ft]).collect();
        let pe: Vec<[f64; 2]> = l.angles_deg.iter().zip(&l.torque_energy).map(|(a, v)| [*a, v * ft]).collect();
        let mut series = vec![(t("couple en charge (Maxwell)").to_string(), pts, CYAN), (t("contrôle énergétique Σ i·dλ/dθ").to_string(), pe, SERIES[2])];
        for (k, (n, p, kind)) in self.overlays.iter().enumerate() {
            if *kind == "charge" {
                series.push((n.clone(), p.iter().map(|q| [q[0], q[1] * ft]).collect(), SERIES[(k + 3) % 6]));
            }
        }
        let h = 260.0;
        let mut cur = *self.cursors.entry("charge").or_default();
        curve_plot(ui, "charge", &series, "θr [°]", &format!("{} [{}]", t("couple"), u.sym(Q::Torque)), h, &mut cur);
        self.cursors.insert("charge", cur);
        let sweep: Vec<[f64; 2]> = l.angle_sweep.iter().map(|v| [v[0], v[1] * ft]).collect();
        let curr: Vec<(String, Vec<[f64; 2]>, Color32)> = (0..3)
            .map(|k| (format!("i{}", ["A", "B", "C"][k]), l.angles_deg.iter().zip(&l.currents[k]).map(|(a, v)| [*a, *v]).collect(), SERIES[k]))
            .collect();
        let lm = l.inductances;
        ui.columns(3, |c| {
            let hh = 220.0;
            Plot::new("sweep").height(hh).x_axis_label(t("angle du courant [°él]")).y_axis_label(format!("{} [{}]", t("couple moyen"), u.sym(Q::Torque))).show(&mut c[0], |p| {
                p.line(Line::new(PlotPoints::from(sweep)).color(ORANGE).width(2.0f32).name(t("couple moyen")));
                p.points(Points::new(vec![[r.spec.load.angle_deg, l.avg_torque * ft]]).radius(5.0f32).color(CYAN));
            });
            Plot::new("courants").height(hh).legend(Legend::default()).x_axis_label("θr [°]").y_axis_label("A").show(&mut c[1], |p| {
                for (n, pts, col) in &curr {
                    p.line(Line::new(PlotPoints::from(pts.clone())).name(n).color(*col));
                }
            });
            let ui = &mut c[2];
            ui.label(RichText::new(t("Matrice d'inductances [mH] (hors têtes de bobines)")).strong());
            egui::Grid::new("lmat").striped(true).num_columns(4).show(ui, |ui| {
                ui.label("");
                for n in ["A", "B", "C"] {
                    ui.label(RichText::new(n).strong());
                }
                ui.end_row();
                for i in 0..3 {
                    ui.label(RichText::new(["A", "B", "C"][i]).strong());
                    for j in 0..3 {
                        ui.label(RichText::new(format!("{:.5}", lm[i][j] * 1e3)).monospace());
                    }
                    ui.end_row();
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new(t("Courants sinusoïdaux équilibrés alignés sur la FEM (angle réglable dans « Bobinage général et charge »). Rendement : puissance utile / (puissance + pertes Joule), pertes fer et mécaniques déduites.")).small().weak());
        });
        self.saturation_block(ui);
    }

    /// Bloc « Saturation magnétique » de l'onglet Charge.
    fn saturation_block(&mut self, ui: &mut egui::Ui) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let u = self.cfg.units.clone();
        let Some(r) = self.res.as_ref() else { return };
        let c = &r.sat;
        let ft = u.factor(Q::Torque);
        ui.separator();
        ui.label(RichText::new(format!("🔩 {} — {} / {}", t("Saturation magnétique"), t(r.spec.saturation.stator.name()), t(r.spec.saturation.rotor.name()))).strong());
        let col = |b: f64, knee: f64| if b > knee + 0.15 { Color32::from_rgb(255, 90, 80) } else if b > knee { ORANGE } else { Color32::from_rgb(110, 210, 120) };
        let knee = c.knee;
        let rk = r.spec.saturation.rotor.knee();
        for (name, p) in [(t("à vide"), &c.no_load), (t("en charge"), &c.at_load)] {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(format!("{name} :")).strong());
                ui.label(RichText::new(format!("{} {:.2} T", t("dents"), p.b_tooth)).color(col(p.b_tooth, knee)));
                ui.label(RichText::new(format!("{} {:.2} T", t("culasse stator"), p.b_yoke)).color(col(p.b_yoke, knee)));
                ui.label(RichText::new(format!("{} {:.2} T", t("culasse rotor"), p.b_rotor)).color(col(p.b_rotor, rk)));
                ui.label(format!("k_sat {:.3} · {} {:+.1} %", p.k_sat, t("flux"), 100.0 * (p.s - 1.0)));
            });
        }
        let emf_lin = r.emf.emf[0].iter().fold(0.0_f64, |a, v| a.max(v.abs()));
        ui.horizontal_wrapped(|ui| {
            chip(ui, t("FEM crête corrigée"), &format!("{:.3} V ({:+.1} %)", emf_lin * c.no_load.s, 100.0 * (c.no_load.s - 1.0)));
            chip(ui, t("couple moyen corrigé"), &format!("{} {} ({:+.1} %)", fnum(u.show(Q::Torque, r.load.avg_torque * c.at_load.s)), u.sym(Q::Torque), 100.0 * (c.at_load.s - 1.0)));
        });
        let lin: Vec<[f64; 2]> = c.torque_current.iter().map(|v| [v[0], v[1] * ft]).collect();
        let sat: Vec<[f64; 2]> = c.torque_current.iter().map(|v| [v[0], v[2] * ft]).collect();
        let i0 = r.spec.load.i_peak;
        Plot::new("sat_tc").height(240.0).legend(Legend::default().position(Corner::LeftTop)).x_axis_label(t("courant de phase crête [A]")).y_axis_label(format!("{} [{}]", t("couple moyen"), u.sym(Q::Torque))).show(ui, |p| {
            p.line(Line::new(PlotPoints::from(lin)).color(SERIES[0]).style(egui_plot::LineStyle::dashed_loose()).name(t("linéaire (fer idéal)")));
            p.line(Line::new(PlotPoints::from(sat)).color(ORANGE).width(2.0f32).name(t("avec saturation")));
            p.vline(VLine::new(i0).color(Color32::GRAY).name(t("courant nominal")));
        });
        ui.label(RichText::new(t("Méthode du facteur de saturation : estimation d'ingénieur sur circuit magnétique équivalent, non validée par éléments finis non linéaires.")).small().weak());
    }
}
