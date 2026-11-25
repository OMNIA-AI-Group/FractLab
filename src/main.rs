#![cfg_attr(windows, windows_subsystem = "windows")]
use fractlab::app::App;

/// Lancée depuis l'AppImage : installe (ou met à jour) le lanceur et l'icône
/// dans le dossier de l'utilisateur, pour que le dock et le sélecteur de
/// fenêtres affichent l'icône FractLab (sous Wayland, l'icône vient du lanceur
/// dont le nom correspond à l'identifiant de l'application).
fn integrer_au_bureau() {
    let (Ok(appimage), Some(data)) = (std::env::var("APPIMAGE"), dirs::data_dir()) else { return };
    let icon = data.join("icons/hicolor/256x256/apps/fractlab.png");
    let desk = data.join("applications/fractlab.desktop");
    let contenu = format!(
        "[Desktop Entry]\nType=Application\nName=FractLab\nGenericName=Magnetic Engine Simulator\nGenericName[fr]=Simulateur de moteurs à aimants\nComment=Analyse des machines à aimants permanents\nExec=\"{appimage}\" %F\nIcon=fractlab\nStartupWMClass=fractlab\nCategories=Science;Engineering;Physics;\nTerminal=false\n"
    );
    if std::fs::read_to_string(&desk).ok().as_deref() != Some(contenu.as_str()) {
        let _ = std::fs::create_dir_all(desk.parent().unwrap());
        let _ = std::fs::write(&desk, contenu);
    }
    if !icon.exists() || std::fs::read(&icon).ok().as_deref() != Some(&include_bytes!("../assets/icon.png")[..]) {
        let _ = std::fs::create_dir_all(icon.parent().unwrap());
        let _ = std::fs::write(&icon, include_bytes!("../assets/icon.png"));
    }
}

/// En cas de plantage, la cause est écrite dans `crash.txt` (dossier des
/// réglages de FractLab), visible même sans console (Windows).
fn journal_des_plantages() {
    let defaut = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(d) = dirs::config_dir() {
            let dir = d.join("fractlab");
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(dir.join("crash.txt"), format!("FractLab {} — {}\n{}\n", env!("CARGO_PKG_VERSION"), std::env::consts::OS, info));
        }
        defaut(info);
    }));
}

fn main() -> eframe::Result<()> {
    journal_des_plantages();
    integrer_au_bureau();
    let icon = image::load_from_memory(include_bytes!("../assets/icon.png")).ok().map(|i| {
        let i = i.to_rgba8();
        eframe::egui::IconData { width: i.width(), height: i.height(), rgba: i.into_raw() }
    });
    let mut vp = eframe::egui::ViewportBuilder::default()
        .with_title(format!("{} — Magnetic Engine Simulator", fractlab::repartition::app_name()))
        .with_app_id("fractlab")
        .with_inner_size([1360.0, 820.0])
        .with_min_inner_size([900.0, 560.0])
        .with_maximized(true);
    if let Some(i) = icon {
        vp = vp.with_icon(i);
    }
    let opts = eframe::NativeOptions { viewport: vp, ..Default::default() };
    let r = eframe::run_native("FractLab", opts, Box::new(|cc| Box::new(App::new(cc))));
    // échec au démarrage (pilote graphique OpenGL…) : cause écrite dans crash.txt
    if let (Err(e), Some(d)) = (&r, dirs::config_dir()) {
        let dir = d.join("fractlab");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("crash.txt"), format!("FractLab {} — {}\nÉchec au démarrage : {e}\n", env!("CARGO_PKG_VERSION"), std::env::consts::OS));
    }
    r
}
