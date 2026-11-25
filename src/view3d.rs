//! Rendu 3D logiciel (peintre) dans egui : caméra orbitale, surfaces
//! z = f(x, y) avec lecture exacte au survol, et vue 3D du moteur.

use eframe::egui::{self, epaint::Mesh, Color32, Pos2, Rect, Sense, Shape, Stroke, Vec2};
use std::sync::atomic::{AtomicBool, Ordering};

/// Thème courant (sombre par défaut) pour les fonds et textes des vues 3D.
pub static DARK: AtomicBool = AtomicBool::new(true);
/// Vrai si le thème sombre est actif.
pub fn is_dark() -> bool {
    DARK.load(Ordering::Relaxed)
}
/// Les toiles de dessin (moteur 3D, coupes et cartes animées, arbre de
/// l'I.A) gardent un fond sombre dans les deux thèmes : un fond blanc sous
/// des rendus lumineux fatigue les yeux, comme dans les logiciels de CAO.
pub fn canvas_dark() -> bool {
    true
}
/// Couleur de fond des toiles de dessin.
pub fn bg() -> Color32 {
    Color32::from_rgb(14, 18, 28)
}
fn fg() -> Color32 {
    Color32::from_rgb(170, 200, 230)
}
/// Texte d'aide affiché en bas des vues 3D (traduit par l'appelant).
pub static HINT: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());
fn hint() -> String {
    HINT.read().map(|s| s.clone()).unwrap_or_default()
}

#[derive(Clone, Copy)]
/// Caméra orbitale des vues 3D.
pub struct Camera {
    /// Rotation autour de l'axe vertical [rad].
    pub yaw: f32,
    /// Inclinaison [rad].
    pub pitch: f32,
    /// Facteur de zoom.
    pub zoom: f32,
    /// Décalage à l'écran [points].
    pub pan: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { yaw: -0.7, pitch: 0.55, zoom: 0.8, pan: Vec2::ZERO }
    }
}

impl Camera {
    /// Projette un point de la boîte normalisée [-1,1]³ ; renvoie (écran, profondeur).
    pub fn project(&self, p: [f32; 3], rect: Rect) -> (Pos2, f32) {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        // z vertical : rotation autour de z (yaw) puis inclinaison (pitch)
        let x1 = cy * p[0] - sy * p[1];
        let y1 = sy * p[0] + cy * p[1];
        let z1 = p[2];
        let y2 = cp * y1 - sp * z1;
        let z2 = sp * y1 + cp * z1;
        let d = 5.0;
        let f = d / (d + y2);
        let s = rect.width().min(rect.height()) * 0.36 * self.zoom;
        (rect.center() + self.pan + Vec2::new(x1 * f * s, -z2 * f * s), y2)
    }

    /// Interactions : glisser = orbite, clic droit/maj + glisser = déplacement, molette = zoom.
    pub fn interact(&mut self, ui: &egui::Ui, resp: &egui::Response) {
        if resp.dragged_by(egui::PointerButton::Primary) && !ui.input(|i| i.modifiers.shift) {
            let d = resp.drag_delta();
            self.yaw -= d.x * 0.008;
            self.pitch = (self.pitch + d.y * 0.008).clamp(-1.5, 1.5);
        }
        if resp.dragged_by(egui::PointerButton::Secondary)
            || (resp.dragged_by(egui::PointerButton::Primary) && ui.input(|i| i.modifiers.shift))
        {
            self.pan += resp.drag_delta();
        }
        if resp.hovered() {
            let s = ui.input(|i| i.smooth_scroll_delta.y);
            if s != 0.0 {
                self.zoom = (self.zoom * (1.0 + s * 0.0015)).clamp(0.2, 20.0);
            }
        }
        if resp.double_clicked() {
            *self = Camera::default();
        }
    }
}

/// Palette continue (bleu → cyan → blanc → orange → rouge), t ∈ [0,1].
pub fn colormap(t: f32) -> Color32 {
    let stops: [(f32, [f32; 3]); 6] = [
        (0.0, [0.05, 0.08, 0.35]),
        (0.25, [0.0, 0.55, 0.85]),
        (0.45, [0.35, 0.9, 0.95]),
        (0.6, [0.98, 0.92, 0.55]),
        (0.8, [1.0, 0.55, 0.1]),
        (1.0, [0.75, 0.05, 0.05]),
    ];
    let t = t.clamp(0.0, 1.0);
    for w in stops.windows(2) {
        if t <= w[1].0 {
            let u = (t - w[0].0) / (w[1].0 - w[0].0);
            let c: Vec<u8> = (0..3).map(|k| ((w[0].1[k] + u * (w[1].1[k] - w[0].1[k])) * 255.0) as u8).collect();
            return Color32::from_rgb(c[0], c[1], c[2]);
        }
    }
    Color32::WHITE
}

fn shade(c: Color32, k: f32) -> Color32 {
    let k = k.clamp(0.0, 1.3);
    // composantes non prémultipliées : une face transparente garde sa teinte
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied((r as f32 * k).min(255.0) as u8, (g as f32 * k).min(255.0) as u8, (b as f32 * k).min(255.0) as u8, a)
}

/// Surface z = f(x, y) sur une grille régulière.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Surface {
    /// Abscisses.
    pub xs: Vec<f64>,
    /// Ordonnées.
    pub ys: Vec<f64>,
    /// z[iy * nx + ix]
    pub z: Vec<f64>,
    /// Légende de l'axe x.
    pub x_label: String,
    /// Légende de l'axe y.
    pub y_label: String,
    /// Légende de l'axe z.
    pub z_label: String,
}

/// Dessine une surface 3D interactive ; renvoie le point survolé (x, y, z).
pub fn surface_ui(ui: &mut egui::Ui, cam: &mut Camera, s: &Surface, wire: bool) -> Option<(f64, f64, f64)> {
    let size = ui.available_size().max(Vec2::new(200.0, 200.0));
    let (resp, painter) = ui.allocate_painter(size, Sense::click_and_drag());
    cam.interact(ui, &resp);
    let rect = resp.rect;
    painter.rect_filled(rect, 4.0, bg());
    let (nx, ny) = (s.xs.len(), s.ys.len());
    if nx < 2 || ny < 2 {
        return None;
    }
    let (x0, x1) = (s.xs[0], s.xs[nx - 1]);
    let (y0, y1) = (s.ys[0], s.ys[ny - 1]);
    let (z0, z1) = s.z.iter().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    let zr = if (z1 - z0).abs() < 1e-30 { 1.0 } else { z1 - z0 };
    let nrm = |ix: usize, iy: usize| -> [f32; 3] {
        [
            (2.0 * (s.xs[ix] - x0) / (x1 - x0) - 1.0) as f32,
            (2.0 * (s.ys[iy] - y0) / (y1 - y0) - 1.0) as f32,
            (1.4 * (s.z[iy * nx + ix] - z0) / zr - 0.7) as f32,
        ]
    };
    let mut pts = Vec::with_capacity(nx * ny);
    for iy in 0..ny {
        for ix in 0..nx {
            pts.push(cam.project(nrm(ix, iy), rect));
        }
    }
    // quads triés du plus loin au plus proche
    let mut quads: Vec<(f32, usize, usize)> = Vec::with_capacity((nx - 1) * (ny - 1));
    for iy in 0..ny - 1 {
        for ix in 0..nx - 1 {
            let d = pts[iy * nx + ix].1 + pts[iy * nx + ix + 1].1 + pts[(iy + 1) * nx + ix].1 + pts[(iy + 1) * nx + ix + 1].1;
            quads.push((d, ix, iy));
        }
    }
    quads.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    // boîte et axes
    draw_box(&painter, cam, rect);
    let mut mesh = Mesh::default();
    for &(_, ix, iy) in &quads {
        let ids = [iy * nx + ix, iy * nx + ix + 1, (iy + 1) * nx + ix + 1, (iy + 1) * nx + ix];
        let zm = ids.iter().map(|&k| s.z[k]).sum::<f64>() / 4.0;
        // ombrage selon la pente
        let p = |k: usize| nrm(k % nx, k / nx);
        let (a, b, c) = (p(ids[0]), p(ids[1]), p(ids[3]));
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
        let light = 0.55 + 0.45 * ((n[2] / nl).abs());
        let col = shade(colormap(((zm - z0) / zr) as f32), light);
        let base = mesh.vertices.len() as u32;
        for &k in &ids {
            mesh.colored_vertex(pts[k].0, col);
        }
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
        if wire {
            let st = Stroke::new(0.4f32, Color32::from_black_alpha(90));
            painter.add(Shape::closed_line(ids.iter().map(|&k| pts[k].0).collect(), st));
        }
        if mesh.vertices.len() > 60000 {
            painter.add(Shape::mesh(std::mem::take(&mut mesh)));
        }
    }
    painter.add(Shape::mesh(mesh));
    draw_axes_labels(&painter, cam, rect, s, (x0, x1), (y0, y1), (z0, z1));
    // lecture au survol : sommet projeté le plus proche
    let mut hit = None;
    if let Some(hp) = resp.hover_pos() {
        let mut best = (f32::MAX, 0);
        for (k, (p, _)) in pts.iter().enumerate() {
            let d = p.distance_sq(hp);
            if d < best.0 {
                best = (d, k);
            }
        }
        if best.0 < 400.0 {
            let k = best.1;
            let (ix, iy) = (k % nx, k / nx);
            painter.circle(pts[k].0, 4.5, Color32::WHITE, Stroke::new(1.5f32, Color32::BLACK));
            hit = Some((s.xs[ix], s.ys[iy], s.z[k]));
        }
    }
    painter.text(
        rect.left_bottom() + Vec2::new(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        hint(),
        egui::FontId::proportional(11.0),
        Color32::from_gray(150),
    );
    hit
}

fn draw_box(painter: &egui::Painter, cam: &Camera, rect: Rect) {
    let c = [-1.0f32, 1.0];
    let st = Stroke::new(0.6f32, if canvas_dark() { Color32::from_rgba_unmultiplied(120, 170, 220, 70) } else { Color32::from_rgba_unmultiplied(40, 80, 140, 90) });
    for &a in &c {
        for &b in &c {
            let segs = [
                ([-1.0, a, b * 0.7], [1.0, a, b * 0.7]),
                ([a, -1.0, b * 0.7], [a, 1.0, b * 0.7]),
                ([a, b, -0.7], [a, b, 0.7]),
            ];
            for (p, q) in segs {
                painter.line_segment([cam.project(p, rect).0, cam.project(q, rect).0], st);
            }
        }
    }
}

fn draw_axes_labels(
    painter: &egui::Painter,
    cam: &Camera,
    rect: Rect,
    s: &Surface,
    xr: (f64, f64),
    yr: (f64, f64),
    zr: (f64, f64),
) {
    let font = egui::FontId::proportional(11.0);
    let col = fg();
    for k in 0..=4 {
        let t = k as f32 / 4.0;
        let v = xr.0 + (xr.1 - xr.0) * t as f64;
        let (p, _) = cam.project([2.0 * t - 1.0, -1.08, -0.7], rect);
        painter.text(p, egui::Align2::CENTER_TOP, fmt_num(v), font.clone(), col);
        let v = yr.0 + (yr.1 - yr.0) * t as f64;
        let (p, _) = cam.project([1.08, 2.0 * t - 1.0, -0.7], rect);
        painter.text(p, egui::Align2::LEFT_CENTER, fmt_num(v), font.clone(), col);
        let v = zr.0 + (zr.1 - zr.0) * t as f64;
        let (p, _) = cam.project([-1.0, -1.0, 1.4 * t - 0.7], rect);
        painter.text(p, egui::Align2::RIGHT_CENTER, format!("{} ", fmt_num(v)), font.clone(), col);
    }
    let (p, _) = cam.project([0.0, -1.3, -0.7], rect);
    painter.text(p, egui::Align2::CENTER_TOP, &s.x_label, font.clone(), if canvas_dark() { Color32::from_rgb(90, 200, 255) } else { Color32::from_rgb(0, 110, 170) });
    let (p, _) = cam.project([1.35, 0.0, -0.7], rect);
    painter.text(p, egui::Align2::LEFT_CENTER, &s.y_label, font.clone(), if canvas_dark() { Color32::from_rgb(90, 200, 255) } else { Color32::from_rgb(0, 110, 170) });
    let (p, _) = cam.project([-1.0, -1.0, 0.95], rect);
    painter.text(p, egui::Align2::CENTER_BOTTOM, &s.z_label, font, Color32::from_rgb(255, 160, 60));
}

/// Formate un nombre avec un nombre de chiffres adapté à sa grandeur.
pub fn fmt_num(v: f64) -> String {
    let a = v.abs();
    if a < 1e-9 {
        "0".into()
    } else if a >= 1e4 || a < 1e-3 {
        format!("{v:.3e}")
    } else if a >= 100.0 {
        format!("{v:.1}")
    } else if a >= 1.0 {
        format!("{v:.3}")
    } else {
        format!("{v:.4}")
    }
}

/// Face 3D colorée (quadrilatère) pour la vue moteur.
pub struct Quad {
    /// Sommets (repère normalisé).
    pub p: [[f32; 3]; 4],
    /// Couleur de base.
    pub col: Color32,
    /// Identifiant de l'élément (pour le survol).
    pub tag: u32,
}

/// Dessine une liste de quadrilatères (repère normalisé), ombrage de Lambert.
/// Renvoie le `tag` de la face survolée la plus proche.
pub fn quads_ui(ui: &mut egui::Ui, cam: &mut Camera, quads: &[Quad], extra: impl FnOnce(&egui::Painter, &Camera, Rect)) -> Option<u32> {
    let size = ui.available_size().max(Vec2::new(200.0, 200.0));
    let (resp, painter) = ui.allocate_painter(size, Sense::click_and_drag());
    cam.interact(ui, &resp);
    let rect = resp.rect;
    painter.rect_filled(rect, 4.0, bg());
    let light = {
        let (sy, cy) = cam.yaw.sin_cos();
        [-0.3 * cy - 0.5 * sy, 0.3 * sy - 0.5 * cy, 0.8f32]
    };
    let mut proj: Vec<(f32, [Pos2; 4], Color32, u32)> = quads
        .iter()
        .map(|q| {
            let pp: Vec<(Pos2, f32)> = q.p.iter().map(|&v| cam.project(v, rect)).collect();
            let u = [q.p[1][0] - q.p[0][0], q.p[1][1] - q.p[0][1], q.p[1][2] - q.p[0][2]];
            let v = [q.p[3][0] - q.p[0][0], q.p[3][1] - q.p[0][1], q.p[3][2] - q.p[0][2]];
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
            let k = 0.45 + 0.6 * ((n[0] * light[0] + n[1] * light[1] + n[2] * light[2]) / nl).abs();
            let depth = pp.iter().map(|x| x.1).sum::<f32>() / 4.0;
            (depth, [pp[0].0, pp[1].0, pp[2].0, pp[3].0], shade(q.col, k), q.tag)
        })
        .collect();
    proj.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let mut mesh = Mesh::default();
    let mut hovered = None;
    let hp = resp.hover_pos();
    for (_, p, c, tag) in &proj {
        let base = mesh.vertices.len() as u32;
        for v in p {
            mesh.colored_vertex(*v, *c);
        }
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
        if let Some(h) = hp
            && point_in_quad(h, p) {
                hovered = Some(*tag); // le dernier dessiné (le plus proche) l'emporte
            }
    }
    painter.add(Shape::mesh(mesh));
    extra(&painter, cam, rect);
    hovered
}

fn point_in_quad(p: Pos2, q: &[Pos2; 4]) -> bool {
    let tri = |a: Pos2, b: Pos2, c: Pos2| {
        let s = |p1: Pos2, p2: Pos2, p3: Pos2| (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y);
        let (d1, d2, d3) = (s(p, a, b), s(p, b, c), s(p, c, a));
        let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
        let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
        !(neg && pos)
    };
    tri(q[0], q[1], q[2]) || tri(q[0], q[2], q[3])
}
