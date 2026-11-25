//! Dessin des machines à flux axial : vue de face (stator et rotor
//! superposés, le long de l'axe), coupe axiale (empilement des disques) et
//! moteur 3D en disques. Les cotes viennent de `MotorSpec::axial_stack`,
//! comme l'export STL.

use super::*;
use crate::geometry::AxialLayer;

/// Couleurs des couches.
fn layer_col(l: AxialLayer, dark: bool) -> Color32 {
    match l {
        AxialLayer::RotorYoke => Color32::from_rgb(110, 118, 135),
        AxialLayer::Magnets => Color32::from_rgb(200, 70, 150),
        AxialLayer::Airgap => if dark { Color32::from_rgb(30, 38, 52) } else { Color32::from_rgb(235, 240, 248) },
        AxialLayer::Teeth => Color32::from_rgb(176, 120, 70),
        AxialLayer::StatorYoke => if dark { Color32::from_rgb(85, 95, 115) } else { Color32::from_rgb(150, 160, 175) },
    }
}

/// Nom de la couche (clé de traduction).
fn layer_name(l: AxialLayer) -> &'static str {
    match l {
        AxialLayer::RotorYoke => "Culasse rotor",
        AxialLayer::Magnets => "Aimants",
        AxialLayer::Airgap => "Entrefer",
        AxialLayer::Teeth => "Dents et bobines",
        AxialLayer::StatorYoke => "Culasse stator",
    }
}

const N_POLE: Color32 = Color32::from_rgb(230, 60, 50);
const S_POLE: Color32 = Color32::from_rgb(40, 110, 230);

impl App {
    /// Onglet Géométrie d'une machine à flux axial.
    pub(super) fn geometrie_axiale(&mut self, ui: &mut egui::Ui, spec: &MotorSpec) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let dark = view3d::canvas_dark();
        let th_r = self.theta_r.to_radians();
        let (ri, ro) = (spec.axial.r_in, spec.axial.r_out);
        let q = spec.n_slots;
        let angles = spec.magnet_angles_deg();
        let npole = angles.len().max(1);
        let pitch = 360.0 / npole as f64;
        ui.columns(2, |cols| {
            let ui = &mut cols[0];
            ui.horizontal(|ui| {
                ui.label(RichText::new(t("Vue de face (le long de l'axe) : stator et rotor superposés")).strong());
                chk(ui, &mut self.axial_show_rotor, t("Rotor superposé"));
            });
            // ---------------- vue de face ----------------
            let h_face = (ui.available_height() * 0.58).max(260.0);
            let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), h_face), egui::Sense::click_and_drag());
            pan_zoom(ui, &resp, &mut self.geo_view);
            let rect = resp.rect;
            painter.rect_filled(rect, 6.0, view3d::bg());
            let sc = rect.width().min(rect.height()) * 0.46 / ro as f32 * self.geo_view.zoom;
            let c = rect.center() + self.geo_view.pan;
            let to = |r: f64, th: f64| c + Vec2::new((r * th.cos()) as f32 * sc, -(r * th.sin()) as f32 * sc);
            let mut mesh = egui::epaint::Mesh::default();
            // disque du stator (fer)
            sector(&mut mesh, &to, ri, ro, 0.0, 2.0 * PI, layer_col(AxialLayer::StatorYoke, dark), 180);
            // encoches : largeur constante en mm, donc angle variable avec le rayon (dents trapézoïdales)
            let coils = winding::layout(spec);
            let phase_col = [Color32::from_rgb(220, 160, 60), Color32::from_rgb(90, 200, 120), Color32::from_rgb(170, 120, 255)];
            let wbody = spec.slot_w.max(spec.slot_open);
            let poly = |mesh: &mut egui::epaint::Mesh, centre: f64, half_at: &dyn Fn(f64) -> f64, col: Color32| {
                let n = 12;
                let base = mesh.vertices.len() as u32;
                for k in 0..=n {
                    let r = ri + (ro - ri) * k as f64 / n as f64;
                    let h = half_at(r);
                    mesh.colored_vertex(to(r, centre - h), col);
                    mesh.colored_vertex(to(r, centre + h), col);
                }
                for k in 0..n as u32 {
                    let a = base + 2 * k;
                    mesh.add_triangle(a, a + 1, a + 3);
                    mesh.add_triangle(a, a + 3, a + 2);
                }
            };
            for i in 0..q {
                let centre = (i as f64 + 0.5) * 2.0 * PI / q as f64;
                // bobine de la dent voisine : teinte de sa phase
                let ph = coils.get((i + 1) % q).map(|c| c.phase).unwrap_or(0);
                let pc = phase_col[ph];
                let col = Color32::from_rgb((pc.r() as u16 * 4 / 5) as u8, (pc.g() as u16 * 4 / 5) as u8, (pc.b() as u16 * 4 / 5) as u8);
                poly(&mut mesh, centre, &|r| (wbody / 2.0) / r, col);
            }
            painter.add(egui::Shape::mesh(mesh));
            // aimants du rotor par-dessus, semi-transparents (arc au rayon intérieur et au rayon extérieur)
            let half_in = spec.magnet_arc_ratio * pitch.to_radians() / 2.0;
            let half_out = spec.axial.arc_ratio_out * pitch.to_radians() / 2.0;
            let mut hovered_txt = None;
            let hp = resp.hover_pos();
            if self.axial_show_rotor {
                let mut mm = egui::epaint::Mesh::default();
                for (k, a) in angles.iter().enumerate() {
                    let centre = a.to_radians() + th_r;
                    let col = if k % 2 == 0 { N_POLE } else { S_POLE }.gamma_multiply(0.42);
                    poly(&mut mm, centre, &|r| half_in + (half_out - half_in) * (r - ri) / (ro - ri).max(1e-9), col);
                }
                painter.add(egui::Shape::mesh(mm));
                // contour de chaque aimant (le stator reste lisible dessous)
                for (k, a) in angles.iter().enumerate() {
                    let centre = a.to_radians() + th_r;
                    let half = |r: f64| half_in + (half_out - half_in) * (r - ri) / (ro - ri).max(1e-9);
                    let mut pts: Vec<egui::Pos2> = (0..=8).map(|j| ri + (ro - ri) * j as f64 / 8.0).map(|r| to(r, centre - half(r))).collect();
                    pts.extend((0..=8).rev().map(|j| ri + (ro - ri) * j as f64 / 8.0).map(|r| to(r, centre + half(r))));
                    painter.add(egui::Shape::closed_line(pts, egui::Stroke::new(1.4f32, if k % 2 == 0 { N_POLE } else { S_POLE })));
                }
                for (k, a) in angles.iter().enumerate() {
                    let th = a.to_radians() + th_r;
                    painter.text(to(ro * 1.06, th), egui::Align2::CENTER_CENTER, format!("{}", k + 1), egui::FontId::proportional(11.0), if dark { Color32::LIGHT_GRAY } else { Color32::DARK_GRAY });
                    if let Some(h) = hp
                        && h.distance(to(0.5 * (ri + ro), th)) < (0.25 * (ro - ri)) as f32 * sc
                    {
                        hovered_txt = Some(format!("{} {} ({})\n{} {:.4}°\n{} {:+.4}°", t("Aimant"), k + 1, if k % 2 == 0 { "N" } else { "S" }, t("centre"), a, t("écart / régulier"), a - k as f64 * pitch));
                    }
                }
            }
            // rayons actifs
            let stroke = egui::Stroke::new(1.0f32, Color32::GRAY);
            painter.circle_stroke(c, (ri as f32) * sc, stroke);
            painter.circle_stroke(c, (ro as f32) * sc, stroke);
            painter.circle_filled(c, (ri * 0.35) as f32 * sc, layer_col(AxialLayer::RotorYoke, dark).gamma_multiply(0.6));
            if let Some(h) = hp {
                let v = (h - c) / sc;
                let r = (v.x * v.x + v.y * v.y).sqrt() as f64;
                let th = (-v.y as f64).atan2(v.x as f64).to_degrees().rem_euclid(360.0);
                let u = self.cfg.units.clone();
                let txt = hovered_txt.unwrap_or_else(|| format!("r = {} {}\nθ = {:.3}°", fnum(u.show(Q::Length, r)), u.sym(Q::Length), th));
                painter.text(h + Vec2::new(14.0, 14.0), egui::Align2::LEFT_TOP, txt, egui::FontId::monospace(12.0), ORANGE);
            }
            painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), egui::Align2::LEFT_BOTTOM, t("molette : zoom · glisser : déplacer · double-clic : recentrer"), egui::FontId::proportional(11.0), Color32::GRAY);

            // ---------------- coupe axiale ----------------
            ui.add_space(6.0);
            ui.label(RichText::new(t("Coupe axiale (plan r–z) : empilement des disques le long de l'axe")).strong());
            let stack = spec.axial_stack();
            let (zmin, zmax) = (stack.first().map(|l| l.1).unwrap_or(0.0), stack.last().map(|l| l.2).unwrap_or(1.0));
            let h_cut = (ui.available_height() - 70.0).max(160.0);
            let (resp2, p2) = ui.allocate_painter(Vec2::new(ui.available_width(), h_cut), egui::Sense::hover());
            let rr = resp2.rect;
            p2.rect_filled(rr, 6.0, view3d::bg());
            let span_z = (zmax - zmin).max(1e-6);
            // demi-coupe (au-dessus de l'axe) cadrée sur la zone active : de 0,7·r int. à r ext.
            let (r_lo_v, r_hi_v) = (ri * 0.7, ro * 1.03);
            let s2 = ((rr.width() - 120.0) / span_z as f32).min((rr.height() - 36.0) / (r_hi_v - r_lo_v) as f32);
            let cz = rr.center().x + 30.0 - ((zmin + zmax) / 2.0) as f32 * s2;
            let base_y = rr.bottom() - 22.0;
            let pt = |z: f64, r: f64| egui::pos2(cz + z as f32 * s2, base_y - (r - r_lo_v) as f32 * s2);
            // repère de l'axe (sous la vue, hors échelle)
            let ay = rr.bottom() - 8.0;
            p2.line_segment([egui::pos2(pt(zmin, 0.0).x - 10.0, ay), egui::pos2(pt(zmax, 0.0).x + 10.0, ay)], egui::Stroke::new(1.0f32, Color32::GRAY));
            p2.text(egui::pos2(pt(zmax, 0.0).x + 14.0, ay), egui::Align2::LEFT_CENTER, format!("{} (r = 0, {} {:.0} mm)", t("axe"), t("à"), r_lo_v), egui::FontId::proportional(10.0), Color32::GRAY);
            let mut hover_layer = None;
            for &(l, z0, z1) in &stack {
                let col = layer_col(l, dark);
                // les disques rotors descendent plus près de l'arbre (moyeu)
                let r_lo = if l == AxialLayer::RotorYoke { ri * 0.8 } else { ri };
                let rc = egui::Rect::from_two_pos(pt(z0, r_lo), pt(z1, ro));
                if l == AxialLayer::Airgap {
                    p2.rect_stroke(rc, 0.0, egui::Stroke::new(1.0f32, Color32::from_gray(120)));
                } else {
                    p2.rect_filled(rc, 0.0, col);
                }
                // épaisseur écrite dans la couche quand elle est assez large
                if rc.width() > 34.0 {
                    p2.text(rc.center(), egui::Align2::CENTER_CENTER, format!("{:.1}", z1 - z0), egui::FontId::proportional(10.0), Color32::WHITE);
                }
                if let Some(h) = resp2.hover_pos()
                    && rc.expand(2.0).contains(h)
                {
                    hover_layer = Some((l, z0, z1));
                }
            }
            if let Some((l, z0, z1)) = hover_layer
                && let Some(h) = resp2.hover_pos()
            {
                p2.text(h + Vec2::new(12.0, 12.0), egui::Align2::LEFT_TOP, format!("{}\n{:.2} mm (z {:.2} … {:.2})", t(layer_name(l)), z1 - z0, z0, z1), egui::FontId::monospace(12.0), ORANGE);
            }
            // cotes radiales
            for (r, lab) in [(ri, "r int."), (ro, "r ext.")] {
                p2.line_segment([pt(zmin, r), pt(zmin - span_z * 0.04, r)], egui::Stroke::new(1.0f32, Color32::GRAY));
                p2.text(pt(zmin - span_z * 0.05, r), egui::Align2::RIGHT_CENTER, format!("{lab} {r:.1}"), egui::FontId::proportional(10.0), Color32::GRAY);
            }
            // légende
            ui.horizontal_wrapped(|ui| {
                let mut seen = Vec::new();
                for &(l, z0, z1) in &stack {
                    if seen.contains(&l) {
                        continue;
                    }
                    seen.push(l);
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, 2.0, layer_col(l, dark));
                    ui.label(RichText::new(format!("{} {:.2} mm", t(layer_name(l)), z1 - z0)).small());
                }
            });

            // ---------------- colonne droite ----------------
            let ui = &mut cols[1];
            ui.label(RichText::new(t("Écarts des aimants par rapport au pas régulier")).strong());
            let bars: Vec<Bar> = angles.iter().enumerate().map(|(k, a)| Bar::new((k + 1) as f64, a - k as f64 * pitch).width(0.7).fill(if k % 2 == 0 { N_POLE } else { S_POLE })).collect();
            Plot::new("ecarts_ax")
                .height(ui.available_height() * 0.5)
                .x_axis_label(t("aimant n°"))
                .y_axis_label(t("écart [°]"))
                .show(ui, |p| p.bar_chart(BarChart::new(bars).element_formatter(Box::new(|b, _| format!("#{:.0} : {:+.4}°", b.argument, b.value)))));
            egui::Grid::new("geo_info_ax").num_columns(2).striped(true).show(ui, |ui| {
                let u = self.cfg.units.clone();
                let l = |v_mm: f64| format!("{} {}", fnum(u.show(Q::Length, v_mm)), u.sym(Q::Length));
                lbl(ui, t("Type de machine"));
                ui.label(t("Flux axial (multi-tranches)"));
                ui.end_row();
                lbl(ui, t("Rayon intérieur actif"));
                ui.label(l(ri));
                ui.end_row();
                lbl(ui, t("Rayon extérieur actif"));
                ui.label(l(ro));
                ui.end_row();
                lbl(ui, t("Nombre d'entrefers"));
                ui.label(format!("{}", spec.axial.gaps.clamp(1, 2)));
                ui.end_row();
                lbl(ui, t("Longueur axiale totale"));
                ui.label(l(zmax - zmin));
                ui.end_row();
                lbl(ui, t("Profondeur d'encoche"));
                ui.label(l(spec.slot_depth_m() * 1e3));
                ui.end_row();
                lbl(ui, t("Période de cogging (rotor régulier)"));
                ui.label(format!("{:.4}°", spec.cogging_period_regular_deg()));
                ui.end_row();
                lbl(ui, t("Période réelle de ce rotor"));
                ui.label(format!("{:.4}°", spec.cogging_period_deg()));
                ui.end_row();
            });
        });
    }

    /// Scène 3D d'une machine à flux axial : disques empilés le long de
    /// l'axe (stator au centre), rotor avant découpé pour voir l'intérieur.
    /// `tooth_col` donne la couleur de chaque dent ; renvoie les faces, en
    /// repère normalisé (rayon extérieur = 1).
    pub(super) fn axial_quads(spec: &MotorSpec, th_r: f64, tooth_col: &dyn Fn(usize) -> Color32) -> Vec<Quad> {
        let (ri, ro) = (spec.axial.r_in, spec.axial.r_out);
        let s = 1.0 / ro;
        let q = spec.n_slots;
        let mut quads = Vec::new();
        let stack = spec.axial_stack();
        let iron = Color32::from_rgb(140, 150, 170);
        let rm = 0.5 * (ri + ro);
        let half_slot = spec.slot_w.max(spec.slot_open) / 2.0 / rm;
        let angles = spec.magnet_angles_deg();
        let pitch = 360.0 / angles.len().max(1) as f64;
        let half_mag = 0.5 * (spec.magnet_arc_ratio + spec.axial.arc_ratio_out) * 0.5 * pitch.to_radians();
        let zmax = stack.last().map(|l| l.2).unwrap_or(0.0);
        for &(l, z0, z1) in &stack {
            let (za, zb) = ((z0 * s) as f32, (z1 * s) as f32);
            match l {
                AxialLayer::StatorYoke => ring_sector(&mut quads, ri * s, ro * s, 0.0, 2.0 * PI, za, zb, iron, 72, 2000),
                AxialLayer::Teeth => {
                    for i in 0..q {
                        let a0 = (i as f64 + 0.5) * 2.0 * PI / q as f64 + half_slot;
                        let a1 = (i as f64 + 1.5) * 2.0 * PI / q as f64 - half_slot;
                        let tooth = (i + 1) % q;
                        ring_sector(&mut quads, ri * s, ro * s, a0, a1, za, zb, tooth_col(tooth), 6, 1000 + tooth as u32);
                    }
                }
                AxialLayer::Magnets => {
                    for (k, a) in angles.iter().enumerate() {
                        let c = a.to_radians() + th_r;
                        let col = if k % 2 == 0 { Color32::from_rgb(230, 70, 60) } else { Color32::from_rgb(50, 120, 235) };
                        ring_sector(&mut quads, ri * s, ro * s, c - half_mag, c + half_mag, za, zb, col, 4, k as u32);
                    }
                }
                AxialLayer::RotorYoke => {
                    // disque rotor du côté de la caméra (z > 0) découpé d'un quart pour voir l'intérieur
                    let (b0, b1) = if z1 >= zmax - 1e-9 { (PI * 0.5 + th_r, 2.0 * PI + th_r) } else { (0.0, 2.0 * PI) };
                    ring_sector(&mut quads, ri * 0.8 * s, ro * s, b0, b1, za, zb, Color32::from_rgb(110, 118, 135), 54, 3000);
                }
                AxialLayer::Airgap => {}
            }
        }
        quads
    }

    /// Onglet Champ d'une machine à flux axial.
    pub(super) fn champ_axial(&mut self, ui: &mut egui::Ui) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let u = self.cfg.units.clone();
        let fb = u.factor(Q::Induction);
        let Some(r) = self.res.as_ref() else { return };
        let ns = r.model.n_sections();
        let k = self.axial_slice.unwrap_or(r.model.reference_index()).min(ns - 1);
        let (_, dr) = r.model.slice_radius(k);
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.axial_champ_mode, 0, t("Courbes d'entrefer"));
            ui.selectable_value(&mut self.axial_champ_mode, 1, t("Carte du champ"));
            ui.selectable_value(&mut self.axial_champ_mode, 2, t("Entrefer sur tout le disque"));
            ui.separator();
            if self.axial_champ_mode != 2 {
                lbl(ui, t("Tranche"));
                let label = |i: usize| {
                    let (ri, _) = r.model.slice_radius(i);
                    format!("{} / {} — r = {:.1} mm", i + 1, ns, ri * 1e3)
                };
                let mut sel = k;
                egui::ComboBox::from_id_source("tranche").selected_text(label(k)).show_ui(ui, |ui| {
                    for i in 0..ns {
                        ui.selectable_value(&mut sel, i, label(i));
                    }
                });
                if sel != k {
                    self.axial_slice = Some(sel);
                }
                ui.label(RichText::new(format!("{} {:.1} mm", t("largeur radiale"), dr * 1e3)).weak());
            }
        });
        let k = self.axial_slice.unwrap_or(r.model.reference_index()).min(ns - 1);
        match self.axial_champ_mode {
            0 => {
                let f = r.model.solve(self.theta_r.to_radians());
                let g = analysis::gap_field_slice(&r.model, &f, k, 1440);
                let bz: Vec<[f64; 2]> = g.theta_deg.iter().zip(&g.br).map(|(x, y)| [*x, y * fb]).collect();
                let bt: Vec<[f64; 2]> = g.theta_deg.iter().zip(&g.bt).map(|(x, y)| [*x, y * fb]).collect();
                let h = ui.available_height() * 0.55;
                let mut cur = *self.cursors.entry("champ").or_default();
                curve_plot(ui, "champ_ax", &[(t("Bz axiale (normale)").into(), bz, CYAN), (t("Bθ tangentielle").into(), bt, ORANGE)], "θ [°]", &format!("B [{}]", u.sym(Q::Induction)), h, &mut cur);
                self.cursors.insert("champ", cur);
                let db = self.spectrum_db;
                chk(ui, &mut self.spectrum_db, t("spectre en dB"));
                spectrum_plot(ui, "sp_bz", &g.spectrum_br, 1.0, fb, t("rang harmonique (par tour)"), &format!("|Bz| [{}]", u.sym(Q::Induction)), db, CYAN);
            }
            1 => self.slice_map_ui(ui, k),
            _ => self.disc_map_ui(ui),
        }
    }

    /// Carte du champ de la tranche `k` développée : circonférence en
    /// abscisse (longueur réelle à r0), cote axiale en ordonnée.
    fn slice_map_ui(&mut self, ui: &mut egui::Ui, k: usize) {
        let fresh = matches!(&self.slice_map_cache, Some((a, kk, _)) if (*a - self.theta_r).abs() < 1e-12 && *kk == k);
        if !fresh {
            let r = self.res.as_ref().unwrap();
            let f = r.model.solve(self.theta_r.to_radians());
            self.slice_map_cache = Some((self.theta_r, k, analysis::slice_field_map(&r.model, &f, k, 30, 10, 18, 2160)));
        }
        let lang = self.cfg.lang;
        let t = |kk: &'static str| T(lang, kk);
        let u = self.cfg.units.clone();
        let dark = view3d::canvas_dark();
        let r = self.res.as_ref().unwrap();
        let spec = r.spec.clone();
        let sec = &r.model.sections()[k];
        let g = sec.g.clone();
        let r0 = r.model.slice_radius(k).0;
        let (_, _, fm) = self.slice_map_cache.as_ref().unwrap();
        let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), ui.available_height() - 4.0), egui::Sense::click_and_drag());
        pan_zoom(ui, &resp, &mut self.slice_view);
        let rect = resp.rect;
        painter.rect_filled(rect, 6.0, view3d::bg());
        // cotes : y du fond d'encoche jusqu'au dos des aimants, plus les culasses
        let (ylo, yhi) = (g.y_sb.min(g.y_r), g.y_sb.max(g.y_r));
        let yk_s = spec.yoke_th * 1e-3;
        let yk_r = spec.rotor_yoke_th * 1e-3;
        let (y_min, y_max) = (ylo - yk_s, yhi + yk_r);
        let h_px = rect.height() - 50.0;
        let sc = h_px / (y_max - y_min) as f32 * self.slice_view.zoom;
        // abscisse : longueur d'arc r0·θ ; vue initiale sur environ trois pas d'encoche
        let x_of = |th: f64| (r0 * th) as f32 * sc;
        let x0 = rect.left() + 60.0 + self.slice_view.pan.x;
        let y0 = rect.bottom() - 30.0 + self.slice_view.pan.y;
        let to = |y: f64, th: f64| egui::pos2(x0 + x_of(th), y0 - ((y - y_min) as f32) * sc);
        let iron = if dark { Color32::from_rgb(60, 66, 80) } else { Color32::from_rgb(175, 182, 195) };
        let painter = painter.with_clip_rect(rect.shrink(2.0));
        // culasses (fer)
        painter.rect_filled(egui::Rect::from_two_pos(to(y_min, 0.0), to(ylo, 2.0 * PI)), 0.0, iron);
        painter.rect_filled(egui::Rect::from_two_pos(to(yhi, 0.0), to(y_max, 2.0 * PI)), 0.0, iron);
        let (nr, nt) = (fm.nr, fm.nt);
        let dth = 2.0 * PI / nt as f64;
        let bmax = 2.5f64;
        let yb = |i: usize| -> (f64, f64) {
            let lo = if i == 0 { ylo } else { 0.5 * (fm.r[i - 1] + fm.r[i]) };
            let hi = if i + 1 == nr { yhi } else { 0.5 * (fm.r[i] + fm.r[i + 1]) };
            (lo, hi)
        };
        // cellules visibles seulement (la circonférence peut être très longue)
        let j_vis = |j: usize| {
            let x = x0 + x_of(fm.th[j]);
            x > rect.left() - 20.0 && x < rect.right() + 20.0
        };
        let mut mesh = egui::epaint::Mesh::default();
        for i in 0..nr {
            let (lo, hi) = yb(i);
            for j in (0..nt).filter(|&j| j_vis(j)) {
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
        // lignes de flux : isovaleurs de A
        let (amin, amax) = fm.a.iter().flatten().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        let stroke = egui::Stroke::new(0.7f32, if dark { Color32::from_white_alpha(110) } else { Color32::from_black_alpha(120) });
        for lv in 1..18 {
            let level = amin + (amax - amin) * lv as f64 / 18.0;
            for i in 0..nr - 1 {
                for j in (0..nt).filter(|&j| j_vis(j)) {
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
        // repères : zones, cote axiale
        let lab = |y: f64, s: &str| painter.text(egui::pos2(rect.left() + 4.0, to(y, 0.0).y), egui::Align2::LEFT_CENTER, s, egui::FontId::proportional(10.0), Color32::GRAY);
        lab(0.5 * (y_min + ylo), t("culasse stator"));
        lab(0.5 * (g.y_sb + g.y_s), t("encoches"));
        lab(0.5 * (g.y_m + g.y_r), t("Aimants"));
        lab(0.5 * (yhi + y_max), t("culasse rotor"));
        // barre de couleurs
        let bx = rect.right() - 70.0;
        let (by0, by1) = (rect.top() + 30.0, rect.bottom() - 40.0);
        for kk in 0..100 {
            let tt = kk as f32 / 100.0;
            let y = by1 - (by1 - by0) * tt;
            painter.rect_filled(egui::Rect::from_min_max(egui::pos2(bx, y - (by1 - by0) / 100.0 - 0.5), egui::pos2(bx + 16.0, y)), 0.0, view3d::colormap(tt));
        }
        for kk in 0..=5 {
            let v = bmax * kk as f64 / 5.0;
            let y = by1 - (by1 - by0) * kk as f32 / 5.0;
            painter.text(egui::pos2(bx + 20.0, y), egui::Align2::LEFT_CENTER, fnum(u.show(Q::Induction, v)), egui::FontId::proportional(11.0), Color32::GRAY);
        }
        painter.text(egui::pos2(bx, by0 - 8.0), egui::Align2::LEFT_BOTTOM, format!("|B| [{}]", u.sym(Q::Induction)), egui::FontId::proportional(12.0), Color32::GRAY);
        // survol : lecture exacte
        if let Some(hp) = resp.hover_pos() {
            let th = ((hp.x - x0) / sc) as f64 / r0;
            let y = y_min + ((y0 - hp.y) / sc) as f64;
            if (0.0..2.0 * PI).contains(&th) {
                let f = r.model.solve(self.theta_r.to_radians());
                let d = 1e-7;
                let at = |yy: f64, tt: f64| sec.a_at(&f.secs[k], yy, tt);
                let zone = if y < ylo || y > yhi {
                    t("fer")
                } else if (y - g.y_s) * (y - g.y_sb) <= 0.0 {
                    t("encoche / dent")
                } else if (y - g.y_s) * (y - g.y_m) <= 0.0 {
                    t("entrefer")
                } else {
                    t("aimant")
                };
                let mut txt = format!("{zone}\nθ = {:.3}°  (x = {:.2} mm)\nz = {:.2} mm", th.to_degrees(), r0 * th * 1e3, (y - ylo) * 1e3);
                if let (Some(a1), Some(a2), Some(a3), Some(a4)) = (at(y, th + d / r0), at(y, th - d / r0), at(y + d, th), at(y - d, th)) {
                    let bz = (a1 - a2) / (2.0 * d);
                    let bt = -(a3 - a4) / (2.0 * d);
                    txt += &format!("\nBz = {} {s}\nBθ = {} {s}\n|B| = {} {s}", fnum(u.show(Q::Induction, bz)), fnum(u.show(Q::Induction, bt)), fnum(u.show(Q::Induction, bz.hypot(bt))), s = u.sym(Q::Induction));
                }
                painter.text(hp + Vec2::new(14.0, 14.0), egui::Align2::LEFT_TOP, txt, egui::FontId::monospace(12.0), ORANGE);
            }
        }
        painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), egui::Align2::LEFT_BOTTOM, format!("{} — r = {:.1} mm · {}", t("tranche développée"), r0 * 1e3, t("molette : zoom · glisser : parcourir le tour · double-clic : recentrer")), egui::FontId::proportional(11.0), Color32::GRAY);
    }

    /// Induction axiale d'entrefer sur toute la face du disque (couleur
    /// signée : pôles N et S), une couronne par tranche.
    fn disc_map_ui(&mut self, ui: &mut egui::Ui) {
        let fresh = matches!(&self.disc_cache, Some((a, _)) if (*a - self.theta_r).abs() < 1e-12);
        if !fresh {
            let r = self.res.as_ref().unwrap();
            let f = r.model.solve(self.theta_r.to_radians());
            self.disc_cache = Some((self.theta_r, analysis::disc_gap_map(&r.model, &f, 720)));
        }
        let lang = self.cfg.lang;
        let t = |kk: &'static str| T(lang, kk);
        let u = self.cfg.units.clone();
        let spec = self.res.as_ref().unwrap().spec.clone();
        let (_, rings) = self.disc_cache.as_ref().unwrap();
        let bmax = rings.iter().flat_map(|r| r.2.iter()).fold(0.0f64, |a, v| a.max(v.abs())).max(1e-6);
        let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), ui.available_height() - 4.0), egui::Sense::click_and_drag());
        pan_zoom(ui, &resp, &mut self.disc_view);
        let rect = resp.rect;
        painter.rect_filled(rect, 6.0, view3d::bg());
        let ro = spec.axial.r_out * 1e-3;
        let sc = (rect.width() - 110.0).min(rect.height()) * 0.46 / ro as f32 * self.disc_view.zoom;
        let c = rect.center() - Vec2::new(45.0, 0.0) + self.disc_view.pan;
        let to = |r: f64, th: f64| c + Vec2::new((r * th.cos()) as f32 * sc, -(r * th.sin()) as f32 * sc);
        let mut mesh = egui::epaint::Mesh::default();
        for (lo, hi, bz) in rings.iter() {
            let nt = bz.len();
            let dth = 2.0 * PI / nt as f64;
            for (j, b) in bz.iter().enumerate() {
                let col = view3d::colormap((0.5 + 0.5 * b / bmax) as f32);
                let th0 = j as f64 * dth - dth / 2.0;
                let p = [to(*lo, th0), to(*hi, th0), to(*hi, th0 + dth), to(*lo, th0 + dth)];
                let base = mesh.vertices.len() as u32;
                for v in p {
                    mesh.colored_vertex(v, col);
                }
                mesh.add_triangle(base, base + 1, base + 2);
                mesh.add_triangle(base, base + 2, base + 3);
            }
        }
        painter.add(egui::Shape::mesh(mesh));
        // limites des tranches
        for (lo, hi, _) in rings.iter() {
            painter.circle_stroke(c, *lo as f32 * sc, egui::Stroke::new(0.6f32, Color32::from_black_alpha(90)));
            painter.circle_stroke(c, *hi as f32 * sc, egui::Stroke::new(0.6f32, Color32::from_black_alpha(90)));
        }
        // barre de couleurs signée
        let bx = rect.right() - 80.0;
        let (by0, by1) = (rect.top() + 30.0, rect.bottom() - 40.0);
        for kk in 0..100 {
            let tt = kk as f32 / 100.0;
            let y = by1 - (by1 - by0) * tt;
            painter.rect_filled(egui::Rect::from_min_max(egui::pos2(bx, y - (by1 - by0) / 100.0 - 0.5), egui::pos2(bx + 16.0, y)), 0.0, view3d::colormap(tt));
        }
        for kk in 0..=4 {
            let v = -bmax + 2.0 * bmax * kk as f64 / 4.0;
            let y = by1 - (by1 - by0) * kk as f32 / 4.0;
            painter.text(egui::pos2(bx + 20.0, y), egui::Align2::LEFT_CENTER, fnum(u.show(Q::Induction, v)), egui::FontId::proportional(11.0), Color32::GRAY);
        }
        painter.text(egui::pos2(bx, by0 - 8.0), egui::Align2::LEFT_BOTTOM, format!("Bz [{}]", u.sym(Q::Induction)), egui::FontId::proportional(12.0), Color32::GRAY);
        if let Some(hp) = resp.hover_pos() {
            let v = (hp - c) / sc;
            let rr = (v.x * v.x + v.y * v.y).sqrt() as f64;
            let th = (-v.y as f64).atan2(v.x as f64).rem_euclid(2.0 * PI);
            if let Some((i, (_, _, bz))) = rings.iter().enumerate().find(|(_, (lo, hi, _))| rr >= *lo && rr <= *hi) {
                let j = ((th / (2.0 * PI)) * bz.len() as f64).round() as usize % bz.len();
                painter.text(hp + Vec2::new(14.0, 14.0), egui::Align2::LEFT_TOP, format!("{} {}\nr = {:.1} mm\nθ = {:.2}°\nBz = {} {}", t("Tranche"), i + 1, rr * 1e3, th.to_degrees(), fnum(u.show(Q::Induction, bz[j])), u.sym(Q::Induction)), egui::FontId::monospace(12.0), ORANGE);
            }
        }
        painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), egui::Align2::LEFT_BOTTOM, t("Induction axiale au milieu de l'entrefer, une couronne par tranche du modèle (rouge et bleu : pôles N et S)"), egui::FontId::proportional(11.0), Color32::GRAY);
    }
}

