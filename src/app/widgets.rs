//! Aides graphiques : cartes, vignettes, courbes, secteurs, imprimantes.

use super::*;


// ----------------------------------------------------------------------
// Aides graphiques
// ----------------------------------------------------------------------

pub(super) fn badge(ok: bool) -> Color32 {
    if ok { Color32::from_rgb(70, 200, 110) } else { Color32::from_rgb(245, 85, 75) }
}

/// Formatage numérique de l'interface.
pub fn fnum(v: f64) -> String {
    view3d::fmt_num(v)
}

pub(super) fn chip(ui: &mut egui::Ui, k: &str, v: &str) {
    // passe à la ligne si la pastille entière ne tient pas
    let w = ui.fonts(|f| {
        f.layout_no_wrap(k.to_string(), egui::TextStyle::Small.resolve(ui.style()), Color32::WHITE).size().x
            + f.layout_no_wrap(v.to_string(), egui::TextStyle::Monospace.resolve(ui.style()), Color32::WHITE).size().x
    }) + 34.0;
    if w > ui.available_size_before_wrap().x && ui.available_size_before_wrap().x < ui.max_rect().width() - 1.0 {
        ui.end_row();
    }
    egui::Frame::none()
        .fill(ui.visuals().faint_bg_color)
        .rounding(12.0)
        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
        .show(ui, |ui| {
            // jamais de retour à la ligne : une pastille trop large passe à la ligne suivante entière
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(RichText::new(k).small().weak()).wrap(false));
                ui.add(egui::Label::new(RichText::new(v).strong().monospace()).wrap(false));
            });
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn card(ui: &mut egui::Ui, w: f32, title: &str, val: &str, ok: Option<bool>, tip: &str, theme: Theme, compact: bool) -> egui::Response {
    let accent = match ok {
        Some(true) => badge(true),
        Some(false) => badge(false),
        None => if theme == Theme::Sombre { CYAN } else { Color32::from_rgb(0, 120, 180) },
    };
    egui::Frame::none()
        .fill(ui.visuals().faint_bg_color)
        .rounding(10.0)
        .stroke(egui::Stroke::new(1.0f32, accent.gamma_multiply(0.6)))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_width(w - 22.0);
            ui.set_max_width(w - 22.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).weak());
                    if let Some(o) = ok {
                        ui.label(RichText::new(if o { "✔" } else { "✖" }).color(accent).strong());
                    }
                });
                ui.label(RichText::new(val).size(if compact { 16.0 } else { 21.0 }).strong().color(accent));
                if !compact {
                    ui.add(egui::Label::new(RichText::new(tip).small().weak()).wrap(true));
                }
            });
        })
        .response
        .interact(egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!("{tip}\n\n{}", tr("Cliquer : ouvrir en plein écran")))
}

/// Pulsation douce 0..1 pour signaler un résultat neuf.
pub(super) fn pulse(ui: &egui::Ui) -> f32 {
    let t = ui.input(|i| i.time) as f32;
    0.5 + 0.5 * (t * 5.0).sin()
}

/// Vignette d'aperçu : titre + mini-courbe (ou barres) ; cadre pulsant si neuf.
pub(super) fn vignette(ui: &mut egui::Ui, title: &str, data: &[f64], bars: bool, is_new: bool, pulse: f32) -> egui::Response {
    let size = Vec2::new(168.0, 96.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let p = ui.painter_at(rect);
    let dark = view3d::is_dark();
    let base = if dark { Color32::from_rgb(20, 28, 46) } else { Color32::from_rgb(236, 242, 250) };
    p.rect_filled(rect, 10.0, base);
    let accent = if is_new { ORANGE.gamma_multiply(0.3 + 0.7 * pulse) } else if resp.hovered() { CYAN } else { CYAN.gamma_multiply(0.35) };
    p.rect_stroke(rect, 10.0, egui::Stroke::new(if is_new { 2.5f32 } else { 1.2f32 }, accent));
    p.text(rect.left_top() + Vec2::new(8.0, 6.0), egui::Align2::LEFT_TOP, title, egui::FontId::proportional(12.5), if dark { Color32::from_gray(220) } else { Color32::from_gray(40) });
    if is_new {
        p.text(rect.right_top() + Vec2::new(-8.0, 6.0), egui::Align2::RIGHT_TOP, tr("nouveau"), egui::FontId::proportional(10.0), ORANGE.gamma_multiply(0.4 + 0.6 * pulse));
    }
    let plot = egui::Rect::from_min_max(rect.left_top() + Vec2::new(8.0, 26.0), rect.right_bottom() - Vec2::new(8.0, 8.0));
    if data.len() >= 2 {
        let (mn, mx) = data.iter().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        let span = (mx - mn).max(1e-30);
        let y = |v: f64| plot.bottom() - ((v - mn) / span) as f32 * plot.height();
        if bars {
            let n = data.len();
            let bw = plot.width() / n as f32;
            let y0 = if mn < 0.0 && mx > 0.0 { y(0.0) } else { plot.bottom() };
            for (k, v) in data.iter().enumerate() {
                let x0 = plot.left() + k as f32 * bw + bw * 0.15;
                let r = egui::Rect::from_two_pos(egui::pos2(x0, y0), egui::pos2(x0 + bw * 0.7, y(*v)));
                p.rect_filled(r, 1.0, SERIES[k % 6]);
            }
        } else {
            let n = data.len();
            let pts: Vec<egui::Pos2> = data.iter().enumerate().map(|(k, v)| egui::pos2(plot.left() + plot.width() * k as f32 / (n - 1) as f32, y(*v))).collect();
            p.add(egui::Shape::line(pts, egui::Stroke::new(1.4f32, CYAN)));
        }
    } else {
        p.text(plot.center(), egui::Align2::CENTER_CENTER, tr("à calculer"), egui::FontId::proportional(11.0), Color32::GRAY);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tr("Cliquer : ouvrir en plein écran"))
}

pub(super) fn interp(pts: &[[f64; 2]], x: f64) -> Option<f64> {
    if pts.len() < 2 {
        return None;
    }
    let k = pts.partition_point(|p| p[0] < x);
    if k == 0 {
        return if (pts[0][0] - x).abs() < 1e-12 { Some(pts[0][1]) } else { None };
    }
    if k >= pts.len() {
        return None;
    }
    let (a, b) = (pts[k - 1], pts[k]);
    Some(a[1] + (b[1] - a[1]) * (x - a[0]) / (b[0] - a[0]))
}

/// Courbes avec infobulle précise, curseurs A (clic gauche) / B (clic droit)
/// et tableau de mesures (valeurs, écarts, statistiques).
pub(super) fn curve_plot(ui: &mut egui::Ui, id: &str, series: &[(String, Vec<[f64; 2]>, Color32)], xl: &str, yl: &str, h: f32, cur: &mut Cursors) {
    let table_h = 26.0 + 20.0 * series.len() as f32;
    let resp = Plot::new(id)
        .height((h - table_h).max(120.0))
        .legend(Legend::default().position(Corner::RightTop))
        .x_axis_label(xl.to_string())
        .y_axis_label(yl.to_string())
        .label_formatter(|name, v| if name.is_empty() { format!("x = {:.5}\ny = {:.6}", v.x, v.y) } else { format!("{name}\nx = {:.5}\ny = {:.6}", v.x, v.y) })
        .show(ui, |p| {
            for (n, pts, c) in series {
                p.line(Line::new(PlotPoints::from(pts.clone())).name(n).color(*c).width(1.6f32));
            }
            if let Some(a) = cur.a {
                p.vline(VLine::new(a).color(Color32::from_rgb(90, 220, 120)).width(1.2f32).name("A"));
            }
            if let Some(b) = cur.b {
                p.vline(VLine::new(b).color(Color32::from_rgb(240, 90, 200)).width(1.2f32).name("B"));
            }
            p.pointer_coordinate()
        });
    if let Some(pc) = resp.inner {
        if resp.response.clicked() {
            cur.a = Some(pc.x);
        }
        if resp.response.secondary_clicked() {
            cur.b = Some(pc.x);
        }
    }
    egui::Grid::new(format!("{id}_meas")).striped(true).num_columns(9).spacing([10.0, 2.0]).show(ui, |ui| {
        let hdr = |ui: &mut egui::Ui, s: String| {
            ui.label(RichText::new(s).small().strong());
        };
        hdr(ui, "".into());
        hdr(ui, cur.a.map(|a| format!("A @ {a:.4}")).unwrap_or(tr("A : clic gauche").into()));
        hdr(ui, cur.b.map(|b| format!("B @ {b:.4}")).unwrap_or(tr("B : clic droit").into()));
        hdr(ui, match (cur.a, cur.b) { (Some(a), Some(b)) => format!("Δ (Δx = {:.4})", b - a), _ => "Δ".into() });
        hdr(ui, "min".into());
        hdr(ui, "max".into());
        hdr(ui, tr("crête-crête").into());
        hdr(ui, tr("moy.").into());
        hdr(ui, "RMS".into());
        ui.end_row();
        for (n, pts, c) in series {
            let ys: Vec<f64> = pts.iter().map(|p| p[1]).collect();
            let (mn, mx) = ys.iter().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
            let mean = ys.iter().sum::<f64>() / ys.len().max(1) as f64;
            let rms = (ys.iter().map(|v| v * v).sum::<f64>() / ys.len().max(1) as f64).sqrt();
            let ya = cur.a.and_then(|a| interp(pts, a));
            let yb = cur.b.and_then(|b| interp(pts, b));
            ui.label(RichText::new(n).color(*c).small().strong());
            let f = |v: Option<f64>| v.map(|v| format!("{v:.6}")).unwrap_or("—".into());
            ui.label(RichText::new(f(ya)).small().monospace());
            ui.label(RichText::new(f(yb)).small().monospace());
            ui.label(RichText::new(f(match (ya, yb) { (Some(a), Some(b)) => Some(b - a), _ => None })).small().monospace());
            for v in [mn, mx, mx - mn, mean, rms] {
                ui.label(RichText::new(format!("{v:.6}")).small().monospace());
            }
            ui.end_row();
        }
    });
}

/// Spectre en barres ; abscisse = rang × `order_scale` ; amplitude × `f`.
pub(super) fn spectrum_plot(ui: &mut egui::Ui, id: &str, sp: &[f64], order_scale: f64, f: f64, xl: &str, yl: &str, db: bool, col: Color32) {
    let mx = sp.iter().skip(1).cloned().fold(0.0, f64::max).max(1e-30);
    let keep = sp.len().min(400);
    let bars: Vec<Bar> = (1..keep)
        .filter(|&k| sp[k] > mx * 1e-7)
        .map(|k| {
            let v = if db { 20.0 * (sp[k] / mx).log10() + 140.0 } else { sp[k] * f };
            Bar::new(k as f64 * order_scale, v).width(order_scale * 0.7).fill(col)
        })
        .collect();
    let yl = if db { tr("dB (0 = 140 dB sous la dominante)").to_string() } else { yl.to_string() };
    Plot::new(id)
        .height(ui.available_height().max(120.0))
        .x_axis_label(xl.to_string())
        .y_axis_label(yl)
        .show(ui, |p| {
            p.bar_chart(BarChart::new(bars).element_formatter(Box::new(move |b, _| {
                if db { format!("{} {:.2}\n{:.2} dB", tr("ordre"), b.argument, b.value - 140.0) } else { format!("{} {:.2}\n{:.6}", tr("ordre"), b.argument, b.value) }
            })))
        });
}

pub(super) fn pan_zoom(ui: &egui::Ui, resp: &egui::Response, v: &mut View2d) {
    if resp.dragged() {
        v.pan += resp.drag_delta();
    }
    if resp.hovered() {
        let s = ui.input(|i| i.smooth_scroll_delta.y);
        if s != 0.0 {
            let old = v.zoom;
            v.zoom = (v.zoom * (1.0 + s * 0.0015)).clamp(0.3, 200.0);
            if let Some(h) = resp.hover_pos() {
                let c = resp.rect.center() + v.pan;
                v.pan += (h - c) * (1.0 - v.zoom / old);
            }
        }
    }
    if resp.double_clicked() {
        *v = View2d::default();
    }
}

/// Stator : culasse, corps des dents et becs (si présents).
pub(super) fn stator_mesh(mesh: &mut egui::epaint::Mesh, to: &dyn Fn(f64, f64) -> egui::Pos2, rg: &geometry::Rings, q: usize, col: Color32) {
    sector(mesh, to, rg.yoke.0, rg.yoke.1, 0.0, 2.0 * PI, col, 180);
    for i in 0..q {
        let c0 = (i as f64 + 0.5) * 2.0 * PI / q as f64;
        let c1 = (i as f64 + 1.5) * 2.0 * PI / q as f64;
        sector(mesh, to, rg.sb, rg.tip, c0 + rg.beta_body / 2.0, c1 - rg.beta_body / 2.0, col, 24);
        if (rg.tip - rg.bore).abs() > 1e-9 {
            sector(mesh, to, rg.tip, rg.bore, c0 + rg.beta_open / 2.0, c1 - rg.beta_open / 2.0, col, 24);
        }
    }
}

/// Imprimantes installées (celle par défaut en premier).
pub(super) fn list_printers() -> Vec<String> {
    crate::platform::printers()
}

pub(super) fn sector(mesh: &mut egui::epaint::Mesh, to: &dyn Fn(f64, f64) -> egui::Pos2, r1: f64, r2: f64, a0: f64, a1: f64, col: Color32, n: usize) {
    for k in 0..n {
        let t0 = a0 + (a1 - a0) * k as f64 / n as f64;
        let t1 = a0 + (a1 - a0) * (k + 1) as f64 / n as f64;
        let base = mesh.vertices.len() as u32;
        for p in [to(r1, t0), to(r2, t0), to(r2, t1), to(r1, t1)] {
            mesh.colored_vertex(p, col);
        }
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn ring_sector(quads: &mut Vec<Quad>, r1: f64, r2: f64, a0: f64, a1: f64, z0: f32, z1: f32, col: Color32, n: usize, tag: u32) {
    let p = |r: f64, a: f64, z: f32| [(r * a.cos()) as f32, (r * a.sin()) as f32, z];
    for k in 0..n {
        let t0 = a0 + (a1 - a0) * k as f64 / n as f64;
        let t1 = a0 + (a1 - a0) * (k + 1) as f64 / n as f64;
        quads.push(Quad { p: [p(r1, t0, z1), p(r2, t0, z1), p(r2, t1, z1), p(r1, t1, z1)], col, tag });
        quads.push(Quad { p: [p(r1, t0, z0), p(r1, t1, z0), p(r2, t1, z0), p(r2, t0, z0)], col, tag });
        quads.push(Quad { p: [p(r2, t0, z0), p(r2, t1, z0), p(r2, t1, z1), p(r2, t0, z1)], col, tag });
        quads.push(Quad { p: [p(r1, t0, z0), p(r1, t0, z1), p(r1, t1, z1), p(r1, t1, z0)], col, tag });
    }
    quads.push(Quad { p: [p(r1, a0, z0), p(r2, a0, z0), p(r2, a0, z1), p(r1, a0, z1)], col, tag });
    quads.push(Quad { p: [p(r1, a1, z0), p(r1, a1, z1), p(r2, a1, z1), p(r2, a1, z0)], col, tag });
}

/// Annule le balourd au plus près (Newton moindre-norme) puis recentre.
pub(super) fn rebalance(a: &mut [f64]) {
    let n = a.len();
    let c = PI / 180.0;
    for _ in 0..12 {
        let bx: f64 = a.iter().map(|x| x.to_radians().cos()).sum();
        let by: f64 = a.iter().map(|x| x.to_radians().sin()).sum();
        if bx.hypot(by) < 1e-12 {
            break;
        }
        let j0: Vec<f64> = a.iter().map(|x| -x.to_radians().sin() * c).collect();
        let j1: Vec<f64> = a.iter().map(|x| x.to_radians().cos() * c).collect();
        let a11: f64 = j0.iter().map(|x| x * x).sum();
        let a22: f64 = j1.iter().map(|x| x * x).sum();
        let a12: f64 = j0.iter().zip(&j1).map(|(x, y)| x * y).sum();
        let det = a11 * a22 - a12 * a12;
        let y0 = (a22 * bx - a12 * by) / det;
        let y1 = (-a12 * bx + a11 * by) / det;
        for i in 0..n {
            a[i] -= j0[i] * y0 + j1[i] * y1;
        }
    }
}

/// Ajoute la bulle d'aide du réglage `text`, s'il en a une.
pub(super) fn with_tip(r: egui::Response, text: &str) -> egui::Response {
    match crate::tips::tip(crate::i18n::current(), text) {
        Some(h) => r.on_hover_text(h),
        None => r,
    }
}

static DEMO_FOCUS: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
static SEARCH: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
static SEARCH_FIRST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static SEARCH_HITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Texte recherché dans les paramètres (minuscules) ; remis à zéro du compteur à chaque image.
pub(super) fn set_search(q: &str) {
    if let Ok(mut s) = SEARCH.lock() {
        *s = q.trim().to_lowercase();
    }
    SEARCH_FIRST.store(true, std::sync::atomic::Ordering::Relaxed);
    SEARCH_HITS.store(0, std::sync::atomic::Ordering::Relaxed);
}

/// Nombre de paramètres trouvés à l'image précédente.
pub(super) fn search_hits() -> usize {
    SEARCH_HITS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Recherche en cours : sections dépliées.
pub(super) fn search_open() -> Option<bool> {
    SEARCH.lock().ok().filter(|s| !s.is_empty()).map(|_| true)
}
static PANEL_RIGHT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Bord droit du panneau des paramètres (limite du cadre rouge de la démo).
pub(super) fn set_panel_right(x: f32) {
    PANEL_RIGHT.store(x.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

/// Champ mis en évidence par la démo (libellé affiché), ou aucun.
pub(super) fn set_demo_focus(label: Option<String>) {
    if let Ok(mut f) = DEMO_FOCUS.lock() {
        *f = label;
    }
}

/// Libellé avec bulle d'aide au passage de la souris ; pendant la démo, la
/// ligne du champ en cours de saisie est encadrée en rouge.
pub(super) fn lbl(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let r = ui.label(text);
    let q = SEARCH.lock().map(|s| s.clone()).unwrap_or_default();
    if q.len() >= 2 && text.to_lowercase().contains(&q) {
        let right = f32::from_bits(PANEL_RIGHT.load(std::sync::atomic::Ordering::Relaxed));
        let right = if right > r.rect.right() { right } else { ui.clip_rect().right() };
        let row = egui::Rect::from_min_max(r.rect.min - Vec2::new(6.0, 4.0), egui::pos2(right - 6.0, r.rect.max.y + 4.0));
        let p = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("search_focus")));
        p.rect_stroke(row, 4.0, egui::Stroke::new(2.0_f32, Color32::from_rgb(240, 200, 40)));
        p.rect_filled(row, 4.0, Color32::from_rgba_unmultiplied(240, 200, 40, 26));
        SEARCH_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if SEARCH_FIRST.swap(false, std::sync::atomic::Ordering::Relaxed) {
            r.scroll_to_me(Some(egui::Align::Center));
        }
    }
    if DEMO_FOCUS.lock().ok().is_some_and(|f| f.as_deref() == Some(text)) {
        let right = f32::from_bits(PANEL_RIGHT.load(std::sync::atomic::Ordering::Relaxed));
        let right = if right > r.rect.right() { right } else { ui.clip_rect().right() };
        let row = egui::Rect::from_min_max(r.rect.min - Vec2::new(6.0, 4.0), egui::pos2(right - 6.0, r.rect.max.y + 4.0));
        let p = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("demo_focus")));
        p.rect_stroke(row, 4.0, egui::Stroke::new(2.5_f32, Color32::from_rgb(235, 40, 40)));
        p.rect_filled(row, 4.0, Color32::from_rgba_unmultiplied(235, 40, 40, 28));
        r.scroll_to_me(Some(egui::Align::Center));
    }
    with_tip(r, text)
}

/// Case à cocher avec bulle d'aide.
pub(super) fn chk(ui: &mut egui::Ui, v: &mut bool, text: &str) -> egui::Response {
    let r = ui.checkbox(v, text);
    with_tip(r, text)
}

/// Bouton radio avec bulle d'aide.
pub(super) fn rad<V: PartialEq>(ui: &mut egui::Ui, cur: &mut V, val: V, text: &str) -> egui::Response {
    let r = ui.radio_value(cur, val, text);
    with_tip(r, text)
}
