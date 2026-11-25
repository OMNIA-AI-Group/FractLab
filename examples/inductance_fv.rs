//! Inductance propre de la phase A : modèle analytique contre volumes finis,
//! tranche par tranche (machines axiales en géométrie plane, radiales en
//! polaire), aimants éteints. Usage :
//! `inductance_fv projet.(json|txt) [nt]` ; OUVERT=1 applique les hypothèses
//! d'encoches ouvertes (ouverture = corps d'encoche, sans becs).
use fractlab::exchange::import_simumsap;
use fractlab::geometry::{MotorSpec, Topology};
use fractlab::subdomain::{Model, Resolution};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let bytes = std::fs::read(&a[1]).expect("projet illisible");
    let mut s: MotorSpec = if a[1].ends_with(".json") { serde_json::from_slice(&bytes).expect("JSON") } else { import_simumsap(&bytes).expect("SIMUMSAP").spec };
    if std::env::var("OUVERT").is_ok() {
        s.tip_h = 0.0;
        s.slot_open = s.slot_w.max(s.slot_open);
    }
    let nt: usize = a.get(2).and_then(|x| x.parse().ok()).unwrap_or(s.n_slots * 80);
    let m = Model::new(&s, Resolution { nh: std::env::var("NH").ok().and_then(|v| v.parse().ok()).unwrap_or(400), nk: std::env::var("NK").ok().and_then(|v| v.parse().ok()).unwrap_or(20) }).unwrap();
    let per = fractlab::geometry::gcd(s.n_slots, s.n_poles / 2) as f64;
    let g = if s.topology == Topology::Axial { s.axial.gaps as f64 } else { 1.0 };
    let i_a = [1.0, 0.0, 0.0];
    let paths = s.load.parallel_paths.max(1.0);
    let (mut l_an, mut l_fv) = (0.0, 0.0);
    println!("tranche   r0 [mm]   L analytique [mH]   L volumes finis [mH]   écart");
    for (k, sec) in m.sections().iter().enumerate() {
        let j = m.slot_currents(sec, &i_a);
        let mut sk = s.clone();
        sk.magnet_arc_ratio = m.slice_arc_ratio(k);
        let pieces = fractlab::magnets::pieces(&sk);
        let (fd, ge) = fractlab::fdcheck::FdModel::general(&sec.g, pieces, 0.0, nt, std::env::var("SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(1), &j);
        // modèle analytique sur la géométrie effectivement maillée
        let sec2 = fractlab::section::Section::new(ge, sec.h).unwrap();
        let zero = fractlab::magnets::fourier(&[], 0.0, sec.h.nh);
        let fa = sec2.solve(&zero, &j);
        let af = fd.solve(0.0);
        let (mut la, mut lf) = (0.0, 0.0);
        for c in m.coils.iter().filter(|c| c.phase == 0) {
            la += c.turns * (sec2.slot_mean_a(&fa, c.go, 2) - sec2.slot_mean_a(&fa, c.ret, 2));
            lf += c.turns * (fd.slot_mean_a(&af, c.go) - fd.slot_mean_a(&af, c.ret));
        }
        let (la, lf) = (la * sec.g.depth * g / paths, lf * sec.g.depth * g / paths);
        println!("{k:>6} {:>9.2} {:>19.5} {:>22.5} {:>+7.2} %", m.slice_radius(k).0 * 1e3, la * 1e3, lf * 1e3, 100.0 * (la / lf - 1.0));
        l_an += la;
        l_fv += lf;
    }
    println!("machine entière : analytique {:.4} mH, volumes finis {:.4} mH, écart {:+.2} %", l_an * 1e3, l_fv * 1e3, 100.0 * (l_an / l_fv - 1.0));
    println!("par période et par entrefer : analytique {:.4} mH, volumes finis {:.4} mH", l_an * 1e3 / per / g, l_fv * 1e3 / per / g);
    let full = m.phase_flux(&m.solve_armature(&i_a))[0];
    println!("contrôle : flux de phase du modèle complet {:.4} mH (entier), {:.4} mH par période et entrefer", full * 1e3, full * 1e3 / per / g);
}
