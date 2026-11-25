//! Validation du modèle général (section.rs) contre les volumes finis :
//! encoches semi-fermées, rotor intérieur, courants d'encoche, et tranche
//! plane de machine à flux axial (circonférence développée).
use fractlab::fdcheck::FdModel;
use fractlab::geometry::MotorSpec;
use fractlab::magnets;
use fractlab::section::{Coord, Harmonics, Section, SectionGeom};
use std::f64::consts::PI;

fn run(nom: &str, geom: SectionGeom, j: Vec<f64>, br_on: bool) {
    let spec = MotorSpec::irregular_example();
    let br = if br_on { spec.br_magnet } else { 0.0 };
    let pieces = magnets::pieces(&spec);
    let (fd, ge) = FdModel::general(&geom, pieces.clone(), br, 5760, 1, &j);
    let sec = Section::new(ge, Harmonics { nh: 400, ks: 24, ko: 20 }).unwrap();
    let mf = magnets::fourier(&pieces, br.max(1e-30), 400);
    let (mut amp, mut err) = (0.0f64, 0.0f64);
    println!("{nom}");
    for k in 0..6 {
        let th = 5.0 * k as f64 + 1.3;
        let ta = sec.rotor_torque(&sec.solve(&mf.rotated(th.to_radians()), &j)) * 1e3;
        let tf = fd.torque(&fd.solve(th.to_radians())) * 1e3;
        println!("   θ={th:5.1}°  analytique {ta:+10.4} mN·m   numérique {tf:+10.4} mN·m");
        amp = amp.max(ta.abs());
        err = err.max((ta - tf).abs());
    }
    if amp < 1e-6 {
        println!("   => couple nul attendu (pas d'aimants, rotor lisse) : |écart| max = {err:.2e} mN·m");
    } else {
        println!("   => écart max / amplitude = {:.2} %", 100.0 * err / amp);
    }
    // champ : flux entre deux angles au milieu de l'entrefer (indépendant de la jauge)
    let a_fd = fd.solve(0.0);
    let f = sec.solve(&mf.rotated(0.0), &j);
    let nr = fd.g.rc.len();
    let i_mid = (0..nr).filter(|&i| fd.g.zone[i] == 1).nth(3).unwrap();
    let y = fd.g.rc[i_mid];
    let (mut amp2, mut err2) = (0.0f64, 0.0f64);
    for k in 0..24 {
        let (t1, t2) = (k as f64 * 15.0, k as f64 * 15.0 + 15.0);
        let cell = |t: f64| ((t.to_radians() / fd.g.dth).round() as usize % fd.g.nt) * nr + i_mid;
        let dfd = a_fd[cell(t2)] - a_fd[cell(t1)];
        let th = |t: f64| ((t.to_radians() / fd.g.dth).round()) * fd.g.dth + 0.5 * fd.g.dth;
        let dan = sec.gap_a(&f, y, th(t2)) - sec.gap_a(&f, y, th(t1));
        amp2 = amp2.max(dan.abs());
        err2 = err2.max((dan - dfd).abs());
    }
    println!("   champ (flux sur 15° dans l'entrefer) : écart max / amplitude = {:.2} %", 100.0 * err2 / amp2);
}

fn main() {
    let q = 12;
    let centers: Vec<f64> = (0..q).map(|i| (i as f64 + 0.5) * 2.0 * PI / q as f64).collect();
    let jpat: Vec<f64> = (0..q).map(|i| 8e6 * (2.0 * PI * i as f64 * 5.0 / q as f64).cos()).collect();
    let outer = SectionGeom { coord: Coord::Polar, y_sb: 5.8e-3, y_ib: 10.4e-3, y_s: 11e-3, y_m: 11.2e-3, y_r: 12.8e-3,
        slot_center: centers.clone(), beta_s: 0.36, beta_o: 0.16, mur: 1.05, depth: 7e-3 };
    let inner = SectionGeom { coord: Coord::Polar, y_sb: 15e-3, y_ib: 10.4e-3, y_s: 9.8e-3, y_m: 9.6e-3, y_r: 8.0e-3,
        slot_center: centers.clone(), beta_s: 0.30, beta_o: 0.14, mur: 1.05, depth: 7e-3 };
    run("1. rotor extérieur, encoches semi-fermées (à vide)", outer.clone(), vec![0.0; q], true);
    run("2. rotor intérieur, encoches semi-fermées (à vide)", inner.clone(), vec![0.0; q], true);
    run("3. rotor extérieur, aimants + courants (en charge)", outer.clone(), jpat.clone(), true);
    run("4. rotor intérieur, courants seuls (réaction d'induit)", inner, jpat, false);
    // tranche plane (flux axial) : rayon moyen 50 mm, épaisseur radiale 8 mm ; y = cote axiale
    let plane = SectionGeom { coord: Coord::Planar { r0: 0.05 }, y_sb: 0.0, y_ib: 6.0e-3, y_s: 6.8e-3, y_m: 7.8e-3, y_r: 10.8e-3,
        slot_center: centers.clone(), beta_s: 0.30, beta_o: 0.12, mur: 1.05, depth: 8e-3 };
    let jpat2: Vec<f64> = (0..q).map(|i| 4e6 * (2.0 * PI * i as f64 * 5.0 / q as f64).cos()).collect();
    run("5. tranche plane de machine axiale, encoches semi-fermées (à vide)", plane.clone(), vec![0.0; q], true);
    run("6. tranche plane de machine axiale, aimants + courants (en charge)", plane, jpat2, true);
}
