//! Imports et exports par fichiers.

use super::*;

/// Zoom de l'interface selon la taille de l'écran en points logiques
/// (pixels / mise à l'échelle du système), **hors zoom de l'application**.
pub(super) fn zoom_for_screen(w: f32, h: f32) -> f32 {
    // la hauteur décide (un double écran très large n'est pas plus haut) ; la
    // largeur ne fait que réduire sur un petit portable
    let z: f32 = if h < 720.0 {
        0.8
    } else if h < 860.0 {
        0.9
    } else if h < 1250.0 {
        1.0
    } else if h < 1500.0 {
        1.2
    } else if h < 1900.0 {
        1.4
    } else {
        1.7
    };
    if w < 1300.0 { z.min(0.8) } else if w < 1500.0 { z.min(0.9) } else { z }
}

#[cfg(test)]
mod tests_zoom {
    use super::zoom_for_screen;

    /// Simulation de Windows : egui donne la taille de l'écran divisée par
    /// (échelle du système × zoom appliqué). L'ancien calcul réinjectait ce
    /// zoom et oscillait (1920×1080 à 150 % : 0,8 ↔ 1,0 à chaque image) ; le
    /// nouveau retire le zoom et reste stable pour tous les écrans courants.
    #[test]
    fn zoom_stable_sur_les_ecrans_courants() {
        for (pw, ph) in [(1366.0, 768.0), (1600.0, 900.0), (1920.0, 1080.0), (2560.0, 1440.0), (3840.0, 2160.0)] {
            for scale in [1.0f32, 1.25, 1.5, 1.75, 2.0] {
                let mut z = 1.0f32;
                let mut seen = Vec::new();
                for _ in 0..6 {
                    let (mw, mh) = (pw / (scale * z), ph / (scale * z)); // ce que donne egui
                    z = zoom_for_screen(mw * z, mh * z); // correction : zoom retiré
                    seen.push(z);
                }
                assert!(seen.windows(2).all(|w| w[0] == w[1]), "{pw}×{ph} à {scale} : {seen:?}");
            }
        }
        // l'ancien calcul oscillait bien dans ce cas
        let (mut z, mut seen) = (1.0f32, Vec::new());
        for _ in 0..4 {
            z = zoom_for_screen(1920.0 / (1.5 * z), 1080.0 / (1.5 * z));
            seen.push(z);
        }
        assert_eq!(seen, vec![0.8, 1.0, 0.8, 1.0]);
    }
}

impl App {
    /// Taille de l'interface : automatique selon la résolution de l'écran
    /// (portables 1366×768 / 1920×1080 en 15″), ou choix manuel.
    pub(super) fn apply_zoom(&mut self, ctx: &egui::Context) {
        let z = if self.cfg.ui_scale > 0.0 {
            self.cfg.ui_scale
        } else {
            // Taille de l'écran indépendante du zoom déjà appliqué (egui la donne
            // en points zoomés : sans cette correction, le zoom choisi modifiait
            // la taille mesurée, qui modifiait le zoom… l'interface tremblait,
            // notamment sous Windows avec une mise à l'échelle de 125 ou 150 %).
            // Décidé une seule fois, dès que la taille de l'écran est connue.
            if self.auto_zoom.is_none()
                && let Some(m) = ctx.input(|i| i.viewport().monitor_size)
            {
                self.auto_zoom = Some(zoom_for_screen(m.x * ctx.zoom_factor(), m.y * ctx.zoom_factor()));
            }
            self.auto_zoom.unwrap_or(1.0)
        };
        let first = self.applied_zoom == 0.0;
        if (z - self.applied_zoom).abs() > 1e-3 {
            ctx.set_zoom_factor(z);
            self.applied_zoom = z;
        }
        // panneau des paramètres replié d'office sur un écran très étroit
        if first {
            self.show_params = ctx.screen_rect().width() > 900.0;
        }
    }

    /// Traite des fichiers choisis par l'utilisateur (dialogue natif ou
    /// sélecteur du navigateur).
    pub(super) fn on_opened(&mut self, purpose: u32, files: Vec<crate::platform::Opened>, ctx: &egui::Context) {
        use crate::exchange as x;
        use crate::platform::stem;
        let Some(kind) = Pick::from(purpose) else { return };
        let txt = |b: &[u8]| String::from_utf8_lossy(b).to_string();
        match kind {
            Pick::Figures => self.load_figures(files),
            Pick::Simumsap => {
                if let Some((n, b)) = files.into_iter().next() {
                    self.open_simumsap(&n, &b, ctx);
                }
            }
            _ => {
                let Some((name, bytes)) = files.into_iter().next() else { return };
                match kind {
                    Pick::Project => match serde_json::from_slice::<MotorSpec>(&bytes) {
                        Ok(s) => {
                            self.spec = s;
                            if !crate::platform::WEB {
                                self.cfg.last_project = Some(name.clone());
                                self.push_recent(&name);
                            }
                            self.hist_reset();
                            self.touched(ctx);
                        }
                        Err(_) => self.err = Some(self.t("Fichier de projet illisible").into()),
                    },
                    Pick::Angles => match x::import_angles_csv(&txt(&bytes)) {
                        Ok(a) => {
                            self.spec.n_poles = a.len();
                            self.spec.custom_angles_deg = Some(a);
                            self.touched(ctx);
                            self.status = format!("{} {}", self.t("Importé :"), name);
                        }
                        Err(e) => self.err = Some(e),
                    },
                    Pick::PhiJson => match x::import_phi_sim_json(&txt(&bytes)) {
                        Ok(r) => {
                            if let Some(a) = r.magnet_angles.filter(|a| a.len() % 2 == 0 && a.len() >= 2) {
                                self.spec.n_poles = a.len();
                                self.spec.custom_angles_deg = Some(a);
                                self.touched(ctx);
                            }
                            self.overlays.push((format!("{} {}", self.t("externe"), r.variant), r.curve, "cogging"));
                            self.tab = Tab::Cogging;
                        }
                        Err(e) => self.err = Some(e),
                    },
                    Pick::BenchCogging | Pick::BenchEmf => match x::import_curve_csv(&txt(&bytes)) {
                        Ok(c) => {
                            let (kind, tab) = if kind == Pick::BenchCogging { ("cogging", Tab::Cogging) } else { ("fem", Tab::Fem) };
                            self.overlays.push((format!("{} {}", self.t("mesure"), stem(&name)), c, kind));
                            self.tab = tab;
                        }
                        Err(e) => self.err = Some(e),
                    },
                    Pick::Measure => match x::import_curve_csv(&txt(&bytes)) {
                        Ok(mut pts) => {
                            if self.recal_time_x {
                                // temps [s] → angle du rotor [°]
                                for q in &mut pts {
                                    q[0] *= self.recal_rpm * 6.0;
                                }
                            }
                            self.recal_meas = pts;
                            self.recal_file = name.rsplit(['/', '\\']).next().unwrap_or(&name).to_string();
                            self.recal_res = None;
                        }
                        Err(e) => self.err = Some(e),
                    },
                    _ => {}
                }
            }
        }
    }

    /// Enregistre un fichier exporté et met à jour la barre d'état.
    pub(super) fn export_one(&mut self, name: String, filter: &str, ext: &str, data: &[u8]) {
        match crate::platform::save(&name, filter, ext, data) {
            Some(Ok(p)) => self.status = format!("{} {}", self.t("Exporté :"), p),
            Some(Err(e)) => self.err = Some(e),
            None => {}
        }
    }

    /// Enregistre plusieurs fichiers (dossier sur ordinateur, ZIP dans le navigateur).
    pub(super) fn export_many(&mut self, zip_name: String, files: Vec<(String, Vec<u8>)>) {
        match crate::platform::save_many(None, &zip_name, &files) {
            Some(Ok(p)) => self.status = format!("{} {}", self.t("Exporté :"), p),
            Some(Err(e)) => self.err = Some(e),
            None => {}
        }
    }

    pub(super) fn import_menu(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        if ui.button(self.t("Angles des aimants (CSV)…")).on_hover_text(self.t("Un angle par ligne en degrés, ou « n ; angle »")).clicked() {
            ui.close_menu();
            Pick::Angles.ask("CSV", &["csv", "txt"]);
        }
        if ui.button(self.t("Résultats de simulation externe (JSON)…")).on_hover_text(self.t("Reprend les angles d'aimants et superpose la courbe de couple au cogging")).clicked() {
            ui.close_menu();
            Pick::PhiJson.ask("JSON", &["json"]);
        }
        if ui.button(self.t("Mesure de banc : cogging (CSV θ° ; N·m)…")).clicked() {
            ui.close_menu();
            Pick::BenchCogging.ask("CSV", &["csv", "txt"]);
        }
        if ui.button(self.t("Mesure de banc : FEM (CSV θ° ; V)…")).clicked() {
            ui.close_menu();
            Pick::BenchEmf.ask("CSV", &["csv", "txt"]);
        }
        if ui.button(self.t("Projet SIMUMSAP (.txt)…")).clicked() {
            ui.close_menu();
            Pick::Simumsap.ask("SIMUMSAP", &["txt"]);
        }
        if ui.button(self.t("Figures MATLAB (.fig) / données (.mat)…")).on_hover_text(self.t("Lit les fichiers MATLAB sans MATLAB : courbes affichables et superposables aux calculs")).clicked() {
            ui.close_menu();
            Pick::Figures.ask("MATLAB", &["fig", "mat"]);
        }
        if !self.overlays.is_empty() {
            ui.separator();
            if ui.button(self.t("Retirer les courbes importées")).clicked() {
                self.overlays.clear();
                ui.close_menu();
            }
        }
    }

    /// Ouvre un projet SIMUMSAP : machine reconstruite, fiche du projet ; le
    /// dossier (sur ordinateur) sert à retrouver les figures MATLAB.
    pub(super) fn open_simumsap(&mut self, name: &str, bytes: &[u8], ctx: &egui::Context) {
        match crate::exchange::import_simumsap(bytes) {
            Ok(r) => {
                self.spec = r.spec;
                self.spec.name = crate::platform::stem(name).chars().take(40).collect();
                self.err = r.warning.clone();
                self.simu = Some((r.fields, r.inductances, crate::platform::parent_dir(name)));
                self.touched(ctx);
            }
            Err(e) => self.err = Some(e),
        }
    }

    pub(super) fn load_figures(&mut self, files: Vec<crate::platform::Opened>) {
        for (n, b) in files {
            match crate::matfile::figure_curves(&b) {
                Ok(ax) if !ax.is_empty() => self.figs.push((n.rsplit(['/', '\\']).next().unwrap_or(&n).to_string(), ax)),
                Ok(_) => {}
                Err(e) => self.err = Some(format!("{n} : {e}")),
            }
        }
        self.show_figs = !self.figs.is_empty();
    }

    pub(super) fn export_menu(&mut self, ui: &mut egui::Ui) {
        use crate::exchange as x;
        let n = self.spec.name.clone();
        let spec = self.spec.clone();
        ui.label(RichText::new(self.t("Simulation (logiciels libres)")).weak().small());
        if ui.button(self.t("Modèle FEMM (.lua)…")).on_hover_text(self.t("Géométrie, matériaux et balayage du cogging prêts à lancer dans FEMM 4.2")).clicked() {
            ui.close_menu();
            let span = if self.st.common_span_deg > 0.0 { self.st.common_span_deg } else { spec.cogging_period_deg() };
            self.export_one(format!("{n}.lua"), "Lua", "lua", x::femm_lua(&spec, span, 0.25).as_bytes());
        }
        if ui.button(self.t("Géométrie Gmsh (.geo)…")).on_hover_text(self.t("Régions nommées, maillable pour Elmer / GetDP")).clicked() {
            ui.close_menu();
            self.export_one(format!("{n}.geo"), "Gmsh", "geo", x::gmsh_geo(&spec).as_bytes());
        }
        ui.label(RichText::new(self.t("Fabrication")).weak().small());
        if ui.button(self.t("Coupe 2D (DXF)…")).clicked() {
            ui.close_menu();
            self.export_one(format!("{n}.dxf"), "DXF", "dxf", x::dxf(&spec).as_bytes());
        }
        if ui.button(self.t("Gabarit de pose des aimants 1:1 (SVG)…")).on_hover_text(self.t("À imprimer à 100 % : positions, polarités et angles de chaque aimant")).clicked() {
            ui.close_menu();
            self.export_one(format!("{n}_gabarit.svg"), "SVG", "svg", x::svg_gabarit(&spec).as_bytes());
        }
        if ui.button(self.t("Modèle 3D (STL : stator, aimants, cloche)…")).clicked() {
            ui.close_menu();
            let files: Vec<(String, Vec<u8>)> = ["stator", "aimants", "cloche"].iter().map(|part| (format!("{n}_{part}.stl"), x::stl(&spec, part))).collect();
            self.export_many(format!("{n}_stl.zip"), files);
        }
        ui.label(RichText::new(self.t("Rapports et données")).weak().small());
        let has = self.res.is_some();
        if ui.add_enabled(has, egui::Button::new(self.t("Rapport HTML autonome…"))).clicked() {
            ui.close_menu();
            if let Some(d) = self.res.as_ref().map(|res| x::html_report(res, &self.cmp)) {
                self.export_one(format!("{n}_rapport.html"), "HTML", "html", d.as_bytes());
            }
        }
        if ui.add_enabled(has, egui::Button::new(self.t("Tableau LaTeX (.tex)…"))).clicked() {
            ui.close_menu();
            if let Some(d) = self.res.as_ref().map(|res| x::latex(res, &self.cmp)) {
                self.export_one(format!("{n}_tableau.tex"), "LaTeX", "tex", d.as_bytes());
            }
        }
        if ui.add_enabled(has, egui::Button::new(self.t("Script Octave / MATLAB (.m)…"))).clicked() {
            ui.close_menu();
            if let Some(d) = self.res.as_ref().map(x::octave_script) {
                self.export_one(format!("{n}_donnees.m"), "Octave", "m", d.as_bytes());
            }
        }
        if ui.add_enabled(has, egui::Button::new(self.t("Script Python (.py)…"))).clicked() {
            ui.close_menu();
            if let Some(d) = self.res.as_ref().map(x::python_script) {
                self.export_one(format!("{n}_donnees.py"), "Python", "py", d.as_bytes());
            }
        }
        if ui.add_enabled(has, egui::Button::new(self.t("Courbes (CSV)…"))).clicked() {
            ui.close_menu();
            self.export_csv();
        }
        if ui.add_enabled(has, egui::Button::new(self.t("Rapport (JSON)…"))).clicked() {
            ui.close_menu();
            if let Some(d) = self.res.as_ref().map(|res| serde_json::to_string_pretty(&res.report).unwrap_or_default()) {
                self.export_one(format!("rapport_{n}.json"), "JSON", "json", d.as_bytes());
            }
        }
    }

    pub(super) fn export_csv(&mut self) {
        let Some(r) = &self.res else { return };
        let n = r.spec.name.clone();
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut s = String::from("theta_r_deg;couple_Nm\n");
        for (a, v) in r.cog.angles_deg.iter().zip(&r.cog.torque_nm) {
            s += &format!("{a};{v}\n");
        }
        files.push((format!("{n}_cogging.csv"), s.into_bytes()));
        let mut s = String::from("theta_r_deg;eA_V;eB_V;eC_V;psiA_Wb;psiB_Wb;psiC_Wb\n");
        for k in 0..r.emf.angles_deg.len() {
            s += &format!(
                "{};{};{};{};{};{};{}\n",
                r.emf.angles_deg[k], r.emf.emf[0][k], r.emf.emf[1][k], r.emf.emf[2][k], r.emf.flux[0][k], r.emf.flux[1][k], r.emf.flux[2][k]
            );
        }
        files.push((format!("{n}_fem.csv"), s.into_bytes()));
        let mut s = String::from("theta_r_deg;Fx_N;Fy_N\n");
        for k in 0..r.forces.angles_deg.len() {
            s += &format!("{};{};{}\n", r.forces.angles_deg[k], r.forces.fx[k], r.forces.fy[k]);
        }
        files.push((format!("{n}_forces.csv"), s.into_bytes()));
        let f = r.model.solve(self.theta_r.to_radians());
        let g = analysis::gap_field(&r.model, &f, 1440);
        let mut s = format!("# theta_r = {} deg\ntheta_deg;Br_T;Bt_T\n", self.theta_r);
        for k in 0..g.theta_deg.len() {
            s += &format!("{};{};{}\n", g.theta_deg[k], g.br[k], g.bt[k]);
        }
        files.push((format!("{n}_champ_entrefer.csv"), s.into_bytes()));
        match crate::platform::save_many(None, &format!("{n}_courbes.zip"), &files) {
            Some(Ok(p)) => self.status = format!("{} {}", self.t("CSV exportés dans"), p),
            Some(Err(e)) => self.err = Some(e),
            None => {}
        }
    }
}
