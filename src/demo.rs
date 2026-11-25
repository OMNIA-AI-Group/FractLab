//! Moteurs de démonstration : machines génériques aux cotes du commerce,
//! utilisées par la démo et comme projet de départ. Aucune n'est issue d'une
//! étude particulière.

use crate::geometry::{AxialSpec, LoadSpec, Magnetisation, MotorSpec, Topology};

/// Un moteur de démonstration.
pub struct DemoMotor {
    /// Identifiant court.
    pub key: &'static str,
    /// Nom (clé de traduction).
    pub name: &'static str,
    /// Description (clé de traduction).
    pub desc: &'static str,
    /// Icône.
    pub icon: &'static str,
}

/// Les moteurs proposés, dans l'ordre de la démo.
pub const MOTORS: [DemoMotor; 4] = [
    DemoMotor { key: "drone", name: "Moteur de drone 9N12P", desc: "Rotor extérieur (cloche), encoches ouvertes, petit et rapide.", icon: "🚁" },
    DemoMotor { key: "servo", name: "Servomoteur 12N8P", desc: "Rotor intérieur, encoches semi-fermées (becs de dents), couple précis.", icon: "⚙" },
    DemoMotor { key: "roue", name: "Moteur-roue de vélo 36N42P", desc: "Grand rotor extérieur, beaucoup de pôles, couple à basse vitesse.", icon: "🚲" },
    DemoMotor { key: "axial", name: "Génératrice à flux axial 18N16P", desc: "Disques : un stator entre deux rotors (deux entrefers).", icon: "💿" },
];

/// Cotes d'un moteur de démonstration (longueurs en mm).
pub fn spec(key: &str) -> MotorSpec {
    let mut s = MotorSpec::bare();
    match key {
        "servo" => {
            s.name = "Servomoteur 12N8P".into();
            s.topology = Topology::RadialInner;
            s.n_slots = 12;
            s.n_poles = 8;
            // rotor intérieur : `stator_or` = alésage, `stator_ir` = rayon extérieur du stator
            s.stator_or = 22.0;
            s.stator_ir = 40.0;
            s.stack_len = 40.0;
            s.airgap = 0.6;
            s.magnet_th = 3.0;
            s.rotor_yoke_th = 8.0;
            s.tooth_w = 6.0;
            s.slot_open = 2.5;
            s.tip_h = 1.2;
            s.slot_w = 5.0;
            s.yoke_th = 6.0;
            s.magnet_arc_ratio = 0.85;
            s.br_magnet = 1.25;
            s.turns_per_coil = 30.0;
            s.rpm = 3000.0;
            s.load = LoadSpec { i_peak: 8.0, ..Default::default() };
        }
        "roue" => {
            s.name = "Moteur-roue 36N42P".into();
            s.n_slots = 36;
            s.n_poles = 42;
            s.stator_or = 95.0;
            s.stator_ir = 65.0;
            s.stack_len = 25.0;
            s.airgap = 0.8;
            s.magnet_th = 3.0;
            s.rotor_yoke_th = 4.0;
            s.tooth_w = 8.0;
            s.slot_open = 3.5;
            s.yoke_th = 8.0;
            s.magnet_arc_ratio = 0.8;
            s.br_magnet = 1.25;
            s.turns_per_coil = 12.0;
            s.rpm = 300.0;
            s.load = LoadSpec { i_peak: 30.0, ..Default::default() };
        }
        "axial" => {
            s.name = "Génératrice axiale 18N16P".into();
            s.topology = Topology::Axial;
            s.n_slots = 18;
            s.n_poles = 16;
            s.axial = AxialSpec { r_in: 40.0, r_out: 75.0, slices: 4, gaps: 2, arc_ratio_out: 0.8 };
            s.stator_or = 75.0;
            s.stator_ir = 40.0;
            s.airgap = 1.0;
            s.magnet_th = 5.0;
            s.rotor_yoke_th = 6.0;
            s.tooth_w = 14.0;
            s.slot_open = 6.0;
            s.slot_depth = 14.0;
            s.yoke_th = 10.0;
            s.magnet_arc_ratio = 0.8;
            s.br_magnet = 1.25;
            s.turns_per_coil = 40.0;
            s.rpm = 400.0;
            s.load = LoadSpec { i_peak: 10.0, ..Default::default() };
        }
        _ => {
            s.name = "Drone 9N12P".into();
            s.n_slots = 9;
            s.n_poles = 12;
            s.stator_or = 14.0;
            s.stator_ir = 4.5;
            s.stack_len = 6.0;
            s.airgap = 0.25;
            s.magnet_th = 1.5;
            s.rotor_yoke_th = 2.4;
            s.tooth_w = 4.2;
            s.slot_open = 2.2;
            s.yoke_th = 2.6;
            s.magnet_arc_ratio = 0.8;
            s.br_magnet = 1.3;
            s.turns_per_coil = 10.0;
            s.rpm = 9000.0;
            s.load = LoadSpec { i_peak: 20.0, ..Default::default() };
        }
    }
    s.magnetisation = Magnetisation::Radiale;
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque moteur de démonstration se calcule (géométrie cohérente, FEM et couple positifs).
    #[test]
    fn moteurs_de_demo_calculables() {
        let st = crate::compute::Settings { n_cog: 60, n_emf: 120, ..Default::default() };
        for m in MOTORS {
            let s = spec(m.key);
            assert!(s.geometry_problem().is_none(), "{}", m.key);
            let r = crate::compute::compute_results(s, &st).unwrap_or_else(|e| panic!("{} : {e}", m.key));
            assert!(r.emf.emf_peak_v > 0.0, "{}", m.key);
            assert!(r.load.avg_torque > 0.0, "{} couple {}", m.key, r.load.avg_torque);
        }
    }
}
