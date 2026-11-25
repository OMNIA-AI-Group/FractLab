//! Génère une FEM « mesurée » synthétique (CSV angle ; V) pour essayer le recalage :
//! Br = 1,30 T et entrefer = 0,25 mm au lieu de 1,45 T et 0,20 mm, origine
//! décalée de 11°, bruit de ±0,03 V.
fn main() {
    let out = std::env::args().nth(1).expect("fichier de sortie");
    let mut s = fractlab::geometry::MotorSpec::irregular_example();
    s.br_magnet = 1.30;
    s.airgap = 0.25;
    let m = fractlab::subdomain::Model::new(&s, Default::default()).unwrap();
    let e = fractlab::analysis::emf(&m, 720);
    let mut txt = String::from("angle_deg;emf_V\n");
    let mut seed: u64 = 12345;
    for k in 0..180 {
        let a = 2.0 * k as f64;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let bruit = ((seed >> 33) as f64 / (1u64 << 31) as f64 - 0.5) * 0.06;
        let v = fractlab::dynamics::interp_periodic(&e.angles_deg, &e.emf[0], 360.0, a + 11.0);
        txt += &format!("{a};{:.5}\n", v + bruit);
    }
    std::fs::write(out, txt).unwrap();
}
