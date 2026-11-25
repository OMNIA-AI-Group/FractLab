//! Étude de convergence de la tranche de référence d'une machine à flux
//! axial : modèle analytique (harmoniques croissantes) contre volumes finis
//! en géométrie plane (grille raffinée en angle et en cote).
//! Usage : convergence_axiale projet.json
use fractlab::subdomain::{Model, Resolution};
fn main() {
    let spec: fractlab::geometry::MotorSpec = serde_json::from_str(&std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
    let m0 = Model::new(&spec, Resolution::default()).unwrap();
    let k = m0.reference_index();
    let mut sk = spec.clone();
    sk.magnet_arc_ratio = m0.slice_arc_ratio(k);
    let pieces = fractlab::magnets::pieces(&sk);
    let zeros = vec![0.0; spec.n_slots];
    let per = spec.cogging_period_deg();
    for (nt, sc) in [(5760usize, 1usize), (5760, 2), (5760, 3), (11520, 1)] {
        let (fd, ge) = fractlab::fdcheck::FdModel::general(&m0.sections()[k].g, pieces.clone(), spec.br_magnet, nt, sc, &zeros);
        let tf: Vec<f64> = (0..4).map(|i| fd.torque(&fd.solve((per * (i as f64 + 0.5) / 8.0).to_radians())) * 1e3).collect();
        println!("VF nt={nt} raffinement radial ×{sc} : {:?}", tf.iter().map(|v| format!("{v:.2}")).collect::<Vec<_>>());
        for (nh, nk) in [(400, 24), (1200, 32), (2400, 48)] {
            let sec = fractlab::section::Section::new(ge.clone(), fractlab::section::Harmonics { nh, ks: nk, ko: nk }).unwrap();
            let mf = fractlab::magnets::fourier(&pieces, spec.br_magnet, nh);
            let ta: Vec<f64> = (0..4).map(|i| sec.rotor_torque(&sec.solve(&mf.rotated((per * (i as f64 + 0.5) / 8.0).to_radians()), &zeros)) * 1e3).collect();
            println!("   analytique nh={nh} nk={nk} : {:?}", ta.iter().map(|v| format!("{v:.2}")).collect::<Vec<_>>());
        }
    }
}
