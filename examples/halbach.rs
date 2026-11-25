//! Contrôle physique : le réseau de Halbach doit renforcer le fondamental de
//! l'induction d'entrefer par rapport à l'aimantation radiale (côté stator).
use fractlab::analysis;
use fractlab::geometry::{Magnetisation, MotorSpec};
use fractlab::subdomain::{Model, Resolution};
fn main() {
    for mag in [Magnetisation::Radiale, Magnetisation::Parallele, Magnetisation::Halbach { segments: 4 }] {
        let mut s = MotorSpec::default();
        s.magnetisation = mag;
        let m = Model::new(&s, Resolution { nh: 300, nk: 16 }).unwrap();
        let g = analysis::gap_field(&m, &m.solve(0.0), 1440);
        println!("{mag:?}: fondamental Br (rang 7) = {:.4} T", g.spectrum_br[7]);
    }
}
