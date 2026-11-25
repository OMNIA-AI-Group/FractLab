//! Ouvre un projet SIMUMSAP (fichier .txt) et compare FractLab aux résultats
//! de SIMUMSAP : matrice d'inductances du projet et figures .fig passées en
//! arguments. Usage : simumsap_compare projet.txt [figure.fig …] ; OUVERT=1
//! applique les hypothèses d'encoches ouvertes du modèle axial de SIMUMSAP ;
//! PLEINE=1 répartit le courant de chaque bobine sur toute l'encoche (comme SIMUMSAP).
use fractlab::analysis;
use fractlab::exchange::import_simumsap;
use fractlab::subdomain::{Model, Resolution};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let imp = import_simumsap(&std::fs::read(&a[1]).unwrap()).unwrap();
    let mut s = imp.spec.clone();
    if std::env::var("OUVERT").is_ok() {
        // mêmes hypothèses que SolvSys_AFPMSM_OS : encoches ouvertes sur toute leur largeur
        s.tip_h = 0.0;
        s.slot_open = s.slot_w;
    }
    if std::env::var("PLEINE").is_ok() {
        // même hypothèse que SlotCurrDensity_AFPMSM : densité de courant nette
        // uniforme sur toute l'encoche (côtés de bobines non séparés)
        for c in s.coils.iter_mut() {
            c.go_side = 2;
            c.ret_side = 2;
        }
    }
    println!("topologie {:?}, {}N{}P, tranches {}, entrefers {}, ouverture {:.2} mm, corps {:.2} mm, becs {:.2} mm, bobines {}",
        s.topology, s.n_slots, s.n_poles, s.axial.slices, s.axial.gaps, s.slot_open, s.slot_w, s.tip_h, s.coils.len());
    let t0 = std::time::Instant::now();
    let m = Model::new(&s, Resolution { nh: 300, nk: 16 }).unwrap();
    println!("modèle construit en {:?}", t0.elapsed());
    // inductances (réaction d'induit, sans aimants)
    let la = m.phase_flux(&m.solve_armature(&[1.0, 0.0, 0.0]));
    let per = fractlab::geometry::gcd(s.n_slots, s.n_poles / 2) as f64;
    let g = if s.topology == fractlab::geometry::Topology::Axial { s.axial.gaps as f64 } else { 1.0 };
    println!("inductances FractLab (machine entière) : L = {:.4} mH, M_AB = {:.4} mH, M_AC = {:.4} mH (hors têtes de bobines)", la[0] * 1e3, la[1] * 1e3, la[2] * 1e3);
    println!("   ramenées à une période et un entrefer (convention SIMUMSAP) : L = {:.4} mH, M = {:.4} mH", la[0] * 1e3 / per / g, la[1] * 1e3 / per / g);
    let e0 = analysis::emf(&m, 360);
    println!("   FEM ramenée à une période et un entrefer : {:.2} V crête", e0.emf_peak_v / per / g);
    let fl = e0.flux[0].iter().fold(0.0f64, |a, v| a.max(v.abs()));
    println!("   flux de phase ramené : {:.4} Wb crête", fl / per / g);
    if let Some(l) = &imp.inductances {
        println!("inductances SIMUMSAP : L = {:.4} mH, M = {:.4} mH", l[0][0] * 1e3, l[0][1] * 1e3);
    }
    // cogging, FEM, attraction
    let p = 360.0 / fractlab::geometry::lcm(s.n_slots, s.n_poles) as f64;
    let cg = analysis::cogging(&m, p, 64);
    let cpk = cg.torque_nm.iter().fold(0.0f64, |x, v| x.max(v.abs()));
    println!("cogging machine entière : crête {:.3} N·m ; par entrefer {:.3} N·m", cpk, cpk / g);
    let e = analysis::emf(&m, 360);
    println!("FEM à {:.0} tr/min : crête phase {:.2} V", s.rpm, e.emf_peak_v);
    let fz = m.axial_attraction(&m.solve(0.0));
    println!("attraction axiale d'un disque : {:.0} N", fz.abs());
    if a.len() > 2 {
        for f in &a[2..] {
            if !std::path::Path::new(f).exists() { continue; }
            if let Ok(ax) = fractlab::matfile::figure_curves(&std::fs::read(f).unwrap()) {
                for x in ax {
                    for c in x.curves.iter().take(1) {
                        let mx = c.y.iter().fold(0.0f64, |a, v| a.max(v.abs()));
                        let mn = c.y.iter().cloned().fold(f64::MAX, f64::min);
                        println!("  figure {} : {} → crête {:.4}, min {:.4}", f.rsplit('/').next().unwrap(), x.ylabel, mx, mn);
                    }
                }
            }
        }
    }
}
