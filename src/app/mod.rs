//! Interface FractLab : paramètres modifiables, vues multiples (courbes,
//! spectres, coupe 2D, carte de champ, moteur 3D, surfaces 3D), curseurs
//! de mesure, comparaison des variantes, validation, préférences.

use crate::analysis::{self, FieldMap, GapField};
use crate::compute::{self, Job, Msg, Results, Settings, VariantResult};
use crate::geometry::{self, Magnetisation, MotorSpec, Topology};
use crate::i18n::{tr, Lang, T};
use crate::magnets::{self, PieceKind};
use crate::subdomain::{Field, Resolution};
use crate::units::{self, Quantity as Q, Units};
use crate::view3d::{self, Camera, Quad, Surface};
use crate::winding;
use eframe::egui::{self, Color32, RichText, Vec2};
use egui_plot::{Bar, BarChart, Corner, Legend, Line, Plot, PlotPoints, Points, VLine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

mod accueil;
mod axial;
mod champ3d;
mod comparaison_ia;
mod console;
mod criteres;
mod demo;
mod fenetres;
mod fichiers;
mod graphe_ia;
mod historique;
mod menu;
mod onglets_etude;
mod onglets_mesure;
mod onglet_parametrique;
mod parametres;
mod recalage;
mod redaction;
mod vues;
mod widgets;
use widgets::*;

const CYAN: Color32 = Color32::from_rgb(0, 200, 255);
/// Couleur unique des actions principales (Calcul, Comparer, Lancer…).
const ACTION: Color32 = Color32::from_rgb(0, 110, 160);
const ORANGE: Color32 = Color32::from_rgb(255, 140, 30);
const SERIES: [Color32; 6] = [
    Color32::from_rgb(0, 200, 255),
    Color32::from_rgb(255, 140, 30),
    Color32::from_rgb(120, 220, 90),
    Color32::from_rgb(230, 80, 160),
    Color32::from_rgb(170, 130, 255),
    Color32::from_rgb(250, 220, 60),
];

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Thème de l'interface.
pub enum Theme {
    /// Thème sombre.
    Sombre,
    /// Thème clair.
    Clair,
}

#[derive(Clone, Serialize, Deserialize)]
/// Préférences mémorisées (~/.config/fractlab/config.json).
pub struct Config {
    /// Langue de l'interface.
    pub lang: Lang,
    /// Thème.
    pub theme: Theme,
    /// Unités d'affichage.
    pub units: Units,
    #[serde(default)]
    /// Dernier projet ouvert ou enregistré.
    pub last_project: Option<String>,
    /// facteur de taille de l'interface (0 = automatique selon l'écran)
    #[serde(default)]
    pub ui_scale: f32,
    /// le champ 3D à travers le moteur a déjà été essayé (le badge « NOUVEAU » disparaît)
    #[serde(default)]
    pub seen_field3d: bool,
    /// projets récents (chemins)
    #[serde(default)]
    pub recent: Vec<String>,
    /// l'accueil du premier lancement a été vu (« ne plus afficher »)
    #[serde(default)]
    pub welcomed: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config { lang: Lang::Fr, theme: Theme::Sombre, units: Units::default(), last_project: None, ui_scale: 0.0, seen_field3d: false, recent: Vec::new(), welcomed: false }
    }
}



impl Config {
    fn load() -> Config {
        crate::platform::load_kv("config").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    fn save(&self) {
        crate::platform::save_kv("config", &serde_json::to_string_pretty(self).unwrap_or_default());
    }
}

/// Motif d'une demande de fichier (la réponse arrive par la boîte aux lettres
/// de `platform`, tout de suite sur ordinateur, plus tard dans le navigateur).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
enum Pick {
    Project = 1,
    Simumsap,
    Figures,
    Angles,
    PhiJson,
    BenchCogging,
    BenchEmf,
    Measure,
}

impl Pick {
    fn from(v: u32) -> Option<Pick> {
        [Pick::Project, Pick::Simumsap, Pick::Figures, Pick::Angles, Pick::PhiJson, Pick::BenchCogging, Pick::BenchEmf, Pick::Measure].into_iter().find(|p| *p as u32 == v)
    }
    fn ask(self, filter: &str, exts: &[&str]) {
        crate::platform::pick(self as u32, filter, exts, self == Pick::Figures);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Tab {
    Synthese,
    Geometrie,
    Champ,
    Cogging,
    Fem,
    Forces,
    Charge,
    Dynamique,
    Comparaison,
    Moteur3d,
    Carte3d,
    Validation,
    Parametrique,
    /// Ajustement I.A (réglages, propositions).
    Ia,
    /// Console : moteur I.A et script.
    Console,
    /// Comparaison des solutions de l'Ajustement I.A.
    Solutions,
}

#[derive(Default, Clone, Copy)]
struct Cursors {
    a: Option<f64>,
    b: Option<f64>,
}

#[derive(Clone, Copy)]
struct View2d {
    zoom: f32,
    pan: Vec2,
}
impl Default for View2d {
    fn default() -> Self {
        View2d { zoom: 1.0, pan: Vec2::ZERO }
    }
}

/// État complet de l'application.
pub struct App {
    cfg: Config,
    spec: MotorSpec,
    st: Settings,
    auto: bool,
    dirty: bool,
    edit_time: f64,
    job: Option<Job>,
    cmp_job: Option<Job>,
    surf_job: Option<Job>,
    val_job: Option<Job>,
    res: Option<Results>,
    err: Option<String>,
    cmp: Vec<VariantResult>,
    surfaces: HashMap<String, Surface>,
    surf_kind: String,
    amp_max: f64,
    tab: Tab,
    theta_r: f64,
    animate: bool,
    field_cache: Option<(f64, Field, GapField)>,
    map_cache: Option<(f64, FieldMap)>,
    /// flux axial : tranche affichée dans l'onglet Champ (None = tranche de référence)
    axial_slice: Option<usize>,
    /// flux axial : mode de l'onglet Champ (0 courbes, 1 carte de la tranche, 2 disque)
    axial_champ_mode: u8,
    /// flux axial : caches (position, tranche, carte) et (position, carte du disque)
    slice_map_cache: Option<(f64, usize, FieldMap)>,
    disc_cache: Option<(f64, Vec<(f64, f64, Vec<f64>)>)>,
    slice_view: View2d,
    disc_view: View2d,
    cam_motor: Camera,
    /// cotes propres à chaque type de machine (rayon intérieur stator,
    /// profondeur d'encoche), retrouvées quand on revient à ce type
    topo_dims: [Option<(f64, f64)>; 3],
    cam_surf: Camera,
    wire: bool,
    cursors: HashMap<&'static str, Cursors>,
    show_prefs: bool,
    show_about: bool,
    /// fenêtre du développeur (clic sur un logo)
    show_author: bool,
    show_help: bool,
    show_license: bool,
    /// rédaction : fenêtre, document, source LaTeX, PDF (octets, moteur), vue PDF ?
    show_redac: bool,
    doc: Option<crate::doc::Doc>,
    latex_src: String,
    pdf: Option<(Vec<u8>, String)>,
    redac_pdf_view: bool,
    /// langue du document (None = langue de l'interface)
    doc_lang: Option<Lang>,
    /// sauvegarde générale : fenêtre et formats supplémentaires cochés
    show_save: bool,
    printers: Vec<String>,
    printer: String,
    copies: u32,
    save_opts: Vec<(&'static str, bool)>,
    /// excentricité : ε, dynamique ?, courbe calculée
    ecc_eps: f64,
    ecc_dyn: bool,
    ecc_curve: Option<crate::analysis::ForceCurve>,
    /// flux axial : décalage axial et inclinaison (fractions de l'entrefer), résultats
    ax_offset: f64,
    ax_tilt: f64,
    ax_def: Option<crate::analysis::AxialDefects>,
    /// simulation dynamique
    sim_setup: crate::dynsim::SimSetup,
    sim_res: Option<crate::dynsim::SimResult>,
    sim_rx: Option<std::sync::mpsc::Receiver<crate::dynsim::SimResult>>,
    /// Web Worker de la simulation (navigateur)
    sim_worker: Option<crate::compute::WorkerHandle>,
    /// réglage de la dernière simulation lancée (relance automatique si changé)
    sim_last: Option<crate::dynsim::SimSetup>,
    /// étude paramétrique : tâche, paramètre, bornes, points, résultats, sorties tracées
    sweep_job: Option<Job>,
    /// recalage sur mesures
    recal_kind: crate::recalage::MeasKind,
    recal_meas: Vec<[f64; 2]>,
    recal_file: String,
    recal_rpm: f64,
    recal_time_x: bool,
    recal_vars: Vec<(crate::recalage::FitVar, bool)>,
    recal_job: Option<crate::recalage::FitJob>,
    recal_prog: (usize, f64),
    recal_res: Option<crate::recalage::FitResult>,
    recal_vars_run: Vec<crate::recalage::FitVar>,
    sweep_var: crate::sweep::SweepVar,
    sweep_from: f64,
    sweep_to: f64,
    sweep_n: usize,
    sweep_rows: Vec<(f64, Vec<f64>)>,
    sweep_replace: bool,
    /// zoom automatique, choisi une fois d'après la taille de l'écran
    auto_zoom: Option<f32>,
    /// flux axial : aimants du rotor superposés au stator dans la vue de face
    axial_show_rotor: bool,
    /// tableau d'explication du bandeau des critères ouvert
    alerts_open: bool,
    sweep_show: [bool; crate::sweep::OUTPUTS.len()],
    /// une simulation doit être relancée dès que la précédente se termine
    sim_pending: bool,
    /// vues lourdes à recalculer après un nouveau résultat principal
    stale: std::collections::HashSet<Tab>,
    /// cartes 3D à recalculer
    surf_stale: Vec<String>,
    /// l'ajustement applique en direct la meilleure proposition
    opt_live: bool,
    restore_base: bool,
    /// score de la dernière proposition appliquée en direct
    opt_live_score: f64,
    /// réglages libres de l'ajustement en cours
    opt_vars_run: Vec<crate::optimize::Var>,
    /// Ajustement I.A : fenêtre, réglages, tâche, résultats
    show_opt: bool,
    opt_vars: Vec<(crate::optimize::Var, bool)>,
    opt_goal: crate::optimize::Goal,
    opt_budget: usize,
    opt_emf_keep: f64,
    opt_job: Option<crate::optimize::OptJob>,
    opt_base: Option<MotorSpec>,
    opt_top: Vec<crate::optimize::Candidate>,
    opt_hist: Vec<f64>,
    opt_prog: (usize, usize),
    /// interdire la saturation de la culasse du rotor pendant l'ajustement
    opt_sat_limit: bool,
    /// amplitude minimale des décalages imposée à l'ajustement [°]
    opt_skew_min: f64,
    /// pertes Joule au plus : original + x % (None = libres)
    opt_joule_max: Option<f64>,
    /// couple moyen au moins : fraction de l'original (None = libre)
    opt_torque_keep: Option<f64>,
    /// console : moteur I.A et script
    console: console::Console,
    /// démo
    demo: demo::DemoState,
    /// comparaison des solutions de l'Ajustement I.A
    cmpia: comparaison_ia::CmpIa,
    /// annuler / rétablir, projet modifié
    hist: historique::EditHistory,
    /// dernier onglet ouvert dans chaque étape de l'étude
    step_tab: [Option<Tab>; 4],
    /// recherche dans le panneau des paramètres
    param_search: String,
    /// fenêtre d'accueil affichée / case « ne plus afficher »
    welcome: bool,
    welcome_never: bool,
    /// fiche du projet SIMUMSAP ouvert (champs, inductances de référence, dossier)
    simu: Option<(Vec<(String, String)>, Option<Vec<Vec<f64>>>, std::path::PathBuf)>,
    /// figures MATLAB chargées : (fichier, axes)
    figs: Vec<(String, Vec<crate::matfile::Axes>)>,
    show_figs: bool,
    help_sel: usize,
    help_search: String,
    photo_tex: Option<egui::TextureHandle>,
    logo_tex: Option<egui::TextureHandle>,
    geo_view: View2d,
    map_view: View2d,
    map_mode_lines: bool,
    spectrum_db: bool,
    validation: Option<(Vec<(f64, f64, f64)>, f64)>,
    status: String,
    show_ribbon: bool,
    /// moteur 3D : vue transparente, opacité, champ (carte |B|, lignes de flux, vecteurs B)
    vis_transparent: bool,
    vis_opacity: f32,
    vis_field_map: bool,
    vis_field_lines: bool,
    vis_field_vec: bool,
    /// animation 3D : lecture, vitesse physique [tr/min], facteur de ralenti
    anim3d: bool,
    anim_rpm: f64,
    slowmo: f64,
    /// temps physique simulé de l'animation [s]
    sim_time: f64,
    /// coloration des dents : false = phase, true = induction instantanée
    teeth_by_b: bool,
    show_force: bool,
    /// amplification visuelle des déplacements vibratoires
    vib_amp: f64,
    /// son en direct (régénéré à chaque changement de vitesse)
    live_sound: bool,
    sound_rpm: f64,
    sound_child: Option<crate::platform::Sound>,
    /// rampe de vitesse automatique (balayage logarithmique)
    ramp: bool,
    ramp_lo: f64,
    ramp_hi: f64,
    ramp_period: f64,
    ramp_t: f64,
    /// courbes en fonction de la vitesse (clé de cache, données)
    speed_curves: Option<(String, Vec<[f64; 5]>)>,
    show_params: bool,
    /// courbes importées superposées : (nom, points, vue « cogging » ou « fem »)
    overlays: Vec<(String, Vec<[f64; 2]>, &'static str)>,
    applied_zoom: f32,
    /// vues ayant un résultat neuf pas encore consulté (vignettes clignotantes)
    fresh: std::collections::HashSet<Tab>,
    /// vue ouverte en plein écran
    full: Option<Tab>,
}

const ALL_TABS: [(Tab, &str, &str); 16] = [
    (Tab::Synthese, "📋", "Synthèse"),
    (Tab::Geometrie, "◎", "Géométrie"),
    (Tab::Champ, "🌊", "Champ"),
    (Tab::Cogging, "〰", "Cogging"),
    (Tab::Fem, "⚡", "FEM"),
    (Tab::Forces, "↗", "Forces"),
    (Tab::Charge, "⚙", "Charge"),
    (Tab::Dynamique, "⏱", "Dynamique"),
    (Tab::Comparaison, "📊", "Comparaison"),
    (Tab::Moteur3d, "💠", "Moteur 3D"),
    (Tab::Carte3d, "🗻", "Cartes 3D"),
    (Tab::Validation, "✔", "Validation"),
    (Tab::Parametrique, "📈", "Paramétrique"),
    (Tab::Ia, "✨", "Ajustement I.A"),
    (Tab::Console, "🖥", "Console"),
    (Tab::Solutions, "⚖", "Comparaison des solutions"),
];

/// Étapes d'une étude : (nom, icône, onglets).
const STEPS: [(&str, &str, &[Tab]); 4] = [
    ("Conception", "✏", &[Tab::Synthese, Tab::Geometrie, Tab::Moteur3d]),
    ("Analyse", "🔬", &[Tab::Champ, Tab::Cogging, Tab::Fem, Tab::Forces, Tab::Charge, Tab::Dynamique, Tab::Carte3d]),
    ("Optimisation", "✨", &[Tab::Ia, Tab::Console, Tab::Solutions, Tab::Parametrique, Tab::Comparaison]),
    ("Validation", "✔", &[Tab::Validation]),
];

fn tab_from_name(n: &str) -> Option<Tab> {
    let n = n.to_lowercase();
    ALL_TABS.iter().find(|(t, _, _)| format!("{t:?}").to_lowercase() == n).map(|x| x.0)
}

fn load_png(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<egui::TextureHandle> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    Some(ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), egui::TextureOptions::LINEAR))
}


/// Applique le thème (couleurs, arrondis, espacements).
pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let dark = theme == Theme::Sombre;
    view3d::DARK.store(dark, std::sync::atomic::Ordering::Relaxed);
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    if dark {
        v.panel_fill = Color32::from_rgb(14, 19, 32);
        v.window_fill = Color32::from_rgb(18, 25, 42);
        v.extreme_bg_color = Color32::from_rgb(8, 11, 20);
        v.faint_bg_color = Color32::from_rgb(22, 30, 50);
        v.selection.bg_fill = Color32::from_rgb(0, 120, 170);
        v.hyperlink_color = ORANGE;
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0f32, CYAN);
    } else {
        v.panel_fill = Color32::from_rgb(246, 248, 252);
        v.window_fill = Color32::from_rgb(255, 255, 255);
        v.faint_bg_color = Color32::from_rgb(234, 240, 248);
        v.selection.bg_fill = Color32::from_rgb(140, 210, 245);
        v.hyperlink_color = Color32::from_rgb(200, 90, 0);
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0f32, Color32::from_rgb(0, 140, 200));
    }
    v.window_rounding = egui::Rounding::same(10.0);
    v.widgets.inactive.rounding = egui::Rounding::same(6.0);
    v.widgets.hovered.rounding = egui::Rounding::same(6.0);
    v.widgets.active.rounding = egui::Rounding::same(6.0);
    ctx.set_visuals(v);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    ctx.set_style(style);
}

impl App {
    /// Crée l'application : préférences, dernier projet, options de lancement.
    pub fn new(cc: &eframe::CreationContext<'_>) -> App {
        let cfg = Config::load();
        let welcomed = cfg.welcomed;
        apply_theme(&cc.egui_ctx, cfg.theme);
        let logo_tex = load_png(&cc.egui_ctx, "logo", include_bytes!("../../assets/logo.png"));
        // dernier projet : fichier sur ordinateur, stockage du navigateur sinon
        let last = if crate::platform::WEB {
            crate::platform::load_kv("projet")
        } else {
            cfg.last_project.as_ref().and_then(|p| std::fs::read_to_string(p).ok())
        };
        // FRACTLAB_PROJET=fichier.json : ouvre ce projet au démarrage (essais, démonstrations)
        let last = std::env::var("FRACTLAB_PROJET").ok().and_then(|p| std::fs::read_to_string(p).ok()).or(last);
        let spec = last.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_else(crate::repartition::default_project);
        let mut app = App {
            cfg,
            spec,
            st: Settings::default(),
            auto: true,
            dirty: true,
            edit_time: -1.0,
            job: None,
            cmp_job: None,
            surf_job: None,
            val_job: None,
            res: None,
            err: None,
            cmp: Vec::new(),
            surfaces: HashMap::new(),
            surf_kind: "cogging_amplitude".into(),
            amp_max: 6.0,
            tab: Tab::Synthese,
            theta_r: 0.0,
            animate: false,
            field_cache: None,
            map_cache: None,
            axial_slice: None,
            axial_champ_mode: if std::env::var("FRACTLAB_MAP").is_ok() { 1 } else { 0 },
            slice_map_cache: None,
            disc_cache: None,
            slice_view: View2d::default(),
            disc_view: View2d::default(),
            topo_dims: [None; 3],
            cam_motor: Camera { yaw: -0.6, pitch: 0.75, zoom: 1.0, pan: Vec2::ZERO },
            cam_surf: Camera::default(),
            wire: false,
            cursors: HashMap::new(),
            show_prefs: false,
            show_about: false,
            show_author: std::env::var("FRACTLAB_AUTEUR").is_ok(),
            show_help: std::env::var("FRACTLAB_HELP").is_ok(),
            show_license: std::env::var("FRACTLAB_LICENSE").is_ok(),
            simu: None,
            show_redac: std::env::var("FRACTLAB_REDAC").is_ok(),
            doc: None,
            latex_src: String::new(),
            pdf: None,
            redac_pdf_view: false,
            doc_lang: None,
            show_save: std::env::var("FRACTLAB_SAVE").is_ok(),
            printers: Vec::new(),
            printer: String::new(),
            copies: 1,
            save_opts: vec![
                ("Coupe 2D (DXF)", true),
                ("Modèle 3D (STL)", true),
                ("Gabarit de pose 1:1 (SVG)", true),
                ("Modèle FEMM (.lua)", true),
                ("Géométrie Gmsh (.geo)", false),
                ("Courbes (CSV)", true),
                ("Rapport HTML", true),
                ("Script Octave / MATLAB (.m)", true),
                ("Script Python (.py)", false),
                ("Rapport (JSON)", true),
                ("Son synthétisé (WAV)", false),
            ],
            ecc_eps: 0.1,
            ecc_dyn: false,
            ecc_curve: None,
            ax_offset: 0.1,
            ax_tilt: 0.1,
            ax_def: None,
            sim_setup: Default::default(),
            sim_res: None,
            sim_rx: None,
            sim_worker: None,
            sim_last: None,
            sweep_job: None,
            recal_kind: crate::recalage::MeasKind::Emf,
            recal_meas: Vec::new(),
            recal_file: String::new(),
            recal_rpm: 1000.0,
            recal_time_x: false,
            recal_vars: vec![(crate::recalage::FitVar::Br, true), (crate::recalage::FitVar::Airgap, true), (crate::recalage::FitVar::Length, false)],
            recal_job: None,
            recal_prog: (0, 0.0),
            recal_res: None,
            recal_vars_run: Vec::new(),
            sweep_var: crate::sweep::SweepVar::Geo(crate::optimize::Var::ArcRatio),
            sweep_from: 0.5,
            sweep_to: 0.98,
            sweep_n: 11,
            sweep_rows: Vec::new(),
            sweep_replace: false,
            auto_zoom: None,
            axial_show_rotor: true,
            alerts_open: std::env::var("FRACTLAB_ALERTES").is_ok(),
            sweep_show: [true, false, true, true, false, true, false, false, false, true, false],
            sim_pending: false,
            stale: Default::default(),
            surf_stale: Vec::new(),
            opt_live: true,
            restore_base: false,
            opt_live_score: f64::INFINITY,
            opt_vars_run: Vec::new(),
            show_opt: std::env::var("FRACTLAB_IA").is_ok_and(|v| v != "console" && v != "compare"),
            opt_vars: crate::optimize::ALL_VARS.iter().filter(|v| **v != crate::optimize::Var::SkewAmp || crate::repartition::uses_offsets()).map(|v| (*v, matches!(v, crate::optimize::Var::ArcRatio | crate::optimize::Var::SkewAmp | crate::optimize::Var::SlotOpen))).collect(),
            opt_goal: crate::optimize::Goal::MinCogging,
            opt_budget: 150,
            opt_emf_keep: 0.9,
            opt_job: None,
            opt_base: None,
            opt_top: Vec::new(),
            opt_hist: Vec::new(),
            opt_prog: (0, 0),
            opt_sat_limit: false,
            opt_joule_max: None,
            opt_torque_keep: None,
            opt_skew_min: if crate::repartition::uses_offsets() { 1.0 } else { 0.0 },
            console: Default::default(),
            cmpia: Default::default(),
            hist: Default::default(),
            step_tab: [None; 4],
            param_search: std::env::var("FRACTLAB_RECHERCHE").unwrap_or_default(),
            // premier lancement (les lancements d'essai FRACTLAB_… ne l'affichent pas)
            welcome: std::env::var("FRACTLAB_ACCUEIL").is_ok() || (!welcomed && !std::env::vars().any(|(k, _)| k.starts_with("FRACTLAB_") && k != "FRACTLAB_PUBLIC" && k != "FRACTLAB_PRIVE")),
            welcome_never: true,
            demo: demo::DemoState {
                pending: std::env::var("FRACTLAB_DEMO").ok().and_then(|k| crate::demo::MOTORS.iter().find(|m| m.key == k).map(|m| m.key)),
                confirm: std::env::var("FRACTLAB_DEMO").as_deref() == Ok("choix"),
                ..Default::default()
            },
            figs: Vec::new(),
            show_figs: false,
            help_sel: std::env::var("FRACTLAB_HELP").ok().and_then(|v| v.parse().ok()).unwrap_or(0),
            help_search: String::new(),
            photo_tex: None,
            logo_tex,
            geo_view: View2d::default(),
            map_view: View2d::default(),
            map_mode_lines: true,
            spectrum_db: false,
            validation: None,
            status: String::new(),
            show_ribbon: true,
            vis_transparent: std::env::var("FRACTLAB_TRANSPARENT").is_ok(),
            vis_opacity: 0.18,
            vis_field_map: std::env::var("FRACTLAB_TRANSPARENT").is_ok(),
            vis_field_lines: std::env::var("FRACTLAB_TRANSPARENT").is_ok(),
            vis_field_vec: std::env::var("FRACTLAB_TRANSPARENT").as_deref() == Ok("tout"),
            anim3d: std::env::var("FRACTLAB_ANIM").is_ok(),
            anim_rpm: 10000.0,
            slowmo: 1e-3,
            sim_time: 0.0,
            teeth_by_b: true,
            show_force: true,
            vib_amp: 1.0,
            live_sound: false,
            sound_rpm: 0.0,
            sound_child: None,
            ramp: std::env::var("FRACTLAB_RAMP").is_ok(),
            ramp_lo: 1000.0,
            ramp_hi: 30000.0,
            ramp_period: 20.0,
            ramp_t: 0.0,
            speed_curves: None,
            show_params: true,
            overlays: Vec::new(),
            applied_zoom: 0.0,
            fresh: Default::default(),
            full: None,
        };
        // options de lancement (démonstrations, captures, tests)
        if let Ok(v) = std::env::var("FRACTLAB_TAB")
            && let Some(t) = tab_from_name(&v) {
                app.tab = t;
            }
        if let Ok(v) = std::env::var("FRACTLAB_FULL") {
            app.full = tab_from_name(&v);
        }
        if let Ok(v) = std::env::var("FRACTLAB_LANG") {
            app.cfg.lang = match v.to_lowercase().as_str() {
                "en" => Lang::En,
                "es" => Lang::Es,
                "de" => Lang::De,
                _ => Lang::Fr,
            };
        }
        if let Ok(v) = std::env::var("FRACTLAB_THEME") {
            app.cfg.theme = if v.to_lowercase().starts_with('c') || v.to_lowercase().starts_with('l') { Theme::Clair } else { Theme::Sombre };
            apply_theme(&cc.egui_ctx, app.cfg.theme);
        }
        app.map_mode_lines = std::env::var("FRACTLAB_MAP").is_ok();
        if std::env::var("FRACTLAB_COMPARE").is_ok() {
            app.cmp_job = Some(compute::run_compare(app.spec.clone(), app.st));
        }
        if std::env::var("FRACTLAB_SURFACE").is_ok() {
            app.surf_job = Some(compute::run_surface_amplitude(app.spec.clone(), app.st, app.amp_max, 25, 91));
        }
        if std::env::var("FRACTLAB_VALIDATE").is_ok() {
            app.val_job = Some(compute::run_validation(app.spec.clone(), app.st, 5760, 8));
        }
        if std::env::var("FRACTLAB_PREFS").is_ok() {
            app.show_prefs = true;
        }
        // démonstration : FRACTLAB_MESURE=fichier.csv (angle [°] ; FEM [V]) lance un recalage
        if let Ok(p) = std::env::var("FRACTLAB_MESURE")
            && let Ok(pts) = std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|s| crate::exchange::import_curve_csv(&s))
        {
            app.recal_meas = pts;
            app.recal_rpm = app.spec.rpm;
            app.recal_file = p;
            app.recal_vars_run = vec![crate::recalage::FitVar::Br, crate::recalage::FitVar::Airgap];
            let setup = crate::recalage::FitSetup { kind: crate::recalage::MeasKind::Emf, meas: app.recal_meas.clone(), meas_rpm: app.recal_rpm, vars: app.recal_vars_run.clone(), budget: 60 };
            app.recal_job = Some(crate::recalage::run(app.spec.clone(), setup));
        }
        // démonstration : FRACTLAB_IA=run lance l'ajustement en direct
        if matches!(std::env::var("FRACTLAB_IA").as_deref(), Ok("run") | Ok("console") | Ok("compare")) {
            let vars: Vec<crate::optimize::Var> = app.opt_vars.iter().filter(|(_, on)| *on).map(|(v, _)| *v).collect();
            app.opt_base = Some(app.spec.clone());
            app.opt_vars_run = vars.clone();
            app.console_ia_start(&vars);
            app.opt_job = Some(crate::optimize::run(app.spec.clone(), crate::optimize::Setup { vars, goal: app.opt_goal, budget: app.opt_budget, emf_keep: app.opt_emf_keep, sat_limit: app.opt_sat_limit, skew_min: app.opt_skew_min, joule_max_pct: app.opt_joule_max, torque_keep: app.opt_torque_keep }));
        }
        if std::env::var("FRACTLAB_SWEEP").is_ok() {
            app.start_sweep();
        }
        if let Ok(v) = std::env::var("FRACTLAB_THETA") {
            app.theta_r = v.parse().unwrap_or(0.0);
        }
        app.hist_reset();
        app.launch();
        app
    }

    fn t(&self, k: &'static str) -> &'static str {
        T(self.cfg.lang, k)
    }
    fn u(&self) -> &Units {
        &self.cfg.units
    }

    fn launch(&mut self) {
        if crate::platform::WEB {
            // navigateur : le projet en cours est gardé pour la prochaine visite
            crate::platform::save_kv("projet", &serde_json::to_string(&self.spec).unwrap_or_default());
        }
        self.job = Some(compute::run_main(self.spec.clone(), self.st));
        self.dirty = false;
        self.status = self.t("Calcul en cours…").into();
    }

    fn poll(&mut self, ctx: &egui::Context) {
        let mut msgs = Vec::new();
        for j in [&mut self.job, &mut self.cmp_job, &mut self.surf_job, &mut self.val_job, &mut self.sweep_job] {
            if let Some(job) = j {
                while let Ok(m) = job.rx.try_recv() {
                    let done = !matches!(m, Msg::Progress(_) | Msg::SweepRow(..));
                    msgs.push(m);
                    if done {
                        *j = None;
                        break;
                    }
                }
            }
        }
        for m in msgs {
            match m {
                Msg::Main(Ok(r)) => {
                    self.status = format!("{} {:.2} s", self.t("Calcul terminé en"), r.seconds);
                    self.err = None;
                    self.res = Some(r);
                    self.field_cache = None;
                    self.map_cache = None;
                    self.slice_map_cache = None;
                    self.disc_cache = None;
                    for t in [Tab::Synthese, Tab::Geometrie, Tab::Champ, Tab::Cogging, Tab::Fem, Tab::Forces, Tab::Charge, Tab::Moteur3d] {
                        self.fresh.insert(t);
                    }
                    self.propagate();
                }
                Msg::Main(Err(e)) => {
                    self.err = Some(e);
                    self.status = self.t("Erreur de calcul").into();
                }
                Msg::Compare(v) => {
                    self.cmp = v;
                    self.fresh.insert(Tab::Comparaison);
                    self.status = self.t("Comparaison terminée").into();
                }
                Msg::Surface(k, s) => {
                    self.surfaces.insert(k, s);
                    self.fresh.insert(Tab::Carte3d);
                    self.status = self.t("Carte 3D prête").into();
                }
                Msg::Progress(p) => self.status = p,
                Msg::SweepRow(v, o) => {
                    if self.sweep_replace {
                        self.sweep_replace = false;
                        self.sweep_rows.clear();
                    }
                    self.sweep_rows.push((v, o));
                    self.sweep_rows.sort_by(|a, b| a.0.total_cmp(&b.0));
                    self.fresh.insert(Tab::Parametrique);
                }
                Msg::SweepDone => self.status = self.t("Étude paramétrique terminée").into(),
                // la comparaison de solutions a sa propre tâche (fenêtre dédiée)
                Msg::Solution(..) | Msg::SolutionError(..) | Msg::SolutionsDone => {}
                Msg::Validation(rows, open) => {
                    self.validation = Some((rows, open));
                    self.fresh.insert(Tab::Validation);
                    self.status = self.t("Validation terminée").into();
                }
            }
        }
        self.poll_followers();
        let busy = self.job.is_some() || self.cmp_job.is_some() || self.surf_job.is_some() || self.val_job.is_some() || self.sim_rx.is_some() || self.sweep_job.is_some() || self.recal_job.is_some();
        if busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
        }
        // calcul automatique après une courte pause de saisie
        let now = ctx.input(|i| i.time);
        if self.dirty && self.auto && self.job.is_none() && now - self.edit_time > 0.35 {
            self.launch();
        } else if self.dirty && self.auto {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
        }
    }

    /// Propage un nouveau résultat principal à toutes les vues : excentricité
    /// recalculée tout de suite, simulation dynamique relancée, vues lourdes
    /// (comparaison, cartes 3D, validation) remises en file si elles ont déjà
    /// été calculées.
    fn propagate(&mut self) {
        if self.ecc_curve.is_some()
            && let Some(rr) = &self.res
        {
            self.ecc_curve = Some(crate::analysis::forces_eccentric(&rr.model, self.ecc_eps, self.ecc_dyn, 120));
        }
        if self.ax_def.is_some()
            && let Some(rr) = &self.res
        {
            self.ax_def = Some(crate::analysis::axial_defects(&rr.model, self.ax_offset, self.ax_tilt, 8));
        }
        self.start_sim();
        if !self.cmp.is_empty() || self.cmp_job.is_some() {
            self.stale.insert(Tab::Comparaison);
        }
        let kinds: Vec<String> = self.surfaces.keys().cloned().collect();
        for k in kinds {
            if !self.surf_stale.contains(&k) {
                self.surf_stale.push(k);
            }
        }
        if self.validation.is_some() || self.val_job.is_some() {
            self.stale.insert(Tab::Validation);
        }
        if !self.sweep_rows.is_empty() || self.sweep_job.is_some() {
            self.stale.insert(Tab::Parametrique);
        }
    }

    /// Lance la simulation dynamique sur le résultat courant (ou la met en
    /// attente si une simulation tourne déjà).
    fn start_sim(&mut self) {
        let Some(r) = &self.res else { return };
        if self.sim_rx.is_some() {
            self.sim_pending = true;
            return;
        }
        let setup = self.sim_setup.clone();
        let (rx, w) = crate::dynsim::run(r, setup);
        self.sim_worker = w;
        self.sim_last = Some(self.sim_setup.clone());
        self.sim_rx = Some(rx);
    }

    fn launch_surface(&mut self, kind: &str) {
        self.surf_job = Some(if kind == "br" {
            compute::run_surface_br(self.spec.clone(), self.st, 180, 60, 360.0 / self.spec.n_slots as f64)
        } else {
            compute::run_surface_amplitude(self.spec.clone(), self.st, self.amp_max, 25, 91)
        });
    }

    /// Réception de la simulation et relance des vues dépendantes quand le
    /// calcul principal est à jour.
    fn poll_followers(&mut self) {
        if let Some(j) = &self.recal_job {
            let mut done = None;
            while let Ok(m) = j.rx.try_recv() {
                match m {
                    crate::recalage::FitMsg::Progress(n, e) => self.recal_prog = (n, e),
                    crate::recalage::FitMsg::Done(r) => done = Some(r),
                }
            }
            if let Some(r) = done {
                self.recal_job = None;
                match r {
                    Ok(r) => {
                        self.status = format!("{} : {:.2} % ➡ {:.2} %", self.t("Recalage terminé"), r.err_initial, r.err_fitted);
                        self.recal_res = Some(r);
                        self.fresh.insert(Tab::Validation);
                    }
                    Err(e) => self.err = Some(e),
                }
            }
        }
        if let Some(rx) = &self.sim_rx
            && let Ok(r) = rx.try_recv()
        {
            self.sim_res = Some(r);
            self.sim_rx = None;
            self.fresh.insert(Tab::Dynamique);
            if self.sim_pending {
                self.sim_pending = false;
                self.start_sim();
            }
        }
        // réglages de la simulation modifiés : relance automatique
        if self.res.is_some() && self.sim_rx.is_none() && self.sim_last.as_ref() != Some(&self.sim_setup) {
            self.start_sim();
        }
        // vues lourdes : seulement quand le projet est stable (pas de saisie,
        // pas de calcul principal ni d'ajustement en cours)
        if self.job.is_some() || self.dirty || self.opt_job.is_some() {
            return;
        }
        if self.stale.contains(&Tab::Comparaison) && self.cmp_job.is_none() {
            self.stale.remove(&Tab::Comparaison);
            self.cmp_job = Some(compute::run_compare(self.spec.clone(), self.st));
        }
        if self.surf_job.is_none()
            && let Some(k) = self.surf_stale.pop()
        {
            self.launch_surface(&k);
        }
        if self.stale.contains(&Tab::Parametrique) && self.sweep_job.is_none() {
            self.stale.remove(&Tab::Parametrique);
            self.start_sweep();
        }
        if self.stale.contains(&Tab::Validation) && self.val_job.is_none() {
            self.stale.remove(&Tab::Validation);
            self.val_job = Some(compute::run_validation(self.spec.clone(), self.st, 5760, 8));
        }
    }

    fn touched(&mut self, ctx: &egui::Context) {
        self.dirty = true;
        self.edit_time = ctx.input(|i| i.time);
        self.hist_touch();
    }

}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for (purpose, files) in crate::platform::take_opened() {
            self.on_opened(purpose, files, ctx);
        }
        self.poll(ctx);
        self.hist_tick(ctx);
        if self.animate {
            let dt = ctx.input(|i| i.stable_dt).min(0.05) as f64;
            self.theta_r = (self.theta_r + dt * 20.0).rem_euclid(360.0);
            ctx.request_repaint();
        }
        if self.ramp {
            // balayage logarithmique aller-retour entre deux vitesses
            self.ramp_t += ctx.input(|i| i.stable_dt).min(0.05) as f64;
            let u = 0.5 - 0.5 * (2.0 * PI * self.ramp_t / self.ramp_period.max(1.0)).cos();
            self.anim_rpm = self.ramp_lo * (self.ramp_hi / self.ramp_lo.max(1.0)).powf(u);
            ctx.request_repaint();
        }
        self.live_sound_tick();
        if self.anim3d {
            // temps physique = temps écran × ralenti ; θ avance à la vitesse réelle
            let dt = ctx.input(|i| i.stable_dt).min(0.05) as f64 * self.slowmo;
            self.sim_time += dt;
            self.theta_r = (self.theta_r + self.anim_rpm * 6.0 * dt).rem_euclid(360.0);
            ctx.request_repaint();
        }
        self.apply_zoom(ctx);
        crate::i18n::set_lang(self.cfg.lang);
        self.demo_tick(ctx);
        self.menu_bar(ctx);
        // demandes d'ouverture (boutons, menus, fin d'ajustement) : l'onglet correspondant
        for (want, tab) in [(std::mem::take(&mut self.show_opt), Tab::Ia), (std::mem::take(&mut self.console.open), Tab::Console), (std::mem::take(&mut self.cmpia.open), Tab::Solutions)] {
            if want {
                self.tab = tab;
                self.full = None;
            }
        }
        if self.show_params && self.full.is_none() {
            self.params_panel(ctx);
        }
        self.central(ctx);
        if self.show_prefs {
            self.prefs_window(ctx);
        }
        if self.show_about {
            self.about_window(ctx);
        }
        if self.show_author {
            self.author_window(ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_help = !self.show_help;
        }
        if self.show_help {
            self.help_window(ctx);
        }
        if self.show_license {
            self.license_window(ctx);
        }
        self.poll_opt(ctx);
        self.poll_script(ctx);
        self.demo_windows(ctx);
        self.cmpia_poll(ctx);
        self.changes_window(ctx);
        self.welcome_window(ctx);
        if self.show_redac {
            self.redac_window(ctx);
        }
        if self.show_save {
            self.save_window(ctx);
        }
        if self.simu.is_some() {
            self.simu_window(ctx);
        }
        if self.show_figs {
            self.figs_window(ctx);
        }
    }
}
