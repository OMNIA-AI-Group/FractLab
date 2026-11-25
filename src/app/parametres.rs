//! Panneau des paramètres : toutes les données modifiables du projet.

use super::*;

impl App {
    // ------------------------------------------------------------------
    // Panneau des paramètres (toutes les données modifiables)
    // ------------------------------------------------------------------
    pub(super) fn params_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("params").resizable(true).default_width(340.0).show(ctx, |ui| {
            set_panel_right(ui.max_rect().right());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Label::new(RichText::new("FractLab").heading().color(CYAN).strong()).sense(egui::Sense::click())).on_hover_text(tr("Licence")).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.show_license = true;
                }
                if ui.add(egui::Label::new(RichText::new("Φ").color(ORANGE).size(22.0).strong()).sense(egui::Sense::click())).on_hover_text(tr("Développeur de l'application")).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.show_author = true;
                }
            });
            ui.label(RichText::new(self.t("Simulateur de machines à aimants permanents")).small().weak());
            ui.separator();
            let mut changed = false;
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                changed |= self.params_ui(ui);
            });
            if changed {
                self.touched(ctx);
            }
        });
    }

    pub(super) fn len_edit(ui: &mut egui::Ui, units: &Units, label: &str, v: &mut f64, lo: f64, hi: f64) -> bool {
        let f = units.factor(Q::Length);
        let mut x = *v * f;
        lbl(ui, label);
        let r = ui.add(
            egui::DragValue::new(&mut x)
                .speed(0.01 * f)
                .clamp_range(lo * f..=hi * f)
                .max_decimals(4)
                .suffix(format!(" {}", units.sym(Q::Length))),
        );
        ui.end_row();
        if r.changed() {
            *v = x / f;
            true
        } else {
            false
        }
    }

    pub(super) fn params_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut ch = false;
        let units = self.cfg.units.clone();
        let lang = self.cfg.lang;
        let t = |k: &'static str| T(lang, k);
        ui.horizontal(|ui| {
            let busy = self.job.is_some();
            if ui
                .add_enabled(!busy, egui::Button::new(RichText::new(format!("▶ {}", t("Calcul"))).strong()).fill(Color32::from_rgb(0, 110, 160)))
                .on_hover_text(crate::tips::tip(lang, "Calcul").unwrap_or_default())
                .clicked()
            {
                self.launch();
            }
            chk(ui, &mut self.auto, t("auto"));
            if ui.button(format!("📝 {}", t("Rédaction"))).on_hover_text(t("Fiche du projet et document LaTeX / PDF")).clicked() {
                self.show_redac = true;
            }
            if ui.button("💾").on_hover_text(t("Sauvegarde générale")).clicked() {
                self.show_save = true;
            }
            if ui.add(egui::Button::new(RichText::new(format!("✨ {}", t("Ajustement I.A"))).strong()).fill(Color32::from_rgb(140, 70, 0))).on_hover_text(t("Optimise automatiquement les réglages et propose les 5 meilleures solutions")).clicked() {
                self.show_opt = true;
            }
            if ui.button(format!("🖥 {}", t("Console"))).on_hover_text(t("Déroulé du moteur I.A et console de script")).clicked() {
                self.console.open = !self.console.open;
            }
            self.field3d_button(ui, false);
            if busy {
                ui.spinner();
            }
        });
        ui.add_space(4.0);

        egui::CollapsingHeader::new(RichText::new(format!("▣ {}", t("Projet et variante"))).strong()).default_open(true).show(ui, |ui| {
            egui::Grid::new("g_proj").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                lbl(ui, t("Nom"));
                ch |= ui.text_edit_singleline(&mut self.spec.name).changed();
                ui.end_row();
                lbl(ui, t("Variante"));
                let cur = if self.spec.custom_angles_deg.is_some() {
                    t("Personnalisée").to_string()
                } else if self.spec.variant == crate::repartition::REGULIERE {
                    t(crate::repartition::REGULIERE).to_string()
                } else {
                    self.spec.variant.clone()
                };
                egui::ComboBox::from_id_source("variant").selected_text(cur).show_ui(ui, |ui| {
                    let variants = crate::repartition::variants();
                    for v in variants.iter().copied() {
                        let label = if v == crate::repartition::REGULIERE { t(v).to_string() } else { format!("{v} — {}", t(crate::repartition::describe(v))) };
                        if ui.selectable_label(self.spec.variant == v && self.spec.custom_angles_deg.is_none(), label).clicked() {
                            let keep_name = !variants.contains(&self.spec.name.as_str());
                            self.spec.variant = v.into();
                            if !keep_name {
                                self.spec.name = v.into();
                            }
                            self.spec.custom_angles_deg = None;
                            crate::repartition::on_select(&mut self.spec);
                            ch = true;
                        }
                    }
                });
                ui.end_row();
                lbl(ui, t("Aimantation"));
                let mt = match self.spec.magnetisation {
                    Magnetisation::Radiale => t("Radiale"),
                    Magnetisation::Parallele => t("Parallèle"),
                    Magnetisation::Halbach { .. } => "Halbach",
                };
                egui::ComboBox::from_id_source("mag").selected_text(mt).show_ui(ui, |ui| {
                    ch |= ui.selectable_value(&mut self.spec.magnetisation, Magnetisation::Radiale, t("Radiale")).changed();
                    ch |= ui.selectable_value(&mut self.spec.magnetisation, Magnetisation::Parallele, t("Parallèle")).changed();
                    if ui.selectable_label(matches!(self.spec.magnetisation, Magnetisation::Halbach { .. }), "Halbach").clicked() {
                        self.spec.magnetisation = Magnetisation::Halbach { segments: 4 };
                        ch = true;
                    }
                });
                ui.end_row();
                if let Magnetisation::Halbach { segments } = &mut self.spec.magnetisation {
                    lbl(ui, t("Segments par pôle"));
                    ch |= ui.add(egui::DragValue::new(segments).clamp_range(2..=12)).changed();
                    ui.end_row();
                }
            });
        });

        egui::CollapsingHeader::new(RichText::new(format!("⚙ {}", t("Topologie"))).strong()).default_open(true).show(ui, |ui| {
            egui::Grid::new("g_topo").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                lbl(ui, t("Type de machine"));
                let tn = |tp: Topology| match tp {
                    Topology::RadialOuter => t("Flux radial, rotor extérieur"),
                    Topology::RadialInner => t("Flux radial, rotor intérieur"),
                    Topology::Axial => t("Flux axial (multi-tranches)"),
                };
                let ti = |tp: Topology| match tp {
                    Topology::RadialOuter => 0,
                    Topology::RadialInner => 1,
                    Topology::Axial => 2,
                };
                let old_topo = self.spec.topology;
                egui::ComboBox::from_id_source("topo").selected_text(tn(self.spec.topology)).show_ui(ui, |ui| {
                    for tp in [Topology::RadialOuter, Topology::RadialInner, Topology::Axial] {
                        if ui.selectable_value(&mut self.spec.topology, tp, tn(tp)).changed() && tp != old_topo {
                            // chaque type de machine retrouve ses propres cotes
                            self.topo_dims[ti(old_topo)] = Some((self.spec.stator_ir, self.spec.slot_depth));
                            if let Some((ir, sd)) = self.topo_dims[ti(tp)] {
                                self.spec.stator_ir = ir;
                                self.spec.slot_depth = sd;
                            }
                            if tp == Topology::RadialOuter && self.spec.stator_ir >= self.spec.stator_or - self.spec.yoke_th {
                                // stator autour de l'arbre, sous l'alésage
                                self.spec.stator_ir = (self.spec.stator_or * 0.36).max(0.5);
                            }
                            if tp == Topology::RadialInner && self.spec.stator_ir <= self.spec.stator_or {
                                // géométrie de départ cohérente : stator autour du rotor
                                self.spec.stator_ir = self.spec.stator_or + 6.0;
                            }
                            if tp == Topology::Axial && self.spec.slot_depth <= 0.0 {
                                self.spec.slot_depth = (self.spec.axial.r_out - self.spec.axial.r_in) * 0.4;
                            }
                            // retour en radial : une profondeur d'encoche réglée pour un
                            // disque axial ne tient pas dans le stator → automatique
                            if self.spec.radial_slot_room().is_some_and(|room| self.spec.slot_depth > room) {
                                self.spec.slot_depth = 0.0;
                            }
                            ch = true;
                        }
                    }
                });
                ui.end_row();
                lbl(ui, t("Encoches Q"));
                ch |= ui.add(egui::DragValue::new(&mut self.spec.n_slots).clamp_range(3..=72)).changed();
                ui.end_row();
                lbl(ui, t("Pôles 2p"));
                let mut p = self.spec.n_poles;
                if ui.add(egui::DragValue::new(&mut p).clamp_range(2..=80)).changed() {
                    self.spec.n_poles = (p / 2).max(1) * 2;
                    self.spec.custom_angles_deg = None;
                    ch = true;
                }
                ui.end_row();
                let coils = winding::layout(&self.spec);
                lbl(ui, t("Bobinage"));
                let ok = winding::balanced(&coils);
                ui.label(RichText::new(winding::notation(&coils)).monospace().color(if ok { Color32::from_rgb(90, 200, 120) } else { Color32::RED }))
                    .on_hover_text(if ok { t("Bobinage équilibré") } else { t("Combinaison Q/2p non équilibrée") });
                ui.end_row();
            });
        });

        egui::CollapsingHeader::new(RichText::new(format!("◎ {}", t("Stator"))).strong()).default_open(true).show(ui, |ui| {
            egui::Grid::new("g_stator").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                match self.spec.topology {
                    Topology::RadialOuter => {
                        ch |= Self::len_edit(ui, &units, t("Rayon d'alésage (ext. stator)"), &mut self.spec.stator_or, 1.0, 5000.0);
                        ch |= Self::len_edit(ui, &units, t("Rayon intérieur stator"), &mut self.spec.stator_ir, 0.0, 5000.0);
                        ch |= Self::len_edit(ui, &units, t("Longueur active"), &mut self.spec.stack_len, 0.5, 5000.0);
                    }
                    Topology::RadialInner => {
                        ch |= Self::len_edit(ui, &units, t("Rayon d'alésage (int. stator)"), &mut self.spec.stator_or, 1.0, 5000.0);
                        ch |= Self::len_edit(ui, &units, t("Rayon extérieur du stator"), &mut self.spec.stator_ir, 1.0, 6000.0);
                        ch |= Self::len_edit(ui, &units, t("Longueur active"), &mut self.spec.stack_len, 0.5, 5000.0);
                    }
                    Topology::Axial => {
                        ch |= Self::len_edit(ui, &units, t("Rayon intérieur actif"), &mut self.spec.axial.r_in, 1.0, 5000.0);
                        ch |= Self::len_edit(ui, &units, t("Rayon extérieur actif"), &mut self.spec.axial.r_out, 2.0, 6000.0);
                        lbl(ui, t("Nombre de tranches"));
                        ch |= ui.add(egui::DragValue::new(&mut self.spec.axial.slices).clamp_range(1..=20)).changed();
                        ui.end_row();
                        lbl(ui, t("Nombre d'entrefers"));
                        ch |= ui.add(egui::DragValue::new(&mut self.spec.axial.gaps).clamp_range(1..=2)).changed();
                        ui.end_row();
                    }
                }
                ch |= Self::len_edit(ui, &units, t("Culasse stator"), &mut self.spec.yoke_th, 0.1, 500.0);
                ch |= Self::len_edit(ui, &units, t("Profondeur d'encoche (0 = auto)"), &mut self.spec.slot_depth, 0.0, 1000.0);
                ch |= Self::len_edit(ui, &units, t("Ouverture d'encoche"), &mut self.spec.slot_open, 0.05, 500.0);
                ch |= Self::len_edit(ui, &units, t("Becs de dents (0 = encoches ouvertes)"), &mut self.spec.tip_h, 0.0, 50.0);
                if self.spec.tip_h > 0.0 {
                    ch |= Self::len_edit(ui, &units, t("Largeur du corps d'encoche"), &mut self.spec.slot_w, 0.0, 500.0);
                }
                ch |= Self::len_edit(ui, &units, t("Largeur de dent"), &mut self.spec.tooth_w, 0.1, 500.0);
            });
        });

        egui::CollapsingHeader::new(RichText::new(format!("⚛ {}", t("Entrefer, aimants et rotor"))).strong()).default_open(true).show(ui, |ui| {
            egui::Grid::new("g_rotor").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ch |= Self::len_edit(ui, &units, t("Entrefer"), &mut self.spec.airgap, 0.02, 20.0);
                ch |= Self::len_edit(ui, &units, t("Épaisseur aimant"), &mut self.spec.magnet_th, 0.1, 50.0);
                ch |= Self::len_edit(ui, &units, t("Culasse rotor"), &mut self.spec.rotor_yoke_th, 0.1, 50.0);
                lbl(ui, if self.spec.topology == Topology::Axial { t("Arc aimant / pas polaire (r int.)") } else { t("Arc aimant / pas polaire") });
                ch |= ui.add(egui::Slider::new(&mut self.spec.magnet_arc_ratio, 0.3..=1.0).max_decimals(3)).changed();
                ui.end_row();
                if self.spec.topology == Topology::Axial {
                    lbl(ui, t("Arc aimant / pas polaire (r ext.)"));
                    ch |= ui.add(egui::Slider::new(&mut self.spec.axial.arc_ratio_out, 0.3..=1.0).max_decimals(3)).changed();
                    ui.end_row();
                }
                let fb = units.factor(Q::Induction);
                let mut br = self.spec.br_magnet * fb;
                lbl(ui, t("Induction rémanente Br"));
                if ui.add(egui::DragValue::new(&mut br).speed(0.005 * fb).clamp_range(0.1 * fb..=1.6 * fb).max_decimals(4).suffix(format!(" {}", units.sym(Q::Induction)))).changed() {
                    self.spec.br_magnet = br / fb;
                    ch = true;
                }
                ui.end_row();
                lbl(ui, t("Perméabilité de recul μr"));
                ch |= ui.add(egui::DragValue::new(&mut self.spec.mur_magnet).speed(0.005).clamp_range(1.0..=1.5).max_decimals(3)).changed();
                ui.end_row();
                ui.label(RichText::new(t("Préréglages")).weak());
                ui.horizontal(|ui| {
                    for (n, b) in [("N35", 1.19), ("N42", 1.30), ("N48", 1.38), ("N52", 1.45)] {
                        if ui.small_button(n).clicked() {
                            self.spec.br_magnet = b;
                            ch = true;
                        }
                    }
                });
                ui.end_row();
            });
        });

        egui::CollapsingHeader::new(RichText::new(format!("✿ {}", t("Répartition des aimants"))).strong()).default_open(true).show(ui, |ui| {
            egui::Grid::new("g_phi").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                if crate::repartition::uses_offsets() {
                    let fa = units.factor(Q::Angle);
                    let mut s = self.spec.skew_max_deg * fa;
                    lbl(ui, t("Amplitude max des décalages"));
                    if ui.add(egui::DragValue::new(&mut s).speed(0.02 * fa).clamp_range(0.0..=15.0 * fa).max_decimals(4).suffix(format!(" {}", units.sym(Q::Angle)))).changed() {
                        self.spec.skew_max_deg = s / fa;
                        ch = true;
                    }
                    ui.end_row();
                }
                lbl(ui, t("Balourd"));
                let bal = geometry::balourd_mg_cm(&self.spec);
                let okb = crate::repartition::criteria().balourd_max_mg_cm.is_none_or(|m| bal < m);
                ui.label(RichText::new(format!("{:.4} {}", units.show(Q::Unbalance, bal), units.sym(Q::Unbalance))).color(badge(okb)));
                ui.end_row();
                lbl(ui, t("Écart min entre aimants"));
                let gap = geometry::magnet_gaps_deg(&self.spec).into_iter().fold(f64::MAX, f64::min);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{gap:+.3}°")).color(badge(gap >= 0.0)).strong()).on_hover_text(t("Négatif : deux aimants voisins se chevauchent — géométrie irréalisable"));
                    if gap < 0.0 && ui.small_button(t("Ajuster l'arc")).on_hover_text(t("Réduit l'arc des aimants au maximum sans chevauchement (marge 0,1°)")).clicked() {
                        self.spec.magnet_arc_ratio = (geometry::max_arc_ratio(&self.spec, 0.1) * 1000.0).floor() / 1000.0;
                        ch = true;
                    }
                });
                ui.end_row();
                lbl(ui, t("Autocorrélation max"));
                match geometry::autocorr_max(&self.spec) {
                    Some(a) => ui.label(RichText::new(format!("{a:.3}")).color(badge(a < 0.30))),
                    None => ui.label(RichText::new(t("— (périodique)")).weak()),
                };
                ui.end_row();
            });
            let mut custom = self.spec.custom_angles_deg.is_some();
            if chk(ui, &mut custom, t("Positions personnalisées (éditer chaque aimant)")).changed() {
                self.spec.custom_angles_deg = if custom {
                    let mut s = self.spec.clone();
                    s.custom_angles_deg = None;
                    Some(s.magnet_angles_deg())
                } else {
                    None
                };
                ch = true;
            }
            if let Some(angles) = &mut self.spec.custom_angles_deg {
                let n = angles.len();
                let pitch = 360.0 / n as f64;
                egui::Grid::new("g_angles").num_columns(3).striped(true).show(ui, |ui| {
                    ui.label(RichText::new("#").weak());
                    ui.label(RichText::new(t("Centre")).weak());
                    ui.label(RichText::new(t("Écart / régulier")).weak());
                    ui.end_row();
                    for (k, a) in angles.iter_mut().enumerate() {
                        ui.label(format!("{} {}", k + 1, if k % 2 == 0 { "N" } else { "S" }));
                        ch |= ui.add(egui::DragValue::new(a).speed(0.01).max_decimals(4).suffix("°")).changed();
                        ui.label(RichText::new(format!("{:+.4}°", *a - k as f64 * pitch)).monospace());
                        ui.end_row();
                    }
                });
                if ui.button(t("Rééquilibrer (balourd nul, Newton)")).clicked() {
                    rebalance(angles);
                    ch = true;
                }
            }
        });

        egui::CollapsingHeader::new(RichText::new(format!("⟳ {}", t("Bobinage et vitesse"))).strong()).default_open(false).show(ui, |ui| {
            egui::Grid::new("g_wind").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                lbl(ui, t("Spires par bobine"));
                ch |= ui.add(egui::DragValue::new(&mut self.spec.turns_per_coil).clamp_range(1.0..=500.0).speed(0.2)).changed();
                ui.end_row();
                let fs = units.factor(Q::Speed);
                let mut w = self.spec.rpm * fs;
                lbl(ui, t("Vitesse"));
                if ui.add(egui::DragValue::new(&mut w).speed(10.0 * fs).clamp_range(1.0..=200_000.0 * fs).suffix(format!(" {}", units.sym(Q::Speed)))).changed() {
                    self.spec.rpm = w / fs;
                    ch = true;
                }
                ui.end_row();
            });
        });

        egui::CollapsingHeader::new(RichText::new(format!("⚡ {}", t("Bobinage général et charge"))).strong()).default_open(false).show(ui, |ui| {
            egui::Grid::new("g_load").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                lbl(ui, t("Courant de phase crête"));
                ch |= ui.add(egui::DragValue::new(&mut self.spec.load.i_peak).clamp_range(0.0..=1e5).speed(0.1).suffix(" A")).changed();
                ui.end_row();
                lbl(ui, t("Angle du courant / FEM"));
                ch |= ui.add(egui::DragValue::new(&mut self.spec.load.angle_deg).clamp_range(-180.0..=180.0).speed(0.5).suffix(" °él")).changed();
                ui.end_row();
                lbl(ui, t("Voies en parallèle"));
                ch |= ui.add(egui::DragValue::new(&mut self.spec.load.parallel_paths).clamp_range(1.0..=64.0).speed(0.1)).changed();
                ui.end_row();
                lbl(ui, t("Coefficient de remplissage"));
                ch |= ui.add(egui::DragValue::new(&mut self.spec.load.fill_factor).clamp_range(0.05..=0.9).speed(0.005)).changed();
                ui.end_row();
                ui.label(t("Perméance des têtes de bobines λ")).on_hover_text(t("0 = têtes ignorées (comme SIMUMSAP) ; 0,3 à 0,5 usuel pour des bobines sur dent"));
                ch |= ui.add(egui::DragValue::new(&mut self.spec.load.end_lambda).clamp_range(0.0..=2.0).speed(0.01)).changed();
                ui.end_row();
            });
            let auto = self.spec.coils.is_empty();
            ui.horizontal(|ui| {
                ui.label(RichText::new(if auto { t("Bobinage : concentré automatique") } else { t("Bobinage : défini bobine par bobine") }).weak());
                if auto && ui.small_button(t("Éditer bobine par bobine")).clicked() {
                    self.spec.coils = crate::subdomain::effective_coils(&self.spec);
                    ch = true;
                }
                if !auto && ui.small_button(t("Revenir à l'automatique")).clicked() {
                    self.spec.coils.clear();
                    ch = true;
                }
            });
            if !self.spec.coils.is_empty() {
                let q = self.spec.n_slots;
                egui::Grid::new("g_coils").num_columns(6).striped(true).spacing([6.0, 2.0]).show(ui, |ui| {
                    for h in [t("Phase"), t("Aller"), t("Retour"), t("Spires"), t("Côtés"), ""] {
                        ui.label(RichText::new(h).small().strong());
                    }
                    ui.end_row();
                    let mut del = None;
                    for (k, c) in self.spec.coils.iter_mut().enumerate() {
                        egui::ComboBox::from_id_source(("ph", k)).width(40.0).selected_text(["A", "B", "C"][c.phase.min(2)]).show_ui(ui, |ui| {
                            for (i, n) in ["A", "B", "C"].iter().enumerate() {
                                ch |= ui.selectable_value(&mut c.phase, i, *n).changed();
                            }
                        });
                        let mut g1 = c.go + 1;
                        if ui.add(egui::DragValue::new(&mut g1).clamp_range(1..=q)).changed() {
                            c.go = g1 - 1;
                            ch = true;
                        }
                        let mut r1 = c.ret + 1;
                        if ui.add(egui::DragValue::new(&mut r1).clamp_range(1..=q)).changed() {
                            c.ret = r1 - 1;
                            ch = true;
                        }
                        ch |= ui.add(egui::DragValue::new(&mut c.turns).clamp_range(0.0..=1e4).speed(0.2)).changed();
                        let sides = ["G", "D", "E"];
                        ui.label(format!("{}/{}", sides[(c.go_side as usize).min(2)], sides[(c.ret_side as usize).min(2)]))
                            .on_hover_text(t("G = moitié gauche, D = moitié droite, E = encoche entière"));
                        if ui.small_button("✖").clicked() {
                            del = Some(k);
                        }
                        ui.end_row();
                    }
                    if let Some(k) = del {
                        self.spec.coils.remove(k);
                        ch = true;
                    }
                });
                if ui.small_button(format!("+ {}", t("Ajouter une bobine"))).clicked() {
                    self.spec.coils.push(crate::geometry::CoilDef { phase: 0, go: 0, ret: 1, turns: self.spec.turns_per_coil, go_side: 2, ret_side: 2 });
                    ch = true;
                }
            }
        });

        egui::CollapsingHeader::new(RichText::new(format!("🔩 {}", t("Matériaux du fer et saturation"))).strong()).default_open(false).show(ui, |ui| {
            let sat = &mut self.spec.saturation;
            egui::Grid::new("g_sat").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                for (label, m) in [(t("Matériau du stator"), &mut sat.stator), (t("Matériau de la culasse du rotor"), &mut sat.rotor)] {
                    lbl(ui, label);
                    let before = *m;
                    egui::ComboBox::from_id_source(label).selected_text(t(m.name())).show_ui(ui, |ui| {
                        for x in crate::saturation::ALL_MATS {
                            ui.selectable_value(m, x, t(x.name()));
                        }
                    });
                    ch |= before != *m;
                    ui.end_row();
                }
                lbl(ui, t("Foisonnement des tôles"));
                ch |= ui.add(egui::DragValue::new(&mut sat.stacking).clamp_range(0.5..=1.0).speed(0.005)).changed();
                ui.end_row();
            });
            ui.label(RichText::new(t("Courbes B(H) typiques ; saturation estimée par la méthode du facteur de saturation (onglet Charge).")).small().weak());
        });
        egui::CollapsingHeader::new(RichText::new(format!("🔥 {}", t("Frottements, fer et thermique"))).strong()).default_open(false).show(ui, |ui| {
            // ces paramètres n'exigent pas de recalcul magnétique : effet immédiat
            let d = &mut self.spec.dynamics;
            egui::Grid::new("g_dyn").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                let row = |ui: &mut egui::Ui, label: &str, v: &mut f64, lo: f64, hi: f64, speed: f64, suffix: &str| {
                    lbl(ui, label);
                    ui.add(egui::DragValue::new(v).clamp_range(lo..=hi).speed(speed).max_decimals(5).suffix(suffix));
                    ui.end_row();
                };
                row(ui, t("Épaisseur des tôles"), &mut d.lam_thick_mm, 0.05, 1.0, 0.01, " mm");
                row(ui, t("Coefficient d'hystérésis kh"), &mut d.kh, 0.001, 0.1, 0.0005, " W/(kg·Hz·T²)");
                row(ui, t("Conductivité des tôles"), &mut d.sigma_iron, 1e5, 1e7, 1e4, " S/m");
                row(ui, t("Roulements : diamètre moyen"), &mut d.bearing_dm_mm, 1.0, 100.0, 0.1, " mm");
                row(ui, t("Roulements : frottement de charge μ"), &mut d.bearing_mu, 0.0005, 0.01, 0.0001, "");
                row(ui, t("Roulements : coefficient visqueux f0"), &mut d.bearing_f0, 0.5, 6.0, 0.05, "");
                row(ui, t("Viscosité du lubrifiant"), &mut d.bearing_visc_cst, 1.0, 500.0, 0.5, " cSt");
                row(ui, t("Balourd résiduel d'équilibrage"), &mut d.residual_unbalance_mg_cm, 0.0, 500.0, 0.1, " mg·cm");
                row(ui, t("Température ambiante"), &mut d.t_ambient, -40.0, 80.0, 0.5, " °C");
                row(ui, t("Coefficient de Br avec la température"), &mut d.alpha_br_pct, -0.5, 0.0, 0.005, " %/K");
                row(ui, t("Convection naturelle"), &mut d.h_natural, 1.0, 100.0, 0.5, " W/(m²·K)");
                row(ui, t("Raideur radiale des roulements"), &mut d.bearing_stiffness_n_per_um, 0.1, 1000.0, 0.1, " N/µm");
                row(ui, t("Taux d'amortissement"), &mut d.damping_ratio, 0.001, 0.5, 0.001, "");
                row(ui, t("Inertie de la charge (hélice)"), &mut d.load_inertia, 0.0, 1e-2, 1e-7, " kg·m²");
            });
        });

        egui::CollapsingHeader::new(RichText::new(format!("☰ {}", t("Précision du calcul"))).strong()).default_open(false).show(ui, |ui| {
            egui::Grid::new("g_num").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                lbl(ui, t("Harmoniques d'entrefer"));
                ch |= ui.add(egui::DragValue::new(&mut self.st.res.nh).clamp_range(50..=2000)).changed();
                ui.end_row();
                // nombre effectivement utilisé : relevé automatiquement si la géométrie l'exige
                if let Some(used) = self.res.as_ref().map(|r| r.model.harmonics_used())
                    && used > self.st.res.nh
                {
                    ui.label("");
                    ui.label(RichText::new(format!("{} {used} ({})", t("utilisées :"), t("adapté à la géométrie"))).small().color(ORANGE))
                        .on_hover_text(t("Au moins deux fois le rapport circonférence / ouverture d'encoche, sinon le couple de détente n'est pas convergé (grandes machines à ouvertures étroites)."));
                    ui.end_row();
                }
                lbl(ui, t("Harmoniques par encoche"));
                ch |= ui.add(egui::DragValue::new(&mut self.st.res.nk).clamp_range(4..=120)).changed();
                ui.end_row();
                lbl(ui, t("Points de cogging"));
                ch |= ui.add(egui::DragValue::new(&mut self.st.n_cog).clamp_range(32..=4096)).changed();
                ui.end_row();
                lbl(ui, t("Points de FEM (1 tour)"));
                ch |= ui.add(egui::DragValue::new(&mut self.st.n_emf).clamp_range(72..=4096)).changed();
                ui.end_row();
                lbl(ui, t("Fenêtre de cogging"));
                let mut common = self.st.common_span_deg > 0.0;
                if chk(ui, &mut common, t("commune 360°/Q")).on_hover_text(t("Même fenêtre pour toutes les variantes : comparaison loyale des spectres")).changed() {
                    self.st.common_span_deg = if common { 360.0 / self.spec.n_slots as f64 } else { 0.0 };
                    ch = true;
                }
                ui.end_row();
                if ui.button(t("Rapide")).clicked() {
                    self.st.res = Resolution { nh: 200, nk: 12 };
                    self.st.n_cog = 240;
                    ch = true;
                }
                if ui.button(t("Précis")).clicked() {
                    self.st.res = Resolution { nh: 800, nk: 48 };
                    self.st.n_cog = 1440;
                    ch = true;
                }
                ui.end_row();
            });
        });
        if self.st.common_span_deg > 0.0 {
            self.st.common_span_deg = 360.0 / self.spec.n_slots as f64;
        }
        ch
    }
}
