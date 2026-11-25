//! Paramètres du moteur (le « projet ») et répartition des aimants.
//!
//! Les longueurs sont en millimètres dans la spécification et converties en
//! mètres par le modèle. Les répartitions autres que régulière ou
//! personnalisée sont fournies par `crate::repartition`.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Mode d'aimantation des aimants du rotor.
pub enum Magnetisation {
    /// Aimantation radiale (polarité alternée N/S).
    Radiale,
    /// Aimantation parallèle (direction fixe par aimant, selon son axe).
    Parallele,
    /// Réseau de Halbach : chaque pas polaire découpé en segments dont la
    /// direction tourne de (1 + 1/p)·… ; ici `segments` par pôle.
    /// Réseau de Halbach : chaque pôle est découpé en `segments` morceaux dont
    /// la direction d'aimantation tourne pour concentrer le flux vers l'entrefer.
    Halbach {
        /// Nombre de segments par pôle (≥ 2).
        segments: u32,
    },
}

/// Topologie de la machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Topology {
    /// Flux radial, rotor extérieur (cloche), stator encoché à l'intérieur.
    #[default]
    RadialOuter,
    /// Flux radial, rotor intérieur, stator encoché à l'extérieur.
    RadialInner,
    /// Flux axial (disques), modèle quasi-3D multi-tranches.
    Axial,
}

/// Paramètres propres aux machines à flux axial.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AxialSpec {
    /// Rayon intérieur de la partie active [mm].
    pub r_in: f64,
    /// Rayon extérieur de la partie active [mm].
    pub r_out: f64,
    /// Nombre de tranches annulaires (multi-tranches).
    pub slices: usize,
    /// Nombre d'entrefers (1 ou 2 disques rotoriques).
    pub gaps: usize,
    /// Arc d'aimant / pas polaire au rayon extérieur (au rayon intérieur : `magnet_arc_ratio`).
    pub arc_ratio_out: f64,
}

impl Default for AxialSpec {
    fn default() -> Self {
        AxialSpec { r_in: 40.0, r_out: 75.0, slices: 4, gaps: 2, arc_ratio_out: 0.8 }
    }
}

/// Bobine d'un bobinage général (encoche « aller » → encoche « retour »).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CoilDef {
    /// Phase (0 = A, 1 = B, 2 = C).
    pub phase: usize,
    /// Encoche aller (0 … Q−1).
    pub go: usize,
    /// Encoche retour.
    pub ret: usize,
    /// Nombre de spires.
    pub turns: f64,
    /// Position du côté aller dans l'encoche : 0 moitié gauche, 1 droite, 2 entière.
    pub go_side: u8,
    /// Position du côté retour.
    pub ret_side: u8,
}

/// Forme des courants de phase imposée par la commande.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Commutation {
    /// Courants sinusoïdaux (commande vectorielle, FOC).
    #[default]
    Sinus,
    /// Créneaux à 120° (commutation trapézoïdale à six pas, ESC de drone).
    Trapeze,
}

/// Conditions de charge (courants équilibrés, sinusoïdaux ou en créneaux).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LoadSpec {
    /// Courant de phase crête [A].
    pub i_peak: f64,
    /// Angle du courant par rapport à la FEM [° électriques] (0 = couple max).
    pub angle_deg: f64,
    /// Nombre de voies en parallèle par phase.
    pub parallel_paths: f64,
    /// Coefficient de remplissage des encoches.
    pub fill_factor: f64,
    /// Coefficient de perméance des têtes de bobines λ (0 = têtes ignorées,
    /// comme SIMUMSAP ; 0,3 à 0,5 usuel pour des bobines sur dent).
    pub end_lambda: f64,
    /// Forme des courants (sinusoïdale ou trapézoïdale 120°).
    pub commutation: Commutation,
}

impl Default for LoadSpec {
    fn default() -> Self {
        LoadSpec { i_peak: 10.0, angle_deg: 0.0, parallel_paths: 1.0, fill_factor: 0.45, end_lambda: 0.3, commutation: Commutation::Sinus }
    }
}

/// Description complète d'un moteur : dimensions en millimètres, aimants,
/// répartition, bobinage. C'est le « projet » enregistré en JSON.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotorSpec {
    /// Nom du projet (affiché et utilisé pour les fichiers exportés).
    pub name: String,
    /// Nombre d'encoches du stator Q.
    pub n_slots: usize,
    /// Nombre de pôles du rotor 2p (= nombre d'aimants, pair).
    pub n_poles: usize,
    /// rayon externe du stator = rayon d'alésage côté entrefer [mm]
    pub stator_or: f64,
    /// Rayon intérieur du stator (alésage de l'axe) [mm].
    pub stator_ir: f64,
    /// Longueur active du paquet de tôles (profondeur du modèle 2D) [mm].
    pub stack_len: f64,
    /// Épaisseur radiale de l'entrefer mécanique [mm].
    pub airgap: f64,
    /// Épaisseur radiale des aimants [mm].
    pub magnet_th: f64,
    /// Épaisseur de la culasse du rotor (cloche) [mm].
    pub rotor_yoke_th: f64,
    /// Largeur de dent (sert aux ratios de la variante V3) [mm].
    pub tooth_w: f64,
    /// Ouverture d'encoche côté entrefer [mm] ; c'est la largeur des encoches du modèle.
    pub slot_open: f64,
    /// Épaisseur de la culasse du stator [mm].
    pub yoke_th: f64,
    /// Arc d'un aimant rapporté au pas polaire (0 à 1).
    pub magnet_arc_ratio: f64,
    /// induction rémanente [T]
    pub br_magnet: f64,
    /// perméabilité relative de recul de l'aimant
    pub mur_magnet: f64,
    /// Mode d'aimantation (radiale, parallèle, Halbach).
    pub magnetisation: Magnetisation,
    /// Répartition des aimants (voir `crate::repartition`).
    pub variant: String,
    /// Amplitude maximale des décalages angulaires des aimants [°].
    pub skew_max_deg: f64,
    /// angles imposés (import) : remplacent la variante s'ils sont fournis
    #[serde(default)]
    pub custom_angles_deg: Option<Vec<f64>>,
    /// spires par bobine (bobinage concentré sur dent, double couche)
    #[serde(default = "def_turns")]
    pub turns_per_coil: f64,
    /// vitesse pour la FEM [tr/min]
    #[serde(default = "def_rpm")]
    pub rpm: f64,
    /// Paramètres de frottement, de pertes fer et de thermique.
    #[serde(default)]
    pub dynamics: crate::dynamics::DynParams,
    /// Topologie de la machine.
    #[serde(default)]
    pub topology: Topology,
    /// Épaisseur des becs de dents (isthmes) [mm] ; 0 = encoches ouvertes.
    #[serde(default)]
    pub tip_h: f64,
    /// Largeur du corps d'encoche [mm] (≥ ouverture ; 0 = égale à l'ouverture).
    #[serde(default)]
    pub slot_w: f64,
    /// Profondeur d'encoche imposée [mm] (0 = déduite des rayons ; requise en axial).
    #[serde(default)]
    pub slot_depth: f64,
    /// Paramètres de la machine axiale.
    #[serde(default)]
    pub axial: AxialSpec,
    /// Bobinage général (vide = bobinage concentré automatique).
    #[serde(default)]
    pub coils: Vec<CoilDef>,
    /// Conditions de charge.
    #[serde(default)]
    pub load: LoadSpec,
    /// Matériaux du fer et saturation.
    #[serde(default)]
    pub saturation: crate::saturation::SatSpec,
    /// Fiche du projet (rédaction).
    #[serde(default)]
    pub sheet: crate::doc::ProjectSheet,
}

fn def_turns() -> f64 {
    7.0
}
fn def_rpm() -> f64 {
    10000.0
}

impl Default for MotorSpec {
    /// Projet par défaut : le moteur de démonstration « drone ».
    fn default() -> Self {
        crate::demo::spec("drone")
    }
}

impl MotorSpec {
    /// Spécification de base (cotes neutres), complétée par les moteurs de démonstration.
    pub fn bare() -> Self {
        MotorSpec {
            name: "Projet".into(),
            n_slots: 9,
            n_poles: 12,
            stator_or: 14.0,
            stator_ir: 4.5,
            stack_len: 6.0,
            airgap: 0.25,
            magnet_th: 1.5,
            rotor_yoke_th: 1.5,
            tooth_w: 3.6,
            slot_open: 2.2,
            yoke_th: 2.2,
            magnet_arc_ratio: 0.8,
            br_magnet: 1.3,
            mur_magnet: 1.05,
            magnetisation: Magnetisation::Radiale,
            variant: crate::repartition::REGULIERE.into(),
            skew_max_deg: 0.0,
            custom_angles_deg: None,
            turns_per_coil: def_turns(),
            rpm: def_rpm(),
            dynamics: Default::default(),
            topology: Topology::RadialOuter,
            tip_h: 0.0,
            slot_w: 0.0,
            slot_depth: 0.0,
            axial: AxialSpec::default(),
            coils: Vec::new(),
            saturation: Default::default(),
            load: LoadSpec::default(),
            sheet: Default::default(),
        }
    }
}

impl MotorSpec {
    /// Projet de référence pour la répartition `v` (voir `crate::repartition::reference`).
    pub fn with_variant(v: &str) -> MotorSpec {
        crate::repartition::reference(v)
    }

    /// Exemple neutre : rotor régulier (`"regulier"`) ou irrégulier (tout autre nom).
    pub fn example(kind: &str) -> MotorSpec {
        if kind == "regulier" { MotorSpec::default() } else { MotorSpec::irregular_example() }
    }

    /// Projet de test neutre : le moteur de démonstration « drone », aimants
    /// déplacés de quelques degrés (répartition irrégulière équilibrée,
    /// sans rapport avec une étude particulière).
    pub fn irregular_example() -> MotorSpec {
        let mut s = crate::demo::spec("drone");
        let n = s.n_poles;
        let p = s.pole_pitch_deg();
        // décalages harmoniques d'ordres 2 et 3 : somme nulle et rotor
        // équilibré au premier ordre (aucune composante d'ordre 1)
        let tau = 2.0 * std::f64::consts::PI;
        let a: Vec<f64> = (0..n).map(|k| {
            let x = tau * k as f64 / n as f64;
            k as f64 * p + 2.0 * (2.0 * x).sin() + 1.2 * (3.0 * x + 0.7).sin()
        }).collect();
        s.custom_angles_deg = Some(a);
        s.name = "Irrégulier".into();
        s
    }

    /// Pas polaire mécanique [°] = 360 / 2p.
    pub fn pole_pitch_deg(&self) -> f64 {
        360.0 / self.n_poles as f64
    }

    /// Période du cogging pour un rotor RÉGULIER : 360°/ppcm(Q, 2p).
    pub fn cogging_period_regular_deg(&self) -> f64 {
        360.0 / lcm(self.n_slots, self.n_poles) as f64
    }

    /// Période réelle du couple de détente : pour un rotor apériodique, seule
    /// la symétrie du stator subsiste → 360°/Q.
    pub fn cogging_period_deg(&self) -> f64 {
        if self.is_regular() {
            self.cogging_period_regular_deg()
        } else {
            360.0 / self.n_slots as f64
        }
    }

    /// Vrai si les aimants sont exactement au pas régulier.
    pub fn is_regular(&self) -> bool {
        let a = self.magnet_angles_deg();
        let p = self.pole_pitch_deg();
        a.iter().enumerate().all(|(k, x)| (x - k as f64 * p).abs() < 1e-9)
    }

    /// Centres angulaires des aimants [deg].
    pub fn magnet_angles_deg(&self) -> Vec<f64> {
        if let Some(c) = &self.custom_angles_deg {
            return c.clone();
        }
        let pitch = self.pole_pitch_deg();
        let base = || (0..self.n_poles).map(|k| k as f64 * pitch).collect::<Vec<f64>>();
        if crate::repartition::is_regular_name(&self.variant) {
            return base();
        }
        crate::repartition::angles(self).unwrap_or_else(base)
    }

    // --- cotes [m] du tronçon de référence (axial : tranche au rayon moyen) ---
    /// Rayon moyen de la partie active d'une machine axiale [m].
    pub fn axial_r_mean(&self) -> f64 {
        0.5 * (self.axial.r_in + self.axial.r_out) * 1e-3
    }
    /// Profondeur d'encoche [m].
    pub fn slot_depth_m(&self) -> f64 {
        if self.slot_depth > 0.0 {
            return self.slot_depth * 1e-3;
        }
        match self.topology {
            Topology::RadialOuter => (self.stator_or - self.stator_ir - self.yoke_th) * 1e-3,
            Topology::RadialInner => (self.stator_ir - self.yoke_th - self.stator_or) * 1e-3,
            Topology::Axial => 0.012,
        }
    }
    /// Profondeur d'encoche disponible dans un stator radial [mm] (épaisseur
    /// du stator moins la culasse) ; `None` en flux axial.
    pub fn radial_slot_room(&self) -> Option<f64> {
        match self.topology {
            Topology::RadialOuter => Some(self.stator_or - self.stator_ir - self.yoke_th),
            Topology::RadialInner => Some(self.stator_ir - self.yoke_th - self.stator_or),
            Topology::Axial => None,
        }
    }
    /// Contrôle de la géométrie avant calcul : message clair au lieu d'une
    /// erreur numérique (matrice singulière).
    pub fn geometry_problem(&self) -> Option<String> {
        if let Some(room) = self.radial_slot_room() {
            if room <= 0.0 {
                return Some(format!("géométrie impossible : le stator est trop mince pour sa culasse ({room:.2} mm)"));
            }
            if self.slot_depth > room + 1e-9 {
                return Some(format!("géométrie impossible : profondeur d'encoche {:.2} mm, place disponible {room:.2} mm (mettre 0 = automatique)", self.slot_depth));
            }
        }
        None
    }
    /// Surface du stator côté entrefer (alésage) [m].
    pub fn r_bore(&self) -> f64 {
        match self.topology {
            Topology::Axial => self.axial_r_mean(),
            _ => self.stator_or * 1e-3,
        }
    }
    /// Sens « stator → rotor » : +1 si le rotor est du côté des y croissants.
    pub fn rotor_dir(&self) -> f64 {
        if self.topology == Topology::RadialInner { -1.0 } else { 1.0 }
    }
    /// Face des aimants côté entrefer [m].
    pub fn r_magnet_in(&self) -> f64 {
        self.r_bore() + self.rotor_dir() * self.airgap * 1e-3
    }
    /// Face des aimants côté fer rotorique [m].
    pub fn r_magnet_out(&self) -> f64 {
        self.r_magnet_in() + self.rotor_dir() * self.magnet_th * 1e-3
    }
    /// Fond d'encoche [m].
    pub fn r_slot_bottom(&self) -> f64 {
        self.r_bore() - self.rotor_dir() * self.slot_depth_m()
    }
    /// Longueur active [m].
    pub fn length(&self) -> f64 {
        match self.topology {
            Topology::Axial => (self.axial.r_out - self.axial.r_in) * 1e-3,
            _ => self.stack_len * 1e-3,
        }
    }
}

/// Anneaux de la coupe, pour le dessin et les masses (cotes en m, angles en rad).
#[derive(Clone, Copy, Debug)]
pub struct Rings {
    /// Culasse statorique : (fond d'encoche, bord extérieur de la culasse).
    pub yoke: (f64, f64),
    /// Fond d'encoche.
    pub sb: f64,
    /// Base des becs de dents (= alésage si encoches ouvertes).
    pub tip: f64,
    /// Alésage (surface du stator côté entrefer).
    pub bore: f64,
    /// Aimants : (côté entrefer, côté fer).
    pub mag: (f64, f64),
    /// Culasse rotorique.
    pub rotor_yoke: (f64, f64),
    /// Plus grande cote (cadrage).
    pub r_max: f64,
    /// Largeur angulaire du corps d'encoche.
    pub beta_body: f64,
    /// Largeur angulaire de l'ouverture.
    pub beta_open: f64,
}

impl MotorSpec {
    /// Anneaux de la coupe (tranche moyenne déroulée pour une machine axiale).
    pub fn rings(&self) -> Rings {
        let d = self.rotor_dir();
        let bore = self.r_bore();
        let sb = self.r_slot_bottom();
        let tip = bore - d * self.tip_h * 1e-3;
        let mag = (self.r_magnet_in(), self.r_magnet_out());
        let ry = (mag.1, mag.1 + d * self.rotor_yoke_th * 1e-3);
        let yoke = (sb, sb - d * self.yoke_th * 1e-3);
        let beta_open = self.slot_open * 1e-3 / bore;
        let beta_body = if self.tip_h > 0.0 { (self.slot_w.max(self.slot_open) * 1e-3 / tip).max(beta_open) } else { beta_open };
        let r_max = [yoke.1, sb, bore, mag.1, ry.1].iter().fold(0.0f64, |a, v| a.max(v.abs()));
        Rings { yoke, sb, tip, bore, mag, rotor_yoke: ry, r_max, beta_body, beta_open }
    }
}

/// Couche d'une machine à flux axial, le long de l'axe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxialLayer {
    /// Disque de fer du rotor (culasse).
    RotorYoke,
    /// Aimants.
    Magnets,
    /// Entrefer.
    Airgap,
    /// Dents et encoches du stator.
    Teeth,
    /// Culasse du stator.
    StatorYoke,
}

impl MotorSpec {
    /// Empilement axial d'une machine à flux axial : (couche, z début, z fin)
    /// en mm, z = 0 au milieu de la culasse du stator. Avec deux entrefers, le
    /// stator est au centre (dents des deux côtés) entre deux disques rotors ;
    /// avec un seul, l'empilement ne va que vers +z. Même convention que
    /// l'export STL.
    pub fn axial_stack(&self) -> Vec<(AxialLayer, f64, f64)> {
        let (yk, dep, gp, hm, ry) = (self.yoke_th, self.slot_depth_m() * 1e3, self.airgap, self.magnet_th, self.rotor_yoke_th);
        let mut v = vec![(AxialLayer::StatorYoke, -yk / 2.0, yk / 2.0)];
        let sides: &[f64] = if self.axial.gaps >= 2 { &[1.0, -1.0] } else { &[1.0] };
        for &sg in sides {
            let mut z = yk / 2.0;
            for (layer, th) in [(AxialLayer::Teeth, dep), (AxialLayer::Airgap, gp), (AxialLayer::Magnets, hm), (AxialLayer::RotorYoke, ry)] {
                let (a, b) = (z, z + th);
                v.push(if sg > 0.0 { (layer, a, b) } else { (layer, -b, -a) });
                z = b;
            }
        }
        v.sort_by(|a, b| a.1.total_cmp(&b.1));
        v
    }
}

/// Plus grand commun diviseur.
pub fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}
/// Plus petit commun multiple.
pub fn lcm(a: usize, b: usize) -> usize {
    a / gcd(a, b) * b
}

/// Faisabilité : écart angulaire libre [°] entre chaque aimant et le suivant
/// (négatif = les aimants se CHEVAUCHENT, géométrie irréalisable).
pub fn magnet_gaps_deg(spec: &MotorSpec) -> Vec<f64> {
    let a = spec.magnet_angles_deg();
    let n = a.len();
    let arc = spec.magnet_arc_ratio * spec.pole_pitch_deg();
    (0..n)
        .map(|k| {
            let next = a[(k + 1) % n] + if k + 1 == n { 360.0 } else { 0.0 };
            next - a[k] - arc
        })
        .collect()
}

/// Arc maximal (en fraction du pas polaire) sans chevauchement pour la
/// répartition courante, avec une marge de `margin_deg` entre aimants.
pub fn max_arc_ratio(spec: &MotorSpec, margin_deg: f64) -> f64 {
    let a = spec.magnet_angles_deg();
    let n = a.len();
    let min_dist = (0..n)
        .map(|k| a[(k + 1) % n] + if k + 1 == n { 360.0 } else { 0.0 } - a[k])
        .fold(f64::MAX, f64::min);
    ((min_dist - margin_deg) / spec.pole_pitch_deg()).clamp(0.0, 1.0)
}

/// Balourd résiduel dû à la répartition des aimants [mg·cm] : masse d'un
/// aimant (volume × 7,5 g/cm³) placée au rayon moyen des aimants.
pub fn balourd_mg_cm(spec: &MotorSpec) -> f64 {
    let r_cm = 0.5 * (spec.r_magnet_in() + spec.r_magnet_out()).abs() * 100.0;
    let arc = spec.magnet_arc_ratio * 2.0 * std::f64::consts::PI / spec.n_poles.max(1) as f64;
    let vol_cm3 = arc * r_cm * spec.magnet_th * 0.1 * spec.length() * 100.0;
    let m_mg = vol_cm3 * 7.5 * 1000.0;
    let (mut x, mut y) = (0.0, 0.0);
    for a in spec.magnet_angles_deg() {
        x += m_mg * r_cm * a.to_radians().cos();
        y += m_mg * r_cm * a.to_radians().sin();
    }
    x.hypot(y)
}

/// Autocorrélation circulaire maximale (hors décalage nul) des écarts angulaires.
pub fn autocorr_max(spec: &MotorSpec) -> Option<f64> {
    let a = spec.magnet_angles_deg();
    let p = spec.pole_pitch_deg();
    let mut dev: Vec<f64> = a.iter().enumerate().map(|(k, x)| x - k as f64 * p).collect();
    if dev.iter().all(|x| x.abs() < 1e-9) {
        return None;
    }
    let n = dev.len();
    let m = dev.iter().sum::<f64>() / n as f64;
    dev.iter_mut().for_each(|x| *x -= m);
    let ac = |l: usize| (0..n).map(|i| dev[i] * dev[(i + n - l) % n]).sum::<f64>();
    let a0 = ac(0);
    Some((1..n).map(|l| (ac(l) / a0).abs()).fold(0.0, f64::max))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exemple_irregulier_equilibre_et_realisable() {
        let s = MotorSpec::irregular_example();
        assert!(!s.is_regular());
        assert!(magnet_gaps_deg(&s).into_iter().fold(f64::MAX, f64::min) > 0.5);
        let mut r = s.clone();
        r.custom_angles_deg = None;
        assert!(balourd_mg_cm(&r) < 1e-6);
        assert!(autocorr_max(&s).is_some());
    }
    #[test]
    fn arc_maximal_sans_chevauchement() {
        let mut s = MotorSpec::irregular_example();
        s.magnet_arc_ratio = max_arc_ratio(&s, 0.0);
        assert!(magnet_gaps_deg(&s).into_iter().fold(f64::MAX, f64::min).abs() < 1e-9);
    }
}

#[cfg(test)]
mod tests_axial {
    use super::*;

    /// Empilement axial : couches jointives, symétrique avec deux entrefers.
    #[test]
    fn empilement_axial_coherent() {
        let mut s = MotorSpec::default();
        s.topology = Topology::Axial;
        for gaps in [1, 2] {
            s.axial.gaps = gaps;
            let st = s.axial_stack();
            assert_eq!(st.len(), 1 + 4 * gaps);
            assert!(st.windows(2).all(|w| (w[0].2 - w[1].1).abs() < 1e-9), "couches jointives");
            assert_eq!(st.iter().filter(|l| l.0 == AxialLayer::Airgap).count(), gaps);
            if gaps == 2 {
                assert!((st[0].1 + st.last().unwrap().2).abs() < 1e-9, "symétrique");
            }
        }
    }
}

#[cfg(test)]
mod tests_geometrie_impossible {
    use super::*;

    /// Une profondeur d'encoche de disque axial ou un stator inversé ne doit
    /// pas aller jusqu'au solveur (« matrice singulière ») : message clair.
    #[test]
    fn controle_avant_calcul() {
        let mut s = MotorSpec::default();
        assert!(s.geometry_problem().is_none());
        s.slot_depth = 16.4;
        assert!(s.geometry_problem().unwrap().contains("profondeur d'encoche"));
        s.slot_depth = 0.0;
        s.stator_ir = s.stator_or + 6.0;
        assert!(s.geometry_problem().unwrap().contains("trop mince"));
        s.topology = Topology::RadialInner;
        assert!(s.geometry_problem().is_none());
        s.topology = Topology::Axial;
        s.slot_depth = 16.4;
        assert!(s.geometry_problem().is_none());
    }
}

