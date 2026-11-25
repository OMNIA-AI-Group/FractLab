//! Graphe 3D des possibilités explorées par le moteur I.A, à la manière d'un
//! graphe de notes : chaque essai est un nœud, relié au meilleur essai de son
//! départ au moment où il a été calculé. Le nuage se construit en direct et
//! tourne sur 360° (pendant et après le calcul).
//!
//! Placement : le plan horizontal porte les réglages libres (un réglage :
//! axe X ; deux : X et Y ; davantage : projection en étoile), la hauteur porte
//! le score (le meilleur en bas : on voit la « vallée » que l'I.A descend).
//! Couleurs : vert = nouveau meilleur, rouge = essai rejeté (contrainte
//! violée), violet = meilleure solution, gris = essai ordinaire.

use super::console::IaLog;
use super::*;
use crate::optimize::Trial;

/// État de la vue (rotation, zoom, sélection).
pub(super) struct GraphView {
    /// Rotation ajoutée à la main [rad] (lacet, tangage).
    pub drag: (f32, f32),
    /// Zoom.
    pub zoom: f32,
    /// Rotation automatique.
    pub spin: bool,
    /// Angle de la rotation automatique [rad].
    pub yaw: f32,
    /// Essai sélectionné (numéro).
    pub selected: Option<usize>,
    /// Heure d'arrivée de chaque essai (pour l'animation d'apparition) [s].
    pub born: Vec<f64>,
}

impl Default for GraphView {
    fn default() -> Self {
        GraphView { drag: (0.0, 0.0), zoom: 1.0, spin: true, yaw: 0.0, selected: None, born: Vec::new() }
    }
}

struct Node<'a> {
    t: &'a Trial,
    p: [f32; 3],
    parent: Option<usize>,
}

/// Place les essais dans le cube [−1, 1]³.
fn layout(trials: &[&Trial]) -> Vec<[f32; 3]> {
    let d = trials.first().map(|t| t.cand.values.len()).unwrap_or(0);
    // bornes observées de chaque réglage
    let mut lo = vec![f64::MAX; d];
    let mut hi = vec![f64::MIN; d];
    for t in trials {
        for (k, v) in t.cand.values.iter().enumerate().take(d) {
            lo[k] = lo[k].min(*v);
            hi[k] = hi[k].max(*v);
        }
    }
    let norm = |k: usize, v: f64| if hi[k] - lo[k] > 1e-12 { (2.0 * (v - lo[k]) / (hi[k] - lo[k]) - 1.0) as f32 } else { 0.0 };
    // hauteur : score en échelle logarithmique (les pénalités montent haut)
    let smin = trials.iter().map(|t| t.cand.score).fold(f64::MAX, f64::min);
    let h: Vec<f64> = trials.iter().map(|t| (t.cand.score - smin + 1e-4).log10()).collect();
    let (hmin, hmax) = h.iter().fold((f64::MAX, f64::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
    trials
        .iter()
        .zip(&h)
        .map(|(t, hv)| {
            let x: Vec<f32> = (0..d).map(|k| norm(k, t.cand.values[k])).collect();
            let (px, pz) = match d {
                0 => (0.0, 0.0),
                1 => (x[0], 0.0),
                2 => (x[0], x[1]),
                _ => {
                    // projection en étoile : réglage k sur la direction 2πk/d
                    let mut a = (0.0f32, 0.0f32);
                    for (k, v) in x.iter().enumerate() {
                        let ang = std::f32::consts::TAU * k as f32 / d as f32;
                        a.0 += v * ang.cos();
                        a.1 += v * ang.sin();
                    }
                    (a.0 / (d as f32).sqrt(), a.1 / (d as f32).sqrt())
                }
            };
            let py = if hmax - hmin > 1e-12 { (2.0 * (hv - hmin) / (hmax - hmin) - 1.0) as f32 } else { 0.0 };
            [px, py, pz]
        })
        .collect()
}

impl App {
    /// Graphe 3D des essais ; renvoie l'essai à appliquer si l'utilisateur le demande.
    pub(super) fn ia_graph(&mut self, ui: &mut egui::Ui, height: f32) -> Option<usize> {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let dark = self.cfg.theme == Theme::Sombre;
        let now = ui.input(|i| i.time);
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        let trials: Vec<&Trial> = self.console.ia.iter().filter_map(|e| if let IaLog::Tr(t) = e { Some(t) } else { None }).collect();
        let g = &mut self.console.graph;
        while g.born.len() < trials.len() {
            g.born.push(now);
        }
        g.born.truncate(trials.len());
        let pos = layout(&trials);
        // parent : meilleur essai du même départ au moment du calcul
        let mut nodes: Vec<Node> = Vec::with_capacity(trials.len());
        let mut best_of_start: HashMap<usize, (usize, f64)> = HashMap::new();
        let mut prev_start: Option<usize> = None;
        for (i, tr) in trials.iter().enumerate() {
            let parent = match best_of_start.get(&tr.start) {
                Some((j, _)) => Some(*j),
                // premier essai d'un départ : relié au meilleur du départ précédent
                None => prev_start.and_then(|s| best_of_start.get(&s).map(|(j, _)| *j)),
            };
            nodes.push(Node { t: tr, p: pos[i], parent });
            let e = best_of_start.entry(tr.start).or_insert((i, f64::INFINITY));
            if tr.cand.score < e.1 {
                *e = (i, tr.cand.score);
            }
            prev_start = Some(tr.start);
        }
        let best = trials.iter().enumerate().filter(|(_, t)| t.cand.feasible).min_by(|a, b| a.1.cand.score.total_cmp(&b.1.cand.score)).map(|(i, _)| i);

        // barre d'outils
        let mut apply = None;
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("🕸 {}", t("Possibilités explorées"))).strong());
            ui.label(RichText::new(format!("· {} {}", trials.len(), t("nœuds"))).weak());
            ui.checkbox(&mut g.spin, t("Rotation"));
            if ui.small_button("⟲").on_hover_text(t("Vue initiale")).clicked() {
                g.drag = (0.0, 0.0);
                g.zoom = 1.0;
            }
            ui.label(RichText::new(t("glisser : tourner · molette : zoom · clic : détail")).small().weak());
        });
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, if dark { Color32::from_rgb(30, 30, 30) } else { Color32::from_rgb(245, 245, 248) });
        if resp.dragged() {
            let d = resp.drag_delta();
            g.drag.0 += d.x * 0.01;
            g.drag.1 = (g.drag.1 + d.y * 0.01).clamp(-1.2, 1.2);
        }
        if resp.hovered() {
            let s = ui.input(|i| i.raw_scroll_delta.y);
            if s != 0.0 {
                g.zoom = (g.zoom * (1.0 + s * 0.002)).clamp(0.3, 5.0);
            }
        }
        if g.spin {
            // un tour complet en 24 s, pendant et après le calcul
            g.yaw = (g.yaw + dt * std::f32::consts::TAU / 24.0) % std::f32::consts::TAU;
            ui.ctx().request_repaint();
        }
        let yaw = g.yaw + g.drag.0;
        let pitch = 0.35 + g.drag.1;
        let scale = rect.height().min(rect.width()) * 0.30 * g.zoom;
        let c = rect.center();
        let project = |p: [f32; 3]| -> (egui::Pos2, f32) {
            let (sy, cy) = yaw.sin_cos();
            let (sp, cp) = pitch.sin_cos();
            let x = p[0] * cy - p[2] * sy;
            let z = p[0] * sy + p[2] * cy;
            // hauteur : score bas en bas de l'écran
            let y = -p[1] * cp + z * sp;
            let zz = p[1] * sp + z * cp;
            let k = 5.0 / (5.0 + zz);
            (egui::pos2(c.x + x * scale * k, c.y + y * scale * k), zz)
        };
        let proj: Vec<(egui::Pos2, f32)> = nodes.iter().map(|n| project(n.p)).collect();
        // repère : cadre au sol (meilleurs scores) pour lire la rotation
        let grid_col = if dark { Color32::from_rgba_unmultiplied(120, 120, 140, 50) } else { Color32::from_rgba_unmultiplied(80, 80, 100, 50) };
        for k in 0..=4 {
            let v = -1.0 + 0.5 * k as f32;
            for (a, b) in [([v, -1.0, -1.0], [v, -1.0, 1.0]), ([-1.0, -1.0, v], [1.0, -1.0, v])] {
                painter.line_segment([project(a).0, project(b).0], egui::Stroke::new(1.0_f32, grid_col));
            }
        }
        // arêtes
        let edge = if dark { Color32::from_rgba_unmultiplied(170, 170, 170, 70) } else { Color32::from_rgba_unmultiplied(90, 90, 90, 80) };
        for (i, n) in nodes.iter().enumerate() {
            if let Some(j) = n.parent {
                let age = ((now - g.born[i]) as f32 / 0.6).clamp(0.0, 1.0);
                let a = proj[j].0;
                let b = a + (proj[i].0 - a) * age;
                let col = if n.t.new_best { Color32::from_rgba_unmultiplied(110, 230, 120, 140) } else { edge };
                painter.line_segment([a, b], egui::Stroke::new(if n.t.new_best { 1.6_f32 } else { 0.8 }, col));
            }
        }
        // nœuds, du plus lointain au plus proche
        let mut order: Vec<usize> = (0..nodes.len()).collect();
        order.sort_by(|a, b| proj[*b].1.total_cmp(&proj[*a].1));
        let hover = resp.hover_pos();
        let mut hovered: Option<usize> = None;
        let text_col = if dark { Color32::from_gray(225) } else { Color32::from_gray(40) };
        // étiquettes déjà posées (pas de chevauchement) ; la meilleure solution passe en premier
        let mut labels: Vec<egui::Rect> = Vec::new();
        if let Some(b) = best {
            labels.push(egui::Rect::from_center_size(proj[b].0 + egui::vec2(0.0, 14.0), egui::vec2(54.0, 14.0)));
        }
        for &i in &order {
            let n = &nodes[i];
            let age = ((now - g.born[i]) as f32 / 0.4).clamp(0.0, 1.0);
            let depth = 5.0 / (5.0 + proj[i].1);
            let mut r = (if n.t.new_best { 5.5 } else { 3.2 }) * depth * age.max(0.05);
            let col = if Some(i) == best {
                r *= 1.5;
                Color32::from_rgb(170, 140, 255)
            } else if n.t.new_best {
                Color32::from_rgb(110, 230, 120)
            } else if !n.t.violations.is_empty() {
                Color32::from_rgb(220, 90, 80)
            } else if dark {
                Color32::from_gray(175)
            } else {
                Color32::from_gray(110)
            };
            // fondu selon la profondeur
            let col = col.gamma_multiply((0.45 + 0.55 * depth.clamp(0.0, 1.0)).min(1.0));
            painter.circle_filled(proj[i].0, r, col);
            if g.selected == Some(n.t.n) {
                painter.circle_stroke(proj[i].0, r + 4.0, egui::Stroke::new(1.5_f32, ORANGE));
            }
            if let Some(h) = hover
                && h.distance(proj[i].0) < r + 4.0
            {
                hovered = Some(i);
            }
            // étiquettes : meilleure solution, nouveaux meilleurs, premiers essais des départs
            let first_of_start = i == 0 || nodes[i - 1].t.start != n.t.start;
            if Some(i) == best || n.t.new_best || first_of_start {
                let label = if Some(i) == best {
                    format!("★ #{}", n.t.n)
                } else if first_of_start {
                    format!("{} {}", t("départ"), n.t.start)
                } else {
                    format!("#{}", n.t.n)
                };
                let at = proj[i].0 + egui::vec2(0.0, r + 3.0);
                let w = 7.0 * label.chars().count() as f32;
                let bx = egui::Rect::from_min_size(at - egui::vec2(w / 2.0, 0.0), egui::vec2(w, 13.0));
                let free = Some(i) == best || !labels.iter().any(|o| o.intersects(bx));
                if free {
                    if Some(i) != best {
                        labels.push(bx);
                    }
                    painter.text(at, egui::Align2::CENTER_TOP, label, egui::FontId::proportional(11.0), text_col.gamma_multiply(age));
                }
            }
        }
        // légende des axes
        let vars = self.console.ia_vars.clone();
        let axes = match vars.len() {
            0 => String::new(),
            1 => format!("X : {} · {} : {}", t(vars[0].name()), t("hauteur"), t("score")),
            2 => format!("X : {} · Z : {} · {} : {}", t(vars[0].name()), t(vars[1].name()), t("hauteur"), t("score")),
            _ => format!("{} · {} : {}", t("plan : réglages en étoile"), t("hauteur"), t("score")),
        };
        painter.text(rect.left_bottom() + egui::vec2(8.0, -6.0), egui::Align2::LEFT_BOTTOM, axes, egui::FontId::proportional(11.0), text_col.gamma_multiply(0.7));
        if let Some(i) = hovered {
            let line = crate::script::trial_line(nodes[i].t, &vars);
            resp.clone().on_hover_text(RichText::new(line).monospace());
            if resp.clicked() {
                self.console.graph.selected = Some(nodes[i].t.n);
            }
        } else if resp.clicked() {
            self.console.graph.selected = None;
        }
        // détail de l'essai sélectionné
        if let Some(n) = self.console.graph.selected
            && let Some(tr) = trials.iter().find(|t| t.n == n)
        {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(crate::script::trial_line(tr, &vars)).monospace().small());
                if tr.cand.feasible && self.opt_base.is_some() && self.opt_job.is_none() && ui.button(t("Appliquer cet essai")).clicked() {
                    apply = Some(n);
                }
            });
        }
        apply
    }
}
