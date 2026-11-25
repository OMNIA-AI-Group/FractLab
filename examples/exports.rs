//! Génère tous les exports pour la variante donnée (contrôle par outils externes).
use fractlab::compute::{self, Msg, Settings};
use fractlab::exchange as x;
use fractlab::geometry::MotorSpec;

fn main() {
    let out = std::env::args().nth(1).expect("dossier de sortie");
    let v = std::env::args().nth(2).unwrap_or("irregulier".into());
    std::fs::create_dir_all(&out).unwrap();
    let spec = MotorSpec::example(&v);
    let job = compute::run_main(spec.clone(), Settings::default());
    let Msg::Main(Ok(r)) = job.rx.recv().unwrap() else { panic!("calcul") };
    let p = |f: &str| format!("{out}/{f}");
    std::fs::write(p("modele.lua"), x::femm_lua(&spec, 30.0, 0.25)).unwrap();
    std::fs::write(p("modele.geo"), x::gmsh_geo(&spec)).unwrap();
    std::fs::write(p("coupe.dxf"), x::dxf(&spec)).unwrap();
    std::fs::write(p("gabarit.svg"), x::svg_gabarit(&spec)).unwrap();
    for part in ["stator", "aimants", "cloche"] {
        std::fs::write(p(&format!("{part}.stl")), x::stl(&spec, part)).unwrap();
    }
    std::fs::write(p("tableau.tex"), x::latex(&r, &[])).unwrap();
    std::fs::write(p("rapport.html"), x::html_report(&r, &[])).unwrap();
    std::fs::write(p("donnees.m"), x::octave_script(&r)).unwrap();
    std::fs::write(p("donnees.py"), x::python_script(&r)).unwrap();
    println!("ok");
}
