//! Unités d'affichage et de saisie choisies dans les préférences.
//! Les calculs restent en SI ; seules la saisie et l'affichage convertissent.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Grandeur physique affichable.
pub enum Quantity {
    /// Longueur.
    Length,
    /// Couple.
    Torque,
    /// Force.
    Force,
    /// Induction magnétique.
    Induction,
    /// Angle.
    Angle,
    /// Vitesse de rotation.
    Speed,
    /// Flux magnétique.
    Flux,
    /// Tension.
    Voltage,
    /// Balourd.
    Unbalance,
}

/// Liste des grandeurs, dans l'ordre des préférences.
pub const QUANTITIES: [Quantity; 9] = [
    Quantity::Length,
    Quantity::Torque,
    Quantity::Force,
    Quantity::Induction,
    Quantity::Angle,
    Quantity::Speed,
    Quantity::Flux,
    Quantity::Voltage,
    Quantity::Unbalance,
];

/// (symbole, facteur : valeur affichée = valeur de référence × facteur)
/// Référence : mm, N·m, N, T, °, tr/min, Wb, V, mg·cm.
pub fn options(q: Quantity) -> &'static [(&'static str, f64)] {
    match q {
        Quantity::Length => &[("mm", 1.0), ("µm", 1000.0), ("cm", 0.1), ("in", 1.0 / 25.4), ("mil", 1000.0 / 25.4)],
        Quantity::Torque => &[("mN·m", 1000.0), ("N·m", 1.0), ("N·cm", 100.0), ("oz·in", 141.611_932_4), ("lbf·in", 8.850_745_8)],
        Quantity::Force => &[("N", 1.0), ("mN", 1000.0), ("kgf", 1.0 / 9.806_65), ("lbf", 0.224_808_9)],
        Quantity::Induction => &[("T", 1.0), ("mT", 1000.0), ("G", 10_000.0)],
        Quantity::Angle => &[("°", 1.0), ("rad", std::f64::consts::PI / 180.0), ("arcmin", 60.0)],
        Quantity::Speed => &[("tr/min", 1.0), ("rad/s", std::f64::consts::PI / 30.0), ("Hz", 1.0 / 60.0)],
        Quantity::Flux => &[("mWb", 1000.0), ("Wb", 1.0), ("µWb", 1e6)],
        Quantity::Voltage => &[("V", 1.0), ("mV", 1000.0)],
        Quantity::Unbalance => &[("mg·cm", 1.0), ("g·mm", 0.01), ("g·cm", 0.001)],
    }
}

/// Nom français de la grandeur (clé de traduction).
pub fn name_fr(q: Quantity) -> &'static str {
    match q {
        Quantity::Length => "Longueurs",
        Quantity::Torque => "Couple",
        Quantity::Force => "Force",
        Quantity::Induction => "Induction",
        Quantity::Angle => "Angles",
        Quantity::Speed => "Vitesse",
        Quantity::Flux => "Flux",
        Quantity::Voltage => "Tension",
        Quantity::Unbalance => "Balourd",
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Choix d'unité pour chaque grandeur.
#[derive(Default)]
pub struct Units {
    /// indice choisi dans `options(q)` pour chaque grandeur (ordre de QUANTITIES)
    pub pick: [usize; 9],
}


impl Units {
    fn idx(q: Quantity) -> usize {
        QUANTITIES.iter().position(|x| *x == q).unwrap()
    }
    /// Symbole de l'unité choisie.
    pub fn sym(&self, q: Quantity) -> &'static str {
        let o = options(q);
        o[self.pick[Self::idx(q)].min(o.len() - 1)].0
    }
    /// Facteur de conversion depuis l'unité de référence.
    pub fn factor(&self, q: Quantity) -> f64 {
        let o = options(q);
        o[self.pick[Self::idx(q)].min(o.len() - 1)].1
    }
    /// valeur de référence → affichage
    pub fn show(&self, q: Quantity, v: f64) -> f64 {
        v * self.factor(q)
    }
    /// Accès modifiable au choix d'unité.
    pub fn pick_mut(&mut self, q: Quantity) -> &mut usize {
        &mut self.pick[Self::idx(q)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversions() {
        let mut u = Units::default();
        *u.pick_mut(Quantity::Length) = 3; // pouce
        assert!((u.show(Quantity::Length, 25.4) - 1.0).abs() < 1e-12);
        *u.pick_mut(Quantity::Torque) = 3; // oz·in
        assert!((u.show(Quantity::Torque, 1.0) - 141.6119).abs() < 1e-3);
    }
}
