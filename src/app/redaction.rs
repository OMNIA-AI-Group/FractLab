//! Rédaction (document LaTeX / PDF, impression) et sauvegarde générale.

use super::*;

impl App {
    pub(super) fn make_doc(&mut self) {
        let Some(r) = &self.res else { return };
        let d = crate::doc::build(&self.spec.sheet, r, &self.cmp, self.sim_res.as_ref(), self.doc_lang.unwrap_or(self.cfg.lang));
        self.latex_src = crate::doc::to_latex(&d);
        self.pdf = Some(match crate::doc::compile_latex(&self.latex_src) {
            Some((b, eng)) => (b, eng),
            None => (crate::doc::to_pdf_builtin(&d), T(self.cfg.lang, "générateur intégré").to_string()),
        });
        self.doc = Some(d);
    }

    pub(super) fn redac_window(&mut self, ctx: &egui::Context) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let mut open = self.show_redac;
        let mut regen = false;
        egui::Window::new(format!("📝 {}", t("Rédaction"))).open(&mut open).default_size([1000.0, 640.0]).resizable(true).show(ctx, |ui| {
            egui::SidePanel::left("redac_form").resizable(true).default_width(330.0).show_inside(ui, |ui| {
                ui.label(RichText::new(t("Fiche du projet (tout est facultatif)")).strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        lbl(ui, t("Langue du document"));
                        let cur = self.doc_lang.unwrap_or(self.cfg.lang);
                        egui::ComboBox::from_id_source("doclang").selected_text(cur.name()).show_ui(ui, |ui| {
                            for l in [Lang::Fr, Lang::En, Lang::Es, Lang::De] {
                                if ui.selectable_label(cur == l, l.name()).clicked() {
                                    self.doc_lang = Some(l);
                                }
                            }
                        });
                    });
                    let dl = self.doc_lang.unwrap_or(self.cfg.lang);
                    if dl != Lang::Fr {
                        ui.label(RichText::new(t("Titre, sous-titre, mots-clés, résumé et remarques sont propres à cette langue ; un champ vide reprend le texte français.")).small().weak());
                    }
                    let sh = &mut self.spec.sheet;
                    egui::Grid::new("sheet").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                        let f = |ui: &mut egui::Ui, k: &str, v: &mut String| {
                            ui.label(k);
                            ui.add(egui::TextEdit::singleline(v).desired_width(190.0));
                            ui.end_row();
                        };
                        let mut tr_entry = if dl == Lang::Fr { None } else { Some(sh.translations.entry(dl.code().to_string()).or_default().clone()) };
                        match tr_entry.as_mut() {
                            None => {
                                f(ui, t("Titre"), &mut sh.title);
                                f(ui, t("Sous-titre"), &mut sh.subtitle);
                            }
                            Some(e) => {
                                f(ui, t("Titre"), &mut e.title);
                                f(ui, t("Sous-titre"), &mut e.subtitle);
                            }
                        }
                        f(ui, t("Auteurs"), &mut sh.authors);
                        f(ui, t("Organisation"), &mut sh.organisation);
                        f(ui, "INPI", &mut sh.inpi);
                        f(ui, t("Autres dépôts"), &mut sh.deposits);
                        f(ui, t("Licence"), &mut sh.licence);
                        f(ui, "arXiv", &mut sh.arxiv);
                        f(ui, "DOI", &mut sh.doi);
                        f(ui, "GitHub", &mut sh.github);
                        f(ui, t("Site web"), &mut sh.website);
                        f(ui, t("Adresses mail"), &mut sh.emails);
                        f(ui, "LinkedIn", &mut sh.linkedin);
                        match tr_entry.as_mut() {
                            None => f(ui, t("Mots-clés"), &mut sh.keywords),
                            Some(e) => f(ui, t("Mots-clés"), &mut e.keywords),
                        }
                        if let Some(e) = &tr_entry {
                            sh.translations.insert(dl.code().to_string(), e.clone());
                        }
                    });
                    let fr_sheet = sh.clone();
                    let (abs, notes) = if dl == Lang::Fr {
                        (&mut sh.abstract_, &mut sh.notes)
                    } else {
                        let e = sh.translations.entry(dl.code().to_string()).or_default();
                        (&mut e.abstract_, &mut e.notes)
                    };
                    lbl(ui, t("Résumé"));
                    let hint_a = if dl == Lang::Fr { String::new() } else { fr_sheet.abstract_.clone() };
                    ui.add(egui::TextEdit::multiline(abs).hint_text(hint_a).desired_rows(4).desired_width(f32::INFINITY));
                    lbl(ui, t("Remarques"));
                    let hint_n = if dl == Lang::Fr { String::new() } else { fr_sheet.notes.clone() };
                    ui.add(egui::TextEdit::multiline(notes).hint_text(hint_n).desired_rows(3).desired_width(f32::INFINITY));
                    if ui.add(egui::Button::new(RichText::new(format!("⚙ {}", t("Générer le document"))).strong())).clicked() {
                        regen = true;
                    }
                });
            });
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.redac_pdf_view, false, "LaTeX");
                    ui.selectable_value(&mut self.redac_pdf_view, true, "PDF");
                    ui.separator();
                    if let Some((_, eng)) = &self.pdf {
                        ui.label(RichText::new(format!("PDF : {eng}")).weak().small());
                    }
                    if ui.button(t("Ouvrir le PDF")).clicked()
                        && let Some((b, _)) = &self.pdf {
                            let _ = crate::platform::open_bytes(&format!("{}.pdf", self.spec.name), "application/pdf", b);
                        }
                    if ui.button(t("Enregistrer .tex")).clicked() {
                        // le .tex et le logo qu'il cite, côte à côte (archive ZIP dans le navigateur)
                        let files = vec![(format!("{}.tex", self.spec.name), self.latex_src.clone().into_bytes()), ("fractlab_logo.png".to_string(), crate::doc::LOGO_PNG.to_vec())];
                        match crate::platform::save_many(None, &format!("{}_latex.zip", self.spec.name), &files) {
                            Some(Ok(p)) => self.status = format!("{} {}", t("Enregistré :"), p),
                            Some(Err(e)) => self.err = Some(e),
                            None => {}
                        }
                    }
                    ui.menu_button(format!("⎙ {}", t("Imprimer")), |ui| {
                        if self.printers.is_empty() {
                            self.printers = list_printers();
                        }
                        ui.label(RichText::new(t("Imprimante")).strong());
                        if self.printers.is_empty() {
                            ui.label(RichText::new(t("aucune imprimante trouvée")).weak());
                        }
                        for p in self.printers.clone() {
                            ui.radio_value(&mut self.printer, p.clone(), p);
                        }
                        ui.horizontal(|ui| {
                            lbl(ui, t("Copies"));
                            ui.add(egui::DragValue::new(&mut self.copies).clamp_range(1..=99));
                        });
                        if ui.add_enabled(self.pdf.is_some() && !self.printer.is_empty(), egui::Button::new(RichText::new(format!("⎙ {}", t("Lancer l'impression"))).strong())).clicked() {
                            if let Some((b, _)) = &self.pdf {
                                let ok = crate::platform::print_pdf(&format!("{}_impression.pdf", self.spec.name), b, &self.printer, self.copies as u32);
                                self.status = if ok { format!("{} {}", T(lang, "Impression envoyée à"), self.printer) } else { T(lang, "Échec de l'impression").into() };
                            }
                            ui.close_menu();
                        }
                    });
                    if ui.button(t("Enregistrer .pdf")).clicked()
                        && let Some((b, _)) = &self.pdf {
                            let _ = crate::platform::save(&format!("{}.pdf", self.spec.name), "PDF", "pdf", b);
                        }
                });
                ui.separator();
                if self.doc.is_none() {
                    lbl(ui, t("Remplissez la fiche puis « Générer le document »."));
                    return;
                }
                if !self.redac_pdf_view {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.latex_src).code_editor().desired_width(f32::INFINITY));
                    });
                } else {
                    let d = self.doc.clone().unwrap();
                    egui::Frame::none().fill(Color32::WHITE).inner_margin(egui::Margin::same(18.0)).show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let ink = Color32::from_rgb(20, 20, 25);
                            ui.label(RichText::new(&d.sheet.title).size(22.0).strong().color(ink));
                            if !d.sheet.subtitle.is_empty() {
                                ui.label(RichText::new(&d.sheet.subtitle).size(15.0).color(ink));
                            }
                            ui.label(RichText::new(format!("{} — {}", d.sheet.authors, d.sheet.organisation)).color(ink));
                            for (k, v) in [("INPI", &d.sheet.inpi), ("Licence", &d.sheet.licence), ("arXiv", &d.sheet.arxiv), ("GitHub", &d.sheet.github), ("LinkedIn", &d.sheet.linkedin), ("Contact", &d.sheet.emails), ("Site", &d.sheet.website)] {
                                if !v.trim().is_empty() {
                                    ui.label(RichText::new(format!("{k} : {v}")).small().color(Color32::DARK_GRAY));
                                }
                            }
                            for (bi, b) in d.blocks.iter().enumerate() {
                                match b {
                                    crate::doc::Block::Heading(h) => {
                                        ui.add_space(8.0);
                                        ui.label(RichText::new(h).size(17.0).strong().color(Color32::from_rgb(0, 80, 140)));
                                    }
                                    crate::doc::Block::Para(p) => {
                                        ui.add(egui::Label::new(RichText::new(p).color(ink)).wrap(true));
                                    }
                                    crate::doc::Block::Table(h, rows) => {
                                        egui::Grid::new(("dt", bi)).striped(true).num_columns(h.len()).show(ui, |ui| {
                                            for c in h {
                                                ui.label(RichText::new(c).strong().color(ink));
                                            }
                                            ui.end_row();
                                            for r in rows {
                                                for c in r {
                                                    ui.label(RichText::new(c).color(ink));
                                                }
                                                ui.end_row();
                                            }
                                        });
                                    }
                                    crate::doc::Block::Plot(tt, xl, yl, series) => {
                                        ui.label(RichText::new(tt).strong().color(ink));
                                        Plot::new(("dp", bi)).height(170.0).x_axis_label(xl.clone()).y_axis_label(yl.clone()).legend(Legend::default()).show(ui, |p| {
                                            for (k, (n, pts)) in series.iter().enumerate() {
                                                p.line(Line::new(PlotPoints::from(pts.clone())).name(n).color(SERIES[k % 6]));
                                            }
                                        });
                                    }
                                }
                            }
                        });
                    });
                }
            });
        });
        if regen {
            self.make_doc();
            self.redac_pdf_view = true;
        }
        self.show_redac = open;
    }

    pub(super) fn save_window(&mut self, ctx: &egui::Context) {
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        let mut open = self.show_save;
        let mut go = false;
        egui::Window::new(format!("💾 {}", t("Sauvegarde générale"))).open(&mut open).default_width(460.0).show(ctx, |ui| {
            lbl(ui, t("Crée un dossier contenant : le document LaTeX et PDF, les données du projet (JSON), une sauvegarde MATLAB (.mat), et les formats cochés ci-dessous."));
            ui.separator();
            for (name, on) in self.save_opts.iter_mut() {
                ui.checkbox(on, t(name));
            }
            ui.separator();
            if ui.add_enabled(self.res.is_some(), egui::Button::new(RichText::new(format!("💾 {}", t("Choisir l'emplacement et sauvegarder"))).strong())).clicked() {
                go = true;
            }
        });
        if go {
            match self.save_all() {
                Some(Ok(dir)) => {
                    self.status = format!("{} {}", t("Sauvegardé dans"), dir);
                    open = false;
                }
                Some(Err(e)) => self.err = Some(e),
                None => {}
            }
        }
        self.show_save = open;
    }

    /// Sauvegarde générale dans un nouveau dossier (archive ZIP dans le
    /// navigateur) ; renvoie son emplacement, ou `None` si annulé.
    pub fn save_all(&mut self) -> Option<Result<String, String>> {
        let files = match self.save_all_files() {
            Ok(f) => f,
            Err(e) => return Some(Err(e)),
        };
        let safe: String = self.spec.name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
        let stamp = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let folder = format!("{safe}_FractLab_{stamp}");
        crate::platform::save_many(Some(&folder), &format!("{folder}.zip"), &files)
    }

    /// Contenu de la sauvegarde générale : (nom de fichier, octets).
    pub fn save_all_files(&mut self) -> Result<Vec<(String, Vec<u8>)>, String> {
        use crate::exchange as x;
        self.make_doc();
        let r = self.res.as_ref().ok_or("aucun résultat")?;
        let safe: String = self.spec.name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut w = |name: &str, data: &[u8]| -> Result<(), String> {
            files.push((name.to_string(), data.to_vec()));
            Ok(())
        };
        w(&format!("{safe}.tex"), self.latex_src.as_bytes())?;
        w("fractlab_logo.png", crate::doc::LOGO_PNG)?;
        if let Some((b, _)) = &self.pdf {
            w(&format!("{safe}.pdf"), b)?;
        }
        w(&format!("{safe}_projet.json"), serde_json::to_string_pretty(&self.spec).unwrap_or_default().as_bytes())?;
        w(&format!("{safe}.mat"), &crate::doc::results_mat(r, self.sim_res.as_ref()))?;
        if !self.sweep_rows.is_empty() {
            w(&format!("{safe}_etude_parametrique.csv"), crate::sweep::csv(self.sweep_var, &self.sweep_rows).as_bytes())?;
        }
        let spec = &r.spec;
        for (name, on) in &self.save_opts {
            if !*on {
                continue;
            }
            match *name {
                "Coupe 2D (DXF)" => w(&format!("{safe}.dxf"), x::dxf(spec).as_bytes())?,
                "Modèle 3D (STL)" => {
                    for part in ["stator", "aimants", "cloche"] {
                        w(&format!("{safe}_{part}.stl"), &x::stl(spec, part))?;
                    }
                }
                "Gabarit de pose 1:1 (SVG)" => w(&format!("{safe}_gabarit.svg"), x::svg_gabarit(spec).as_bytes())?,
                "Modèle FEMM (.lua)" => w(&format!("{safe}.lua"), x::femm_lua(spec, r.cog.metrics.span_deg, 0.25).as_bytes())?,
                "Géométrie Gmsh (.geo)" => w(&format!("{safe}.geo"), x::gmsh_geo(spec).as_bytes())?,
                "Courbes (CSV)" => {
                    let mut c = String::from("theta_deg;couple_Nm\n");
                    for (a, v) in r.cog.angles_deg.iter().zip(&r.cog.torque_nm) {
                        c += &format!("{a};{v}\n");
                    }
                    w(&format!("{safe}_cogging.csv"), c.as_bytes())?;
                    let mut c = String::from("theta_deg;eA_V;eB_V;eC_V\n");
                    for k in 0..r.emf.angles_deg.len() {
                        c += &format!("{};{};{};{}\n", r.emf.angles_deg[k], r.emf.emf[0][k], r.emf.emf[1][k], r.emf.emf[2][k]);
                    }
                    w(&format!("{safe}_fem.csv"), c.as_bytes())?;
                }
                "Rapport HTML" => w(&format!("{safe}_rapport.html"), x::html_report(r, &self.cmp).as_bytes())?,
                "Script Octave / MATLAB (.m)" => w(&format!("{safe}_donnees.m"), x::octave_script(r).as_bytes())?,
                "Script Python (.py)" => w(&format!("{safe}_donnees.py"), x::python_script(r).as_bytes())?,
                "Rapport (JSON)" => w(&format!("{safe}_rapport.json"), serde_json::to_string_pretty(&r.report).unwrap_or_default().as_bytes())?,
                "Son synthétisé (WAV)" => w(&format!("{safe}_{:.0}rpm.wav", spec.rpm), &crate::dynamics::synth_wav(r, spec.rpm, 5.0))?,
                _ => {}
            }
        }
        Ok(files)
    }
}
