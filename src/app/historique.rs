//! Gestion du projet : annuler / rétablir, projet modifié depuis
//! l'enregistrement, liste des modifications, projets récents.
//!
//! Chaque saisie (`App::touched`) est regroupée : quand l'utilisateur
//! s'arrête une demi-seconde, l'état précédent part dans la pile
//! d'annulation. Ctrl+Z annule, Ctrl+Maj+Z ou Ctrl+Y rétablit (sauf pendant
//! la saisie dans un champ de texte, qui garde son propre annuler).

use super::*;

/// Profondeur de la pile d'annulation.
const UNDO_MAX: usize = 100;
/// Projets récents gardés.
const RECENT_MAX: usize = 8;

/// État de l'historique d'édition.
#[derive(Default)]
pub(super) struct EditHistory {
    undo: Vec<MotorSpec>,
    redo: Vec<MotorSpec>,
    /// dernier état enregistré dans l'historique
    committed: Option<MotorSpec>,
    /// une saisie attend d'être regroupée
    pending: bool,
    /// projet tel qu'enregistré ou ouvert (JSON), pour « modifié »
    saved: String,
    /// fenêtre « Modifications depuis l'enregistrement »
    pub show_changes: bool,
    /// titre de la fenêtre déjà envoyé au système
    title: String,
}

fn json(s: &MotorSpec) -> String {
    serde_json::to_string(s).unwrap_or_default()
}

/// Aplatit un JSON en (chemin, valeur).
fn flatten(v: &serde_json::Value, path: &str, out: &mut Vec<(String, String)>) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, x) in m {
                flatten(x, &if path.is_empty() { k.clone() } else { format!("{path}.{k}") }, out);
            }
        }
        serde_json::Value::Array(a) if a.len() > 8 => out.push((path.into(), format!("[{} valeurs]", a.len()))),
        serde_json::Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                flatten(x, &format!("{path}[{i}]"), out);
            }
        }
        x => out.push((path.into(), x.to_string())),
    }
}

/// Libellé affiché d'un champ du projet (clé de traduction), sinon le chemin brut.
fn field_label(path: &str) -> Option<&'static str> {
    Some(match path {
        "name" => "Nom",
        "n_slots" => "Encoches Q",
        "n_poles" => "Pôles 2p",
        "stator_or" => "Rayon d'alésage (ext. stator)",
        "stator_ir" => "Rayon intérieur stator",
        "stack_len" => "Longueur active",
        "airgap" => "Entrefer",
        "magnet_th" => "Épaisseur aimant",
        "rotor_yoke_th" => "Culasse rotor",
        "tooth_w" => "Largeur de dent",
        "slot_open" => "Ouverture d'encoche",
        "yoke_th" => "Culasse stator",
        "magnet_arc_ratio" => "Arc aimant / pas polaire",
        "br_magnet" => "Induction rémanente Br",
        "mur_magnet" => "Perméabilité de recul μr",
        "variant" => "Variante",
        "skew_max_deg" => "Amplitude max des décalages",
        "turns_per_coil" => "Spires par bobine",
        "rpm" => "Vitesse",
        "topology" => "Type de machine",
        "tip_h" => "Becs de dents (0 = encoches ouvertes)",
        "slot_w" => "Largeur du corps d'encoche",
        "slot_depth" => "Profondeur d'encoche (0 = auto)",
        "load.i_peak" => "Courant de phase crête",
        "load.angle_deg" => "Angle du courant / FEM",
        "axial.r_in" => "Rayon intérieur actif",
        "axial.r_out" => "Rayon extérieur actif",
        "axial.slices" => "Nombre de tranches",
        "axial.gaps" => "Nombre d'entrefers",
        _ => return None,
    })
}

impl App {
    /// Le projet a été ouvert ou enregistré : il devient la référence « non modifié »,
    /// et l'historique d'édition repart de lui.
    pub(super) fn hist_mark_saved(&mut self) {
        self.hist.saved = json(&self.spec);
        self.hist.committed = Some(self.spec.clone());
        self.hist.pending = false;
    }

    /// Projet ouvert depuis un fichier : nouvelle base, historique vidé.
    pub(super) fn hist_reset(&mut self) {
        self.hist.undo.clear();
        self.hist.redo.clear();
        self.hist_mark_saved();
    }

    /// Projet modifié depuis l'enregistrement ?
    pub(super) fn modified(&self) -> bool {
        json(&self.spec) != self.hist.saved
    }

    pub(super) fn can_undo(&self) -> bool {
        !self.hist.undo.is_empty() || (self.hist.pending && self.hist.committed.as_ref().is_some_and(|c| json(c) != json(&self.spec)))
    }

    pub(super) fn can_redo(&self) -> bool {
        !self.hist.redo.is_empty()
    }

    /// Une saisie a eu lieu (appelé par `touched`).
    pub(super) fn hist_touch(&mut self) {
        if self.hist.committed.is_none() {
            self.hist.committed = Some(self.spec.clone());
        }
        self.hist.pending = true;
    }

    /// Range la saisie en attente dans la pile d'annulation.
    fn hist_commit(&mut self) {
        self.hist.pending = false;
        let Some(prev) = self.hist.committed.take() else {
            self.hist.committed = Some(self.spec.clone());
            return;
        };
        if json(&prev) != json(&self.spec) {
            self.hist.undo.push(prev);
            if self.hist.undo.len() > UNDO_MAX {
                self.hist.undo.remove(0);
            }
            self.hist.redo.clear();
        }
        self.hist.committed = Some(self.spec.clone());
    }

    /// Relance le calcul sans créer d'entrée d'historique.
    fn hist_apply(&mut self, s: MotorSpec, ctx: &egui::Context) {
        self.spec = s;
        self.hist.committed = Some(self.spec.clone());
        self.hist.pending = false;
        self.dirty = true;
        self.edit_time = ctx.input(|i| i.time);
    }

    pub(super) fn undo(&mut self, ctx: &egui::Context) {
        if self.hist.pending {
            self.hist_commit();
        }
        if let Some(prev) = self.hist.undo.pop() {
            self.hist.redo.push(self.spec.clone());
            self.hist_apply(prev, ctx);
            self.status = self.t("Annulé").into();
        }
    }

    pub(super) fn redo(&mut self, ctx: &egui::Context) {
        if let Some(next) = self.hist.redo.pop() {
            self.hist.undo.push(self.spec.clone());
            self.hist_apply(next, ctx);
            self.status = self.t("Rétabli").into();
        }
    }

    /// Autotest (FRACTLAB_AUTOTEST=undo) : deux saisies, deux annulations, un
    /// rétablissement ; le résultat est écrit sur la sortie d'erreur.
    fn hist_autotest(&mut self, ctx: &egui::Context, now: f64) {
        // FRACTLAB_AUTOTEST=recent:<fichier.json> : ouverture par les projets récents
        if let Ok(v) = std::env::var("FRACTLAB_AUTOTEST")
            && let Some(path) = v.strip_prefix("recent:")
        {
            static FAIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if now > 1.0 && !FAIT.swap(true, std::sync::atomic::Ordering::Relaxed) {
                self.open_recent(path, ctx);
                let ok = |c: bool, m: &str| eprintln!("AUTOTEST {} : {m}", if c { "OK" } else { "ÉCHEC" });
                ok(self.spec.name == "Essai récent", &format!("projet ouvert (nom {})", self.spec.name));
                ok(!self.modified(), "projet ouvert = non modifié");
                ok(!self.can_undo(), "historique d'annulation vidé");
                ok(self.cfg.recent.first().map(String::as_str) == Some(path), "en tête des projets récents");
                eprintln!("AUTOTEST fin");
            }
            return;
        }
        if std::env::var("FRACTLAB_AUTOTEST").as_deref() != Ok("undo") {
            return;
        }
        static STEP: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
        static mut A0: f64 = 0.0;
        let st = STEP.load(std::sync::atomic::Ordering::Relaxed);
        let at = [1.0, 1.1, 2.0, 2.1, 3.0, 3.5, 4.0, 4.5];
        if (st as usize) >= at.len() || now < at[st as usize] {
            return;
        }
        let a = self.spec.airgap;
        let ok = |c: bool, m: &str| eprintln!("AUTOTEST {} : {m}", if c { "OK" } else { "ÉCHEC" });
        match st {
            0 => {
                unsafe { A0 = a };
                self.spec.airgap = 0.31;
                self.touched(ctx);
            }
            // deuxième saisie rapprochée : même groupe (une seule annulation)
            1 => {
                self.spec.airgap = 0.33;
                self.touched(ctx);
            }
            2 => {
                self.spec.airgap = 0.42;
                self.touched(ctx);
            }
            3 => ok(self.modified(), "projet signalé modifié"),
            4 => {
                self.undo(ctx);
                ok((self.spec.airgap - 0.33).abs() < 1e-12, &format!("1re annulation → 0,33 (obtenu {})", self.spec.airgap));
            }
            5 => {
                self.undo(ctx);
                ok((self.spec.airgap - unsafe { A0 }).abs() < 1e-12, &format!("2e annulation → valeur d'origine {} (obtenu {})", unsafe { A0 }, self.spec.airgap));
                ok(!self.modified(), "projet revenu à l'état enregistré");
            }
            6 => {
                self.redo(ctx);
                ok((self.spec.airgap - 0.33).abs() < 1e-12, &format!("rétablir → 0,33 (obtenu {})", self.spec.airgap));
            }
            _ => {
                ok(self.can_undo() && self.can_redo(), "piles annuler et rétablir non vides");
                eprintln!("AUTOTEST fin");
            }
        }
        STEP.store(st + 1, std::sync::atomic::Ordering::Relaxed);
    }

    /// À chaque image : regroupement des saisies, raccourcis, titre de la fenêtre.
    pub(super) fn hist_tick(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        self.hist_autotest(ctx, now);
        if std::env::var("FRACTLAB_AUTOTEST").is_ok() {
            ctx.request_repaint();
        }
        if self.hist.pending && now - self.edit_time > 0.5 {
            self.hist_commit();
        }
        // raccourcis, hors saisie dans un champ de texte
        if ctx.memory(|m| m.focused().is_none()) {
            let (z, sz, y) = ctx.input_mut(|i| {
                (
                    i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z),
                    i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::Z),
                    i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y),
                )
            });
            if sz || y {
                self.redo(ctx);
            } else if z {
                self.undo(ctx);
            }
        }
        let title = format!("FractLab — {}{}", self.spec.name, if self.modified() { " •" } else { "" });
        if title != self.hist.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.hist.title = title;
        }
    }

    /// Ajoute un fichier aux projets récents.
    pub(super) fn push_recent(&mut self, path: &str) {
        self.cfg.recent.retain(|p| p != path);
        self.cfg.recent.insert(0, path.to_string());
        self.cfg.recent.truncate(RECENT_MAX);
        self.cfg.save();
    }

    /// Ouvre un projet récent.
    pub(super) fn open_recent(&mut self, path: &str, ctx: &egui::Context) {
        match std::fs::read(path).ok().and_then(|b| serde_json::from_slice::<MotorSpec>(&b).ok()) {
            Some(s) => {
                self.spec = s;
                self.cfg.last_project = Some(path.to_string());
                self.push_recent(path);
                self.hist_reset();
                self.dirty = true;
                self.edit_time = ctx.input(|i| i.time);
                self.status = format!("{} {path}", self.t("Ouvert :"));
            }
            None => {
                self.cfg.recent.retain(|p| p != path);
                self.cfg.save();
                self.err = Some(format!("{} : {path}", self.t("Fichier de projet illisible")));
            }
        }
    }

    /// Fenêtre « Modifications depuis l'enregistrement ».
    pub(super) fn changes_window(&mut self, ctx: &egui::Context) {
        if !self.hist.show_changes {
            return;
        }
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let mut a = Vec::new();
        let mut b = Vec::new();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.hist.saved) {
            flatten(&v, "", &mut a);
        }
        if let Ok(v) = serde_json::to_value(&self.spec) {
            flatten(&v, "", &mut b);
        }
        let old: HashMap<String, String> = a.into_iter().collect();
        let mut rows: Vec<(String, String, String)> = b.into_iter().filter(|(k, v)| old.get(k) != Some(v)).map(|(k, v)| {
            let o = old.get(&k).cloned().unwrap_or_else(|| "—".into());
            (k, o, v)
        }).collect();
        rows.sort();
        let mut open = true;
        egui::Window::new(format!("✎ {}", t("Modifications depuis l'enregistrement"))).open(&mut open).default_size([620.0, 420.0]).show(ctx, |ui| {
            if rows.is_empty() {
                ui.label(t("Aucune modification : le projet est identique au fichier enregistré."));
                return;
            }
            ui.label(RichText::new(format!("{} {}", rows.len(), t("champ(s) modifié(s)"))).strong());
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("changes").striped(true).num_columns(3).spacing([18.0, 3.0]).show(ui, |ui| {
                    for h in [t("Grandeur"), t("Enregistré"), t("Actuel")] {
                        ui.label(RichText::new(h).strong());
                    }
                    ui.end_row();
                    for (k, o, v) in &rows {
                        match field_label(k) {
                            Some(l) => ui.label(t(l)).on_hover_text(k.as_str()),
                            None => ui.label(RichText::new(k.as_str()).monospace()),
                        };
                        ui.label(RichText::new(o).monospace().weak());
                        ui.label(RichText::new(v).monospace().strong());
                        ui.end_row();
                    }
                });
            });
        });
        self.hist.show_changes = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// La liste des modifications retrouve les champs changés, imbriqués compris.
    #[test]
    fn modifications_detectees() {
        let a = MotorSpec::default();
        let mut b = a.clone();
        b.airgap = 0.42;
        b.load.i_peak = 12.0;
        let (mut fa, mut fb) = (Vec::new(), Vec::new());
        flatten(&serde_json::to_value(&a).unwrap(), "", &mut fa);
        flatten(&serde_json::to_value(&b).unwrap(), "", &mut fb);
        let old: HashMap<String, String> = fa.into_iter().collect();
        let diff: Vec<String> = fb.into_iter().filter(|(k, v)| old.get(k) != Some(v)).map(|(k, _)| k).collect();
        assert_eq!(diff, vec!["airgap".to_string(), "load.i_peak".to_string()]);
        assert_eq!(field_label("load.i_peak"), Some("Courant de phase crête"));
    }
}
