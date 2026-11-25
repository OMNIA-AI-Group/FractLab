//! Champ magnétique dans le moteur 3D : carte de |B|, lignes de flux et
//! vecteurs B, calculés à la position courante du rotor (ils suivent
//! l'animation). Avec la vue transparente, le champ se voit à l'intérieur de
//! la machine.
//!
//! - Flux radial : le champ est 2D (invariant le long de l'axe) ; il est
//!   dessiné dans le plan médian (vue transparente) ou sur la face avant.
//! - Flux axial : induction axiale Bz au milieu de chaque entrefer, une
//!   couronne par tranche ; les vecteurs sont dirigés le long de l'axe.

use super::*;
use crate::geometry::AxialLayer;
use crate::subdomain::Model;

/// Champ prêt à dessiner : faces colorées (triées avec le moteur) et
/// segments (lignes de flux, vecteurs), en repère normalisé.
#[derive(Default)]
pub(super) struct Field3d {
    /// Faces de la carte du champ.
    pub quads: Vec<Quad>,
    /// Segments : (début, fin, couleur, épaisseur).
    pub segs: Vec<([f32; 3], [f32; 3], Color32, f32)>,
    /// Valeur de pleine échelle de la carte [T] (légende).
    pub bmax: f64,
    /// Carte signée (Bz) plutôt que module |B|.
    pub signed: bool,
}

/// Étiquette des faces du champ (non déplacées par la vibration du rotor).
pub(super) const FIELD_TAG: u32 = 6000;

fn with_alpha(c: Color32, a: u8) -> Color32 {
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// Rend transparentes les faces du moteur (pas celles du champ). Avec
/// `neutral`, leurs couleurs sont désaturées (gris bleuté) : seul le champ
/// porte la couleur et reste lisible.
pub(super) fn make_transparent(quads: &mut [Quad], opacity: f32, neutral: bool) {
    let a = (opacity.clamp(0.05, 1.0) * 255.0) as u8;
    for q in quads.iter_mut().filter(|q| q.tag != FIELD_TAG) {
        let c = if neutral {
            let [r, g, b, _] = q.col.to_srgba_unmultiplied();
            let l = (0.3 * r as f32 + 0.59 * g as f32 + 0.11 * b as f32) as u16;
            // 80 % gris bleuté, 20 % couleur d'origine
            let mix = |x: u8, t: u16| ((x as u16 + 4 * t) / 5) as u8;
            Color32::from_rgb(mix(r, l * 9 / 10), mix(g, l * 95 / 100), mix(b, l.min(235) + 20))
        } else {
            q.col
        };
        q.col = with_alpha(c, a);
    }
}

/// Flèche 3D (corps + deux branches) dans le plan contenant `perp`.
fn arrow(segs: &mut Vec<([f32; 3], [f32; 3], Color32, f32)>, a: [f32; 3], b: [f32; 3], perp: [f32; 3], col: Color32) {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if l < 1e-6 {
        return;
    }
    segs.push((a, b, col, 1.6));
    let h = 0.3 * l;
    for s in [1.0f32, -1.0] {
        let tip = [b[0] - d[0] / l * h + s * perp[0] * h * 0.5, b[1] - d[1] / l * h + s * perp[1] * h * 0.5, b[2] - d[2] / l * h + s * perp[2] * h * 0.5];
        segs.push((b, tip, col, 1.6));
    }
}

/// Champ d'une machine radiale (repère : rayon extérieur `r_out` = 1).
pub(super) fn radial(model: &Model, f: &Field, r_out: f64, z: f32, map: bool, lines: bool, vectors: bool) -> Field3d {
    let s = 1.0 / r_out;
    let mut out = Field3d { bmax: 2.5, ..Default::default() };
    let pt = |r: f64, th: f64| [(r * s * th.cos()) as f32, (r * s * th.sin()) as f32, z];
    if map || lines {
        let radii = analysis::map_radii(model, 10, 5, 8);
        let fm = analysis::field_map(model, f, radii, 240);
        let (nr, nt) = (fm.nr, fm.nt);
        let dth = 2.0 * PI / nt as f64;
        // limites radiales des cellules : milieux entre rayons voisins
        let edge = |i: usize, up: bool| -> f64 {
            let r = fm.r[i];
            match (up, i) {
                (true, i) if i + 1 < nr => 0.5 * (r + fm.r[i + 1]),
                (true, _) => r + 0.5 * (r - fm.r[i - 1]),
                (false, 0) => r - 0.5 * (fm.r[1] - r),
                (false, i) => 0.5 * (r + fm.r[i - 1]),
            }
        };
        if map {
            for i in 0..nr {
                let (lo, hi) = (edge(i, false), edge(i, true));
                for j in 0..nt {
                    if let Some(b) = fm.b[i * nt + j] {
                        let col = with_alpha(view3d::colormap((b / out.bmax) as f32), 235);
                        let (t0, t1) = (fm.th[j] - dth / 2.0, fm.th[j] + dth / 2.0);
                        out.quads.push(Quad { p: [pt(lo, t0), pt(hi, t0), pt(hi, t1), pt(lo, t1)], col, tag: FIELD_TAG });
                    }
                }
            }
        }
        if lines {
            let (amin, amax) = fm.a.iter().flatten().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
            let col = Color32::from_rgba_unmultiplied(255, 255, 255, 200);
            let zl = z + 0.002;
            let ptl = |r: f64, th: f64| [(r * s * th.cos()) as f32, (r * s * th.sin()) as f32, zl];
            for lv in 1..16 {
                let level = amin + (amax - amin) * lv as f64 / 16.0;
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
                                let t = (level - a) / (b - a);
                                let (pa, pb) = (pos[e], pos[(e + 1) % 4]);
                                pts.push(ptl(pa.0 + t * (pb.0 - pa.0), pa.1 + t * (pb.1 - pa.1)));
                            }
                        }
                        if pts.len() >= 2 {
                            out.segs.push((pts[0], pts[1], col, 1.0));
                            if pts.len() == 4 {
                                out.segs.push((pts[2], pts[3], col, 1.0));
                            }
                        }
                    }
                }
            }
        }
    }
    if vectors {
        let rg = model.r_mid_gap();
        let n = 120;
        let (br, bt) = model.airgap_b(f, rg, n);
        let bm = br.iter().zip(&bt).map(|(a, b)| a.hypot(*b)).fold(1e-9f64, f64::max);
        let zv = z + 0.004;
        for k in 0..n {
            let th = 2.0 * PI * k as f64 / n as f64;
            let (c, sn) = (th.cos(), th.sin());
            // composantes cartésiennes de B (r, θ → x, y)
            let bx = br[k] * c - bt[k] * sn;
            let by = br[k] * sn + bt[k] * c;
            let len = 0.12 * (bx.hypot(by) / bm);
            let a = [(rg * s * c) as f32, (rg * s * sn) as f32, zv];
            let ux = bx / bx.hypot(by).max(1e-12);
            let uy = by / bx.hypot(by).max(1e-12);
            let b = [a[0] + (ux * len) as f32, a[1] + (uy * len) as f32, zv];
            let col = view3d::colormap((0.5 + 0.5 * br[k] / bm) as f32);
            arrow(&mut out.segs, a, b, [-uy as f32, ux as f32, 0.0], col);
        }
    }
    out
}

/// Champ d'une machine axiale (repère : rayon extérieur actif = 1, cotes en mm).
pub(super) fn axial(model: &Model, f: &Field, spec: &MotorSpec, map: bool, vectors: bool) -> Field3d {
    let ro = spec.axial.r_out;
    let s = 1.0 / ro;
    let rings = analysis::disc_gap_map(model, f, 180);
    let bmax = rings.iter().flat_map(|r| r.2.iter()).fold(1e-9f64, |a, v| a.max(v.abs()));
    let mut out = Field3d { bmax, signed: true, ..Default::default() };
    let gaps: Vec<(f64, f64)> = spec.axial_stack().into_iter().filter(|l| l.0 == AxialLayer::Airgap).map(|l| (l.1, l.2)).collect();
    for (gz0, gz1) in gaps {
        let zc = (0.5 * (gz0 + gz1) * s) as f32;
        // côté −z : le flux traverse l'entrefer en sens opposé (machine symétrique)
        let side = if gz0 < 0.0 { -1.0 } else { 1.0 };
        let pt = |r_mm: f64, th: f64, z: f32| [(r_mm * s * th.cos()) as f32, (r_mm * s * th.sin()) as f32, z];
        for (lo, hi, bz) in &rings {
            let (lo, hi) = (lo * 1e3, hi * 1e3);
            let nt = bz.len();
            let dth = 2.0 * PI / nt as f64;
            if map {
                for (j, b) in bz.iter().enumerate() {
                    let col = with_alpha(view3d::colormap((0.5 + 0.5 * b / bmax) as f32), 225);
                    let (t0, t1) = (j as f64 * dth - dth / 2.0, j as f64 * dth + dth / 2.0);
                    out.quads.push(Quad { p: [pt(lo, t0, zc), pt(hi, t0, zc), pt(hi, t1, zc), pt(lo, t1, zc)], col, tag: FIELD_TAG });
                }
            }
            if vectors {
                let rm = 0.5 * (lo + hi);
                // une flèche tous les 10° environ : lisible même avec plusieurs tranches
                let step = (nt / 36).max(1);
                for j in (0..nt).step_by(step) {
                    let th = j as f64 * dth;
                    let len = (0.16 * bz[j] / bmax * side) as f32;
                    let a = pt(rm, th, zc - len / 2.0);
                    let b = pt(rm, th, zc + len / 2.0);
                    let col = if bz[j] > 0.0 { Color32::from_rgb(240, 80, 70) } else { Color32::from_rgb(70, 140, 250) };
                    arrow(&mut out.segs, a, b, [-(th.sin() as f32), th.cos() as f32, 0.0], col);
                }
            }
        }
    }
    out
}

/// Couleur des commandes du champ 3D (violet), pour qu'on les repère.
pub(super) const FIELD3D_COL: Color32 = Color32::from_rgb(110, 60, 200);

impl App {
    /// Vrai si la vue transparente avec champ est active.
    pub(super) fn field3d_on(&self) -> bool {
        self.vis_transparent && (self.vis_field_map || self.vis_field_lines || self.vis_field_vec)
    }

    /// Un clic : ouvre le moteur 3D en vue transparente avec le champ adapté
    /// à la machine (carte + lignes en radial, carte + vecteurs Bz en axial).
    pub(super) fn enable_field3d(&mut self) {
        let axial = self.spec.topology == Topology::Axial;
        self.tab = Tab::Moteur3d;
        self.full = None;
        self.vis_transparent = true;
        self.vis_field_map = true;
        self.vis_field_lines = !axial;
        self.vis_field_vec = axial;
        if !self.cfg.seen_field3d {
            self.cfg.seen_field3d = true;
            self.cfg.save();
        }
    }

    /// Bouton « 👁 Champ 3D » (avec badge « NOUVEAU » clignotant tant que la
    /// fonction n'a pas été essayée).
    pub(super) fn field3d_button(&mut self, ui: &mut egui::Ui, big: bool) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let on = self.field3d_on();
        let label = if big {
            if on { format!("👁 {}", t("Champ visible à travers le moteur — désactiver")) } else { format!("👁 {}", t("Voir le champ magnétique à travers le moteur")) }
        } else {
            format!("👁 {}", t("Champ 3D"))
        };
        let txt = RichText::new(label).strong().color(Color32::WHITE).size(if big { 16.0 } else { 14.0 });
        let fill = if on { FIELD3D_COL.gamma_multiply(0.7) } else { FIELD3D_COL };
        let mut b = egui::Button::new(txt).fill(fill);
        if big {
            b = b.min_size(Vec2::new(0.0, 34.0));
        }
        let r = ui.add(b).on_hover_text(t("Vue 3D transparente avec le champ magnétique à l'intérieur de la machine : carte de l'induction, lignes de flux, vecteurs B. Le champ suit le rotor pendant l'animation."));
        if !self.cfg.seen_field3d {
            let p = pulse(ui);
            let c = r.rect.right_top() + Vec2::new(-2.0, 2.0);
            let g = ui.painter().layout_no_wrap(t("NOUVEAU").to_string(), egui::FontId::proportional(9.5), Color32::WHITE);
            let bx = egui::Rect::from_center_size(c, g.size() + Vec2::new(8.0, 3.0));
            ui.painter().rect_filled(bx, 6.0, ORANGE.gamma_multiply(0.45 + 0.55 * p));
            ui.painter().galley(bx.center() - g.size() / 2.0, g, Color32::WHITE);
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(60));
        }
        if r.clicked() {
            if on && big {
                self.vis_transparent = false;
            } else {
                self.enable_field3d();
            }
        }
    }
}

