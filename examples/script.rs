//! Exécute un script de console FractLab hors interface :
//! `cargo run --release --example script -- fichier.txt [projet.json]`
//! (sans fichier : le script est lu sur l'entrée standard). Le projet de
//! départ est le projet par défaut, ou le projet JSON donné.

use fractlab::compute::Settings;
use fractlab::geometry::MotorSpec;
use std::io::Read;
use std::sync::atomic::AtomicBool;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let src = match args.first() {
        Some(f) => std::fs::read_to_string(f).expect("script illisible"),
        None => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            s
        }
    };
    let spec = match args.get(1) {
        Some(p) => serde_json::from_str(&std::fs::read_to_string(p).expect("projet illisible")).expect("projet invalide"),
        None => fractlab::repartition::default_project(),
    };
    let stop = AtomicBool::new(false);
    fractlab::script::execute(spec, Settings::default(), &src, &stop, &mut |m| {
        if let fractlab::script::ScriptMsg::Line(_, s) = m {
            println!("{s}");
        }
    });
}
