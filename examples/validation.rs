//! Banc de validation : sous-domaines (analytique) vs volumes finis (numérique).
use fractlab::fdcheck::FdModel;
use fractlab::geometry::MotorSpec;
use fractlab::subdomain::{Model, Resolution};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let nt: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5760);
    let scale: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    use fractlab::geometry::Magnetisation;
    let cases = [
        ("regulier", Magnetisation::Radiale),
        ("irregulier", Magnetisation::Radiale),
        ("irregulier", Magnetisation::Parallele),
        ("irregulier", Magnetisation::Halbach { segments: 4 }),
    ];
    for (v, mag) in cases {
        let t = Instant::now();
        let mut s0 = MotorSpec::example(v);
        s0.magnetisation = mag;
        let fd = FdModel::new(&s0, nt, scale);
        let spec = fd.spec.clone(); // ouverture arrondie à la grille
        let an = Model::new(&spec, Resolution { nh: 600, nk: 36 }).unwrap();
        eprintln!("{v}: grille {}×{} prête en {:?} (ouverture {:.4} mm)", nt, fd.g.rc.len(), t.elapsed(), spec.slot_open);
        let per = spec.cogging_period_deg();
        let (mut amp, mut err) = (0.0f64, 0.0f64);
        println!("{v} {mag:?}  θ[°]    analytique[mN·m]   numérique[mN·m]");
        for k in 0..8 {
            let th = per * (k as f64 + 0.5) / 8.0;
            let ta = an.torque(&an.solve(th.to_radians())) * 1e3;
            let tf = fd.torque(&fd.solve(th.to_radians())) * 1e3;
            println!("   {th:7.3}  {ta:+12.4}   {tf:+12.4}");
            amp = amp.max(ta.abs());
            err = err.max((ta - tf).abs());
        }
        println!("   => écart max / amplitude = {:.2} %", 100.0 * err / amp);
    }
}
