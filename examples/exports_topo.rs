//! Exports pour les trois topologies (contrôle par outils externes).
use fractlab::exchange as x;
use fractlab::geometry::{MotorSpec, Topology};
fn main() {
    let out = std::env::args().nth(1).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    let mut inner = MotorSpec::default();
    inner.topology = Topology::RadialInner;
    inner.stator_or = 9.8; inner.stator_ir = 16.0; inner.yoke_th = 1.5; inner.tip_h = 0.5; inner.slot_w = 3.5;
    let mut ax = MotorSpec::default();
    ax.topology = Topology::Axial; ax.slot_depth = 12.0; ax.slot_w = 8.0; ax.slot_open = 3.0; ax.airgap = 1.5; ax.magnet_th = 6.5;
    ax.axial.r_in = 40.0; ax.axial.r_out = 70.0;
    for (n, s) in [("interieur", inner), ("axial", ax)] {
        for part in ["stator", "aimants", "cloche"] {
            std::fs::write(format!("{out}/{n}_{part}.stl"), x::stl(&s, part)).unwrap();
        }
        std::fs::write(format!("{out}/{n}.dxf"), x::dxf(&s)).unwrap();
        std::fs::write(format!("{out}/{n}.geo"), x::gmsh_geo(&s)).unwrap();
    }
}
