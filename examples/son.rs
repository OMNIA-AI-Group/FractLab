//! Écrit le son synthétisé d'un rotor régulier et d'un rotor irrégulier (contrôle spectral).
use fractlab::compute::{self, Msg, Settings};
use fractlab::geometry::MotorSpec;
fn main() {
    let out = std::env::args().nth(1).unwrap();
    for v in ["regulier", "irregulier"] {
        let job = compute::run_main(MotorSpec::example(v), Settings::default());
        let Msg::Main(Ok(r)) = job.rx.recv().unwrap() else { panic!() };
        std::fs::write(format!("{out}/{v}.wav"), fractlab::dynamics::synth_wav(&r, 10000.0, 1.0)).unwrap();
    }
}
