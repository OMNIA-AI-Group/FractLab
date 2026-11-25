//! Bobinage concentré sur dents (double couche) déterminé par l'étoile des
//! encoches : chaque bobine j (autour de la dent j, centrée en θ = j·2π/Q)
//! a un angle électrique p·j·2π/Q ; secteurs de 60° → phase et sens.

use crate::geometry::MotorSpec;

#[derive(Clone, Debug)]
/// Bobine concentrée autour d'une dent.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Coil {
    /// Numéro de la dent (centrée en θ = dent · 2π/Q).
    pub tooth: usize,
    /// Phase : 0 = A, 1 = B, 2 = C.
    pub phase: usize, // 0 = A, 1 = B, 2 = C
    /// Sens de bobinage (+1 ou −1).
    pub sign: f64,
}

/// Noms des phases.
pub const PHASES: [&str; 3] = ["A", "B", "C"];

/// Bobinage concentré double couche déduit de l'étoile des encoches.
pub fn layout(spec: &MotorSpec) -> Vec<Coil> {
    let q = spec.n_slots;
    let p = spec.n_poles / 2;
    (0..q)
        .map(|j| {
            let e = ((p * j) as f64 * 360.0 / q as f64).rem_euclid(360.0);
            // secteurs [−30,30) A+, [30,90) C−, [90,150) B+, [150,210) A−, [210,270) C+, [270,330) B−
            let s = (((e + 30.0) / 60.0).floor() as usize) % 6;
            let (phase, sign) = [(0, 1.0), (2, -1.0), (1, 1.0), (0, -1.0), (2, 1.0), (1, -1.0)][s];
            Coil { tooth: j, phase, sign }
        })
        .collect()
}

/// Notation compacte du bobinage, ex. « A c' B a' … » (apostrophe = sens inverse).
pub fn notation(coils: &[Coil]) -> String {
    coils
        .iter()
        .map(|c| {
            let n = PHASES[c.phase];
            if c.sign > 0.0 { n.to_string() } else { format!("{}'", n.to_lowercase()) }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Le bobinage est équilibré si chaque phase a le même nombre de bobines.
pub fn balanced(coils: &[Coil]) -> bool {
    let mut n = [0; 3];
    coils.iter().for_each(|c| n[c.phase] += 1);
    n[0] == n[1] && n[1] == n[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bobinage_12n14p() {
        let mut s = MotorSpec::default();
        s.n_slots = 12;
        s.n_poles = 14;
        let c = layout(&s);
        assert!(balanced(&c));
        // paires adjacentes de même phase en sens opposés (AA')
        assert_eq!(c[5].phase, 0);
        assert_eq!(c[6].phase, 0);
        assert!(c[5].sign * c[6].sign < 0.0);
    }
}
