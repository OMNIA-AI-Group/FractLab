//! Démo : une étude complète déroulée toute seule sur un moteur de
//! démonstration. Confirmation (le projet en cours est mis de côté), choix
//! du moteur, puis paramétrage champ par champ (saisie encadrée en rouge,
//! valeur tapée chiffre par chiffre), calcul, visite des onglets dans
//! l'ordre d'une étude, moteur 3D avec le champ, Ajustement I.A avec la
//! console et son graphe. « ÉCHAP — sortir de la DÉMO » clignote en
//! rouge en permanence ; Échap arrête tout.

use super::*;
use crate::geometry::Topology;

type Setter = fn(&mut MotorSpec, f64);

/// Une étape de la démo.
#[derive(Clone, Copy)]
enum Step {
    /// Légende affichée pendant `secs` secondes.
    Say(&'static str, f64),
    /// Saisie d'un champ : libellé (clé), modification, valeur, décimales.
    Field(&'static str, Setter, f64, usize),
    /// Choix du type de machine.
    Topo(Topology),
    /// Onglet affiché pendant `secs` secondes, avec sa légende.
    Tab(Tab, &'static str, f64),
    /// Lance le calcul et attend sa fin.
    Compute,
    /// Moteur 3D en vue transparente avec le champ.
    Field3d(f64),
    /// Ajustement I.A (console et graphe) puis attente de la fin.
    Ia,
    /// Fin.
    End,
}

/// Démo en cours.
pub(super) struct DemoRun {
    key: &'static str,
    steps: Vec<Step>,
    idx: usize,
    t0: f64,
    caption: String,
    typed: usize,
}

/// Fenêtres et état de la démo.
#[derive(Default)]
pub(super) struct DemoState {
    /// Fenêtre de confirmation et de choix du moteur.
    pub confirm: bool,
    /// Démo en cours.
    pub run: Option<DemoRun>,
    /// Projet mis de côté au lancement.
    pub backup: Option<MotorSpec>,
    /// Fenêtre de fin (revenir au projet ou garder le moteur de démo).
    pub ended: bool,
    /// Démo à lancer à la première image (option FRACTLAB_DEMO=drone|servo|roue|axial, pour les essais).
    pub pending: Option<&'static str>,
}

fn steps_for(key: &str) -> Vec<Step> {
    let s = crate::demo::spec(key);
    let mut v = vec![Step::Say("Nouvelle étude : on part de la topologie, puis on saisit les cotes du moteur.", 3.0), Step::Topo(s.topology)];
    v.push(Step::Field("Encoches Q", |m, x| m.n_slots = x as usize, s.n_slots as f64, 0));
    v.push(Step::Field("Pôles 2p", |m, x| m.n_poles = x as usize, s.n_poles as f64, 0));
    match s.topology {
        Topology::RadialOuter => {
            v.push(Step::Field("Rayon d'alésage (ext. stator)", |m, x| m.stator_or = x, s.stator_or, 2));
            v.push(Step::Field("Rayon intérieur stator", |m, x| m.stator_ir = x, s.stator_ir, 2));
            v.push(Step::Field("Longueur active", |m, x| m.stack_len = x, s.stack_len, 2));
        }
        Topology::RadialInner => {
            v.push(Step::Field("Rayon d'alésage (int. stator)", |m, x| m.stator_or = x, s.stator_or, 2));
            v.push(Step::Field("Rayon extérieur du stator", |m, x| m.stator_ir = x, s.stator_ir, 2));
            v.push(Step::Field("Longueur active", |m, x| m.stack_len = x, s.stack_len, 2));
        }
        Topology::Axial => {
            v.push(Step::Field("Rayon intérieur actif", |m, x| m.axial.r_in = x, s.axial.r_in, 2));
            v.push(Step::Field("Rayon extérieur actif", |m, x| m.axial.r_out = x, s.axial.r_out, 2));
        }
    }
    v.push(Step::Field("Ouverture d'encoche", |m, x| m.slot_open = x, s.slot_open, 2));
    if s.tip_h > 0.0 {
        v.push(Step::Field("Becs de dents (0 = encoches ouvertes)", |m, x| m.tip_h = x, s.tip_h, 2));
        v.push(Step::Field("Largeur du corps d'encoche", |m, x| m.slot_w = x, s.slot_w, 2));
    }
    v.push(Step::Say("Entrefer, aimants et rotor.", 1.5));
    v.push(Step::Field("Entrefer", |m, x| m.airgap = x, s.airgap, 2));
    v.push(Step::Field("Épaisseur aimant", |m, x| m.magnet_th = x, s.magnet_th, 2));
    v.push(Step::Field("Culasse rotor", |m, x| m.rotor_yoke_th = x, s.rotor_yoke_th, 2));
    let arc = if s.topology == Topology::Axial { "Arc aimant / pas polaire (r int.)" } else { "Arc aimant / pas polaire" };
    v.push(Step::Field(arc, |m, x| m.magnet_arc_ratio = x, s.magnet_arc_ratio, 3));
    v.push(Step::Field("Induction rémanente Br", |m, x| m.br_magnet = x, s.br_magnet, 3));
    v.push(Step::Say("Bobinage et fonctionnement en charge.", 1.5));
    v.push(Step::Field("Spires par bobine", |m, x| m.turns_per_coil = x, s.turns_per_coil, 0));
    v.push(Step::Field("Vitesse", |m, x| m.rpm = x, s.rpm, 0));
    v.push(Step::Field("Courant de phase crête", |m, x| m.load.i_peak = x, s.load.i_peak, 1));
    v.push(Step::Compute);
    v.extend([
        Step::Tab(Tab::Synthese, "Synthèse : chaque carte donne une grandeur et son seuil ; un critère non tenu s'affiche en rouge.", 5.0),
        Step::Tab(Tab::Geometrie, "Géométrie : la coupe du moteur, à l'échelle.", 4.0),
        Step::Tab(Tab::Champ, "Champ : induction dans l'entrefer et carte du champ avec les lignes de flux.", 5.0),
        Step::Tab(Tab::Cogging, "Couple de détente (cogging) : le couple ressenti en tournant le moteur à la main.", 5.0),
        Step::Tab(Tab::Fem, "FEM : la tension produite quand le rotor tourne, phase par phase.", 4.5),
        Step::Tab(Tab::Forces, "Forces : la force magnétique sur le rotor au cours d'un tour.", 4.0),
        Step::Tab(Tab::Charge, "Charge : couple, ondulation, inductances, rendement et saturation sous courant.", 5.0),
        Step::Field3d(8.0),
        Step::Ia,
        Step::End,
    ]);
    v
}

impl App {
    /// Lance la démo sur le moteur `key` (le projet en cours est mis de côté).
    pub(super) fn demo_start(&mut self, key: &'static str, ctx: &egui::Context) {
        if self.demo.backup.is_none() {
            self.demo.backup = Some(self.spec.clone());
        }
        if let Some(j) = &mut self.opt_job {
            j.cancel();
        }
        self.opt_job = None;
        // point de départ : le moteur de démonstration (géométrie valide), dont
        // les cotes principales sont ensuite saisies une à une
        self.spec = crate::demo::spec(key);
        self.touched(ctx);
        self.tab = Tab::Synthese;
        self.full = None;
        self.show_params = true;
        self.console.open = false;
        self.show_opt = false;
        self.demo.ended = false;
        self.demo.run = Some(DemoRun { key, steps: steps_for(key), idx: 0, t0: ctx.input(|i| i.time), caption: String::new(), typed: 0 });
    }

    /// Arrête la démo (Échap) : propose de revenir au projet mis de côté.
    pub(super) fn demo_stop(&mut self) {
        if self.demo.run.take().is_some() {
            if let Some(j) = &mut self.opt_job {
                j.cancel();
            }
            self.opt_job = None;
            self.demo.ended = true;
        }
        widgets::set_demo_focus(None);
    }

    /// Avance la démo d'une image.
    pub(super) fn demo_tick(&mut self, ctx: &egui::Context) {
        if let Some(k) = self.demo.pending.take() {
            self.demo_start(k, ctx);
        }
        if self.demo.run.is_none() {
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.demo_stop();
            return;
        }
        ctx.request_repaint();
        let now = ctx.input(|i| i.time);
        let lang = self.cfg.lang;
        let Some(run) = self.demo.run.as_mut() else { return };
        let Some(step) = run.steps.get(run.idx).copied() else {
            self.demo_stop();
            return;
        };
        let dt = now - run.t0;
        let mut next = false;
        match step {
            Step::Say(k, secs) => {
                run.caption = T(lang, k).to_string();
                widgets::set_demo_focus(None);
                next = dt >= secs;
            }
            Step::Topo(tp) => {
                widgets::set_demo_focus(Some(T(lang, "Type de machine").to_string()));
                run.caption = format!("{} : {}", T(lang, "Type de machine"), T(lang, match tp {
                    Topology::RadialOuter => "Flux radial, rotor extérieur",
                    Topology::RadialInner => "Flux radial, rotor intérieur",
                    Topology::Axial => "Flux axial (multi-tranches)",
                }));
                next = dt >= 2.2;
            }
            Step::Field(label, set, value, dec) => {
                let txt = format!("{value:.dec$}");
                let n = ((dt - 0.4) / 0.14).floor().max(0.0) as usize;
                let n = n.min(txt.len());
                if n != run.typed {
                    run.typed = n;
                    if let Ok(x) = txt[..n].trim_end_matches('.').parse::<f64>()
                        && x > 0.0
                    {
                        set(&mut self.spec, x);
                    }
                }
                widgets::set_demo_focus(Some(T(lang, label).to_string()));
                let run = self.demo.run.as_mut().unwrap();
                run.caption = format!("{} : {}▌", T(lang, label), &txt[..n]);
                if n == txt.len() && dt >= 0.4 + 0.14 * txt.len() as f64 + 0.7 {
                    set(&mut self.spec, value);
                    self.dirty = true;
                    next = true;
                }
            }
            Step::Compute => {
                widgets::set_demo_focus(None);
                run.caption = T(lang, "Calcul du moteur complet…").to_string();
                if dt < 0.1 {
                    self.launch();
                } else if self.job.is_none() && !self.dirty && dt > 1.5 {
                    next = true;
                }
            }
            Step::Tab(tab, k, secs) => {
                widgets::set_demo_focus(None);
                run.caption = T(lang, k).to_string();
                self.tab = tab;
                next = dt >= secs;
            }
            Step::Field3d(secs) => {
                run.caption = T(lang, "Moteur 3D : vue transparente, le champ magnétique suit le rotor.").to_string();
                if self.tab != Tab::Moteur3d {
                    self.enable_field3d();
                }
                next = dt >= secs;
            }
            Step::Ia => {
                run.caption = T(lang, "Ajustement I.A : chaque essai est un calcul complet ; le graphe montre les possibilités explorées.").to_string();
                if dt < 0.1 && self.opt_job.is_none() {
                    let vars = vec![crate::optimize::Var::ArcRatio, crate::optimize::Var::SlotOpen];
                    self.opt_base = Some(self.spec.clone());
                    self.opt_vars_run = vars.clone();
                    self.opt_live = false;
                    self.opt_top.clear();
                    self.opt_hist.clear();
                    self.console_ia_start(&vars);
                    self.opt_job = Some(crate::optimize::run(self.spec.clone(), crate::optimize::Setup { vars, goal: crate::optimize::Goal::MinCogging, budget: 60, emf_keep: 0.9, sat_limit: false, skew_min: 0.0 }));
                }
                next = dt > 2.0 && self.opt_job.is_none() && dt > 6.0;
                if dt > 90.0 {
                    next = true;
                }
            }
            Step::End => {
                run.caption = T(lang, "Fin de la démo. Échap pour revenir à votre projet.").to_string();
                next = dt >= 6.0;
            }
        }
        if next {
            let run = self.demo.run.as_mut().unwrap();
            run.idx += 1;
            run.t0 = now;
            run.typed = 0;
        }
    }

    /// Fenêtres et bandeaux de la démo.
    pub(super) fn demo_windows(&mut self, ctx: &egui::Context) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        if self.demo.confirm {
            let mut open = true;
            let mut start: Option<&'static str> = None;
            egui::Window::new(format!("▶ {}", t("Démo"))).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).open(&mut open).show(ctx, |ui| {
                ui.label(RichText::new(t("La démo remplace le projet en cours par un moteur de démonstration.")).color(Color32::from_rgb(235, 70, 60)).strong());
                ui.label(RichText::new(t("Votre projet est mis de côté : à la fin de la démo, un bouton permet d'y revenir. Pour le garder dans un fichier, enregistrez-le d'abord.")).weak());
                if ui.button(format!("💾 {}", t("Enregistrer d'abord"))).clicked() {
                    self.save_project_dialog();
                }
                ui.separator();
                ui.label(RichText::new(t("Choisissez le moteur de la démo :")).strong());
                for m in crate::demo::MOTORS {
                    let r = ui.add(egui::Button::new(RichText::new(format!("{}  {}", m.icon, t(m.name))).size(16.0).strong()).min_size(Vec2::new(380.0, 34.0)));
                    ui.label(RichText::new(t(m.desc)).small().weak());
                    if r.clicked() {
                        start = Some(m.key);
                    }
                }
                ui.add_space(4.0);
                ui.label(RichText::new(t("Pendant la démo, Échap l'arrête à tout moment.")).small());
            });
            if let Some(k) = start {
                self.demo.confirm = false;
                self.demo_start(k, ctx);
            }
            if !open {
                self.demo.confirm = false;
            }
        }
        if let Some(run) = &self.demo.run {
            // bandeau rouge clignotant
            let time = ctx.input(|i| i.time);
            let a = (0.55 + 0.45 * (time * 5.0).sin()) as f32;
            egui::Area::new(egui::Id::new("demo_esc")).order(egui::Order::Foreground).anchor(egui::Align2::CENTER_TOP, [0.0, -23.0]).interactable(false).show(ctx, |ui| {
                egui::Frame::none().fill(Color32::from_rgb(200, 20, 20).gamma_multiply(a)).rounding(6.0).inner_margin(egui::Margin::symmetric(14.0, 1.0)).show(ui, |ui| {
                    ui.label(RichText::new(t("ÉCHAP — sortir de la DÉMO")).color(Color32::WHITE).strong().size(15.0));
                });
            });
            // légende
            let name = crate::demo::MOTORS.iter().find(|m| m.key == run.key).map(|m| t(m.name)).unwrap_or_default();
            let caption = run.caption.clone();
            let prog = (run.idx as f32 / run.steps.len().max(1) as f32).min(1.0);
            egui::Area::new(egui::Id::new("demo_caption")).order(egui::Order::Foreground).anchor(egui::Align2::CENTER_BOTTOM, [0.0, -36.0]).interactable(false).show(ctx, |ui| {
                egui::Frame::none().fill(Color32::from_rgba_unmultiplied(15, 15, 25, 235)).stroke(egui::Stroke::new(1.5_f32, Color32::from_rgb(220, 40, 40))).rounding(8.0).inner_margin(egui::Margin::symmetric(18.0, 10.0)).show(ui, |ui| {
                    ui.set_max_width(760.0);
                    ui.label(RichText::new(format!("▶ {} — {name}", t("DÉMO"))).color(Color32::from_rgb(255, 90, 80)).strong());
                    ui.label(RichText::new(caption).color(Color32::WHITE).size(17.0));
                    ui.add(egui::ProgressBar::new(prog).desired_width(720.0).desired_height(4.0));
                });
            });
        }
        if self.demo.ended {
            let mut close = false;
            egui::Window::new(t("Démo terminée")).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.demo.backup.is_some() && ui.button(format!("↺ {}", t("Revenir à mon projet"))).clicked() {
                        if let Some(b) = self.demo.backup.take() {
                            self.spec = b;
                            self.touched(ctx);
                        }
                        close = true;
                    }
                    if ui.button(t("Garder le moteur de démo")).clicked() {
                        self.demo.backup = None;
                        close = true;
                    }
                });
            });
            if close {
                self.demo.ended = false;
            }
        }
    }
}
