//! Ordres de grandeur des pertes, de l'échauffement et du son selon la vitesse.
use fractlab::compute::{self, Msg, Settings};
use fractlab::dynamics as d;
use fractlab::geometry::MotorSpec;
fn main() {
    for v in ["regulier", "irregulier"] {
        let job = compute::run_main(MotorSpec::example(v), Settings::default());
        let Msg::Main(Ok(r)) = job.rx.recv().unwrap() else { panic!() };
        let m = d::masses(&r.spec);
        let p = d::DynParams::default();
        let ic = d::iron_coeffs(&r.iron, &m, &r.spec, &p);
        println!("{v}: rotor {:.1} g, stator {:.1} g, J = {:.2e} kg·m², B dent crête {:.2} T, B culasse crête {:.2} T",
            m.rotor * 1e3, m.stator * 1e3, m.inertia, ic.b_tooth_peak, ic.b_yoke_peak);
        for rpm in [1000.0, 10000.0, 20000.0, 30000.0] {
            let s = d::evaluate(&r, &ic, &m, &p, rpm);
            let t = &s.tones[0];
            let v = d::vibration(&r, &m, &p, rpm, s.br_ratio);
            println!("           vibration : {:.4} µm crête, {:.3} mm/s eff. (zone ISO {}), {:.2} g, critique {:.0} tr/min, ondulation vitesse {:.2} tr/min",
                v.disp_peak_um, v.vel_rms_mm_s, v.iso_zone, v.acc_peak_g, v.critical_rpm, v.speed_ripple_rpm);
            println!("  {rpm:>6} tr/min  f_e {:>6.0} Hz | fer {:>5.2} W (hyst {:.2}, Foucault {:.2}) roul. {:.3} W vent. {:.3} W | total {:>5.2} W | T {:>5.1} °C τ {:>4.0} s Br×{:.3} | raie dominante {:.0} Hz ({}) indice {:.1} dB",
                s.f_elec_hz, s.p_hyst + s.p_eddy, s.p_hyst, s.p_eddy, s.p_bearing, s.p_windage, s.p_total, s.t_steady, s.tau_s, s.br_ratio, t.freq_hz, t.source, s.la_index_db);
        }
    }
}
