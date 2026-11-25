//! Imports et exports : passerelles vers les outils libres et la fabrication.
//!
//! Exports : script FEMM (.lua), géométrie Gmsh (.geo), coupe DXF (R12),
//! gabarit de pose SVG à l'échelle 1:1, modèle 3D STL, tableau LaTeX,
//! rapport HTML autonome, scripts Octave/MATLAB (.m) et Python (.py).
//! Imports : angles d'aimants (CSV), résultats de simulation externe (JSON),
//! mesures de banc (CSV), projets SIMUMSAP (.txt).

use crate::analysis::Report;
use crate::compute::{Results, VariantResult};
use crate::geometry::{Magnetisation, MotorSpec};
use crate::magnets::{self, PieceKind};
use std::f64::consts::PI;
use std::fmt::Write as _;

// ---------------------------------------------------------------------
// Géométrie commune (mm, angles en radians)
// ---------------------------------------------------------------------

/// Géométrie commune des exports (mm, rad).
pub struct Geo {
    /// Rayon intérieur du stator.
    pub r_in: f64,   // alésage intérieur du stator (axe)
    /// Rayon du fond d'encoche.
    pub r_sb: f64,   // fond d'encoche
    /// Rayon d'alésage côté entrefer.
    pub r_s: f64,    // rayon d'alésage côté entrefer
    /// Rayon de la face intérieure des aimants.
    pub r_m: f64,    // face intérieure des aimants
    /// Rayon de la face extérieure des aimants.
    pub r_r: f64,    // face extérieure des aimants
    /// Rayon extérieur de la cloche.
    pub r_o: f64,    // extérieur de la cloche rotor
    /// Encoches : (angle de début, angle de fin).
    pub slots: Vec<(f64, f64)>,
    /// (t1, t2, polarité/aimantation : +1 nord vers l'axe…, direction ψ éventuelle)
    pub magnets: Vec<(f64, f64, f64, Option<f64>)>,
    /// Longueur active.
    pub len: f64,
}

/// Géométrie pour une position du rotor `theta_r` [rad].
pub fn geo(spec: &MotorSpec, theta_r: f64) -> Geo {
    let rg = spec.rings();
    let q = spec.n_slots;
    let beta = rg.beta_open;
    let slots = (0..q)
        .map(|i| {
            let c = (i as f64 + 0.5) * 2.0 * PI / q as f64;
            (c - beta / 2.0, c + beta / 2.0)
        })
        .collect();
    let magnets = magnets::pieces(spec)
        .iter()
        .map(|p| match p.kind {
            PieceKind::Radial { s } => (p.t1 + theta_r, p.t2 + theta_r, s, None),
            PieceKind::Fixed { psi } => {
                let mr = (0.5 * (p.t1 + p.t2) - psi).cos();
                (p.t1 + theta_r, p.t2 + theta_r, mr.signum(), Some(psi + theta_r))
            }
        })
        .collect();
    Geo {
        r_in: rg.yoke.1 * 1e3,
        r_sb: rg.sb * 1e3,
        r_s: rg.bore * 1e3,
        r_m: rg.mag.0 * 1e3,
        r_r: rg.mag.1 * 1e3,
        r_o: rg.rotor_yoke.1 * 1e3,
        slots,
        magnets,
        len: spec.stack_len,
    }
}

fn pol(r: f64, a: f64) -> (f64, f64) {
    (r * a.cos(), r * a.sin())
}

// ---------------------------------------------------------------------
// FEMM (.lua) — modèle complet + balayage du cogging
// ---------------------------------------------------------------------

/// Script Lua FEMM 4.2 : modèle complet et balayage du cogging.
pub fn femm_lua(spec: &MotorSpec, span_deg: f64, step_deg: f64) -> String {
    let g = geo(spec, 0.0);
    let mut s = String::new();
    let hc = spec.br_magnet / (4e-7 * PI * spec.mur_magnet);
    let _ = writeln!(s, "-- FractLab -> FEMM 4.2 : {} ({}N{}P)", spec.name, spec.n_slots, spec.n_poles);
    let _ = writeln!(s, "-- Ouvrir dans FEMM : File > Open Lua Script. Résultats : fractlab_cogging.csv");
    let _ = writeln!(s, "-- Géométrie idéalisée identique au modèle FractLab (encoches droites, largeur = ouverture).");
    let _ = writeln!(s, "newdocument(0)\nmi_probdef(0, 'millimeters', 'planar', 1e-8, {}, 30)", g.len);
    let _ = writeln!(s, "mi_addmaterial('Air', 1, 1, 0)");
    let _ = writeln!(s, "mi_addmaterial('Fer', 5000, 5000, 0)");
    let _ = writeln!(s, "mi_addmaterial('Aimant', {m}, {m}, {hc:.1})", m = spec.mur_magnet);
    let _ = writeln!(s, "mi_addboundprop('A0', 0, 0, 0, 0, 0, 0, 0, 0, 0)");
    let arc = |s: &mut String, r: f64, a1: f64, a2: f64, grp: i32| {
        let (x1, y1) = pol(r, a1);
        let (x2, y2) = pol(r, a2);
        let _ = writeln!(s, "mi_drawarc({x1:.6}, {y1:.6}, {x2:.6}, {y2:.6}, {:.6}, 1)", (a2 - a1).to_degrees());
        if grp > 0 {
            let (xm, ym) = pol(r, 0.5 * (a1 + a2));
            let _ = writeln!(s, "mi_selectarcsegment({xm:.6}, {ym:.6}) mi_setarcsegmentprop(1, '', 0, {grp}) mi_clearselected()");
        }
    };
    let line = |s: &mut String, r1: f64, r2: f64, a: f64, grp: i32| {
        let (x1, y1) = pol(r1, a);
        let (x2, y2) = pol(r2, a);
        let _ = writeln!(s, "mi_drawline({x1:.6}, {y1:.6}, {x2:.6}, {y2:.6})");
        if grp > 0 {
            let (xm, ym) = pol(0.5 * (r1 + r2), a);
            let _ = writeln!(s, "mi_selectsegment({xm:.6}, {ym:.6}) mi_setsegmentprop('', 0, 1, 0, {grp}) mi_clearselected()");
        }
    };
    // stator : cercle intérieur, dents et encoches
    arc(&mut s, g.r_in, 0.0, PI, 0);
    arc(&mut s, g.r_in, PI, 2.0 * PI, 0);
    let q = g.slots.len();
    for i in 0..q {
        let (a1, a2) = g.slots[i];
        let next = g.slots[(i + 1) % q].0 + if i + 1 == q { 2.0 * PI } else { 0.0 };
        line(&mut s, g.r_sb, g.r_s, a1, 0);
        line(&mut s, g.r_sb, g.r_s, a2, 0);
        arc(&mut s, g.r_sb, a1, a2, 0);
        arc(&mut s, g.r_s, a2, next, 0); // tête de dent
    }
    // fermeture des encoches côté entrefer (même matériau air : segment fictif)
    for &(a1, a2) in &g.slots {
        arc(&mut s, g.r_s, a1, a2, 0);
    }
    // rotor (groupe 2) : aimants + cloche
    arc(&mut s, g.r_m, 0.0, PI, 2);
    arc(&mut s, g.r_m, PI, 2.0 * PI, 2);
    arc(&mut s, g.r_r, 0.0, PI, 2);
    arc(&mut s, g.r_r, PI, 2.0 * PI, 2);
    arc(&mut s, g.r_o, 0.0, PI, 2);
    arc(&mut s, g.r_o, PI, 2.0 * PI, 2);
    for &(t1, t2, _, _) in &g.magnets {
        line(&mut s, g.r_m, g.r_r, t1, 2);
        line(&mut s, g.r_m, g.r_r, t2, 2);
    }
    // frontière extérieure
    let rb = g.r_o * 1.3;
    arc(&mut s, rb, 0.0, PI, 0);
    arc(&mut s, rb, PI, 2.0 * PI, 0);
    let (bx, by) = pol(rb, 0.5 * PI);
    let _ = writeln!(s, "mi_selectarcsegment({bx:.6}, {by:.6}) mi_selectarcsegment({:.6}, {:.6}) mi_setarcsegmentprop(1, 'A0', 0, 0) mi_clearselected()", bx, -by);
    // étiquettes
    let label = |s: &mut String, r: f64, a: f64, mat: &str, dir: f64, grp: i32| {
        let (x, y) = pol(r, a);
        let _ = writeln!(s, "mi_addblocklabel({x:.6}, {y:.6}) mi_selectlabel({x:.6}, {y:.6}) mi_setblockprop('{mat}', 1, 0, '', {dir:.4}, {grp}, 0) mi_clearselected()");
    };
    label(&mut s, 0.5 * (g.r_in + g.r_sb), 0.0, "Fer", 0.0, 1);
    for i in 0..q {
        let (a1, a2) = g.slots[i];
        label(&mut s, 0.5 * (g.r_sb + g.r_s), 0.5 * (a1 + a2), "Air", 0.0, 1);
        let next = g.slots[(i + 1) % q].0 + if i + 1 == q { 2.0 * PI } else { 0.0 };
        label(&mut s, 0.5 * (g.r_sb + g.r_s), 0.5 * (a2 + next), "Fer", 0.0, 1);
    }
    label(&mut s, 0.5 * (g.r_s + g.r_m), 0.01, "Air", 0.0, 0);
    label(&mut s, 0.5 * g.r_in, 0.0, "Air", 0.0, 0);
    label(&mut s, 0.5 * (g.r_o + rb), 0.0, "Air", 0.0, 0);
    label(&mut s, 0.5 * (g.r_r + g.r_o), 0.0, "Fer", 0.0, 2);
    let n = g.magnets.len();
    for (k, &(t1, t2, sgn, psi)) in g.magnets.iter().enumerate() {
        let tc = 0.5 * (t1 + t2);
        let dir = match psi {
            Some(p) => p.to_degrees(),
            None => if sgn > 0.0 { tc.to_degrees() } else { tc.to_degrees() + 180.0 },
        };
        label(&mut s, 0.5 * (g.r_m + g.r_r), tc, "Aimant", dir, 2);
        // intervalles entre aimants : air
        let t_next = g.magnets[(k + 1) % n].0 + if k + 1 == n { 2.0 * PI } else { 0.0 };
        if t_next - t2 > 1e-6 {
            label(&mut s, 0.5 * (g.r_m + g.r_r), 0.5 * (t2 + t_next), "Air", 0.0, 2);
        }
    }
    let _ = writeln!(s, "mi_saveas('fractlab_{}.fem')", spec.name);
    let _ = writeln!(s, "f = openfile('fractlab_cogging.csv', 'w') write(f, 'theta_deg;couple_Nm\\n')");
    let npts = (span_deg / step_deg).round() as usize;
    let _ = writeln!(s, "for k = 0, {} do", npts.saturating_sub(1));
    let _ = writeln!(s, "  mi_analyze(1) mi_loadsolution()");
    let _ = writeln!(s, "  mo_groupselectblock(2) t = mo_blockintegral(22) mo_close()");
    let _ = writeln!(s, "  write(f, k * {step_deg}, ';', t, '\\n')");
    let _ = writeln!(s, "  mi_selectgroup(2) mi_moverotate(0, 0, {step_deg}) mi_clearselected()");
    let _ = writeln!(s, "end\nclosefile(f)");
    s
}

// ---------------------------------------------------------------------
// Gmsh (.geo) — noyau OpenCASCADE, surfaces physiques nommées
// ---------------------------------------------------------------------

/// Géométrie Gmsh (noyau OpenCASCADE) avec surfaces physiques nommées.
pub fn gmsh_geo(spec: &MotorSpec) -> String {
    let g = geo(spec, 0.0);
    let mut s = String::new();
    let _ = writeln!(s, "// FractLab -> Gmsh : {} ({}N{}P), unités mm", spec.name, spec.n_slots, spec.n_poles);
    let _ = writeln!(s, "SetFactory(\"OpenCASCADE\");\nlc = {:.4};", (spec.airgap / 3.0).max(0.02));
    // identifiants explicites dans une plage réservée (≥ 1000) pour ne pas
    // entrer en collision avec ceux qu'OpenCASCADE attribue aux disques
    let mut tag = 1000;
    let mut sector = |s: &mut String, r1: f64, r2: f64, a1: f64, a2: f64| -> usize {
        // secteur annulaire : 4 points, 2 droites, 2 arcs, 1 surface
        let t = tag;
        tag += 1;
        let p = [pol(r1, a1), pol(r2, a1), pol(r2, a2), pol(r1, a2)];
        for (k, (x, y)) in p.iter().enumerate() {
            let _ = writeln!(s, "Point({}) = {{{x:.12}, {y:.12}, 0, lc}};", t * 10 + k);
        }
        let _ = writeln!(s, "Point({}) = {{0, 0, 0, lc}};", t * 10 + 4);
        let _ = writeln!(s, "Line({a}) = {{{p0}, {p1}}}; Circle({b}) = {{{p1}, {c}, {p2}}}; Line({cc}) = {{{p2}, {p3}}}; Circle({d}) = {{{p3}, {c}, {p0}}};",
            a = t * 10, b = t * 10 + 1, cc = t * 10 + 2, d = t * 10 + 3, p0 = t * 10, p1 = t * 10 + 1, p2 = t * 10 + 2, p3 = t * 10 + 3, c = t * 10 + 4);
        let _ = writeln!(s, "Curve Loop({t}) = {{{}, {}, {}, {}}}; Plane Surface({t}) = {{{t}}};", t * 10, t * 10 + 1, t * 10 + 2, t * 10 + 3);
        t
    };
    let _ = writeln!(s, "Disk(9001) = {{0, 0, 0, {}}}; Disk(9002) = {{0, 0, 0, {}}};", g.r_s.max(g.r_in), g.r_s.min(g.r_in));
    let _ = writeln!(s, "stator() = BooleanDifference{{ Surface{{9001}}; Delete; }}{{ Surface{{9002}}; Delete; }};");
    // toutes les formes élémentaires d'abord, les opérations booléennes ensuite
    let mut slot_tags = vec![];
    for &(a1, a2) in &g.slots {
        slot_tags.push(sector(&mut s, g.r_sb, g.r_s * 1.0005, a1, a2));
    }
    let mut mag_n = vec![];
    let mut mag_s = vec![];
    for &(t1, t2, sgn, _) in &g.magnets {
        let t = sector(&mut s, g.r_m, g.r_r, t1, t2);
        if sgn > 0.0 { mag_n.push(t) } else { mag_s.push(t) }
    }
    let list = |v: &[usize]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
    let _ = writeln!(s, "fer_stator() = BooleanDifference{{ Surface{{stator()}}; Delete; }}{{ Surface{{{}}}; }};", list(&slot_tags));
    let _ = writeln!(s, "Disk(9003) = {{0, 0, 0, {}}}; Disk(9004) = {{0, 0, 0, {}}};", g.r_o.max(g.r_r), g.r_o.min(g.r_r));
    let _ = writeln!(s, "cloche() = BooleanDifference{{ Surface{{9003}}; Delete; }}{{ Surface{{9004}}; Delete; }};");
    let _ = writeln!(s, "Disk(9005) = {{0, 0, 0, {}}};", g.r_o.max(g.r_in).max(g.r_r) * 1.3);
    let _ = writeln!(s, "air() = BooleanFragments{{ Surface{{9005}}; Delete; }}{{ Surface{{fer_stator(), {}, {}, {}, cloche()}}; Delete; }};", list(&slot_tags), list(&mag_n), list(&mag_s));
    let _ = writeln!(s, "Physical Surface(\"fer_stator\") = {{fer_stator()}};");
    let _ = writeln!(s, "Physical Surface(\"encoches\") = {{{}}};", list(&slot_tags));
    let _ = writeln!(s, "Physical Surface(\"aimants_N\") = {{{}}};", list(&mag_n));
    let _ = writeln!(s, "Physical Surface(\"aimants_S\") = {{{}}};", list(&mag_s));
    let _ = writeln!(s, "Physical Surface(\"cloche_rotor\") = {{cloche()}};");
    let _ = writeln!(s, "// Le reste du domaine (air, entrefer) : surfaces restantes de air()");
    s
}

// ---------------------------------------------------------------------
// DXF R12 (CAO, découpe laser / jet d'eau)
// ---------------------------------------------------------------------

/// Coupe 2D au format DXF R12 (calques STATOR, AIMANTS, ROTOR).
pub fn dxf(spec: &MotorSpec) -> String {
    let g = geo(spec, 0.0);
    let mut s = String::from("0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n");
    let arc = |s: &mut String, layer: &str, r: f64, a1: f64, a2: f64| {
        let _ = write!(s, "0\nARC\n8\n{layer}\n10\n0.0\n20\n0.0\n30\n0.0\n40\n{r:.6}\n50\n{:.6}\n51\n{:.6}\n", a1.to_degrees(), a2.to_degrees());
    };
    let circle = |s: &mut String, layer: &str, r: f64| {
        let _ = write!(s, "0\nCIRCLE\n8\n{layer}\n10\n0.0\n20\n0.0\n30\n0.0\n40\n{r:.6}\n");
    };
    let line = |s: &mut String, layer: &str, r1: f64, r2: f64, a: f64| {
        let (x1, y1) = pol(r1, a);
        let (x2, y2) = pol(r2, a);
        let _ = write!(s, "0\nLINE\n8\n{layer}\n10\n{x1:.6}\n20\n{y1:.6}\n30\n0.0\n11\n{x2:.6}\n21\n{y2:.6}\n31\n0.0\n");
    };
    circle(&mut s, "STATOR", g.r_in);
    let q = g.slots.len();
    for i in 0..q {
        let (a1, a2) = g.slots[i];
        let next = g.slots[(i + 1) % q].0;
        line(&mut s, "STATOR", g.r_sb, g.r_s, a1);
        line(&mut s, "STATOR", g.r_sb, g.r_s, a2);
        arc(&mut s, "STATOR", g.r_sb, a1, a2);
        arc(&mut s, "STATOR", g.r_s, a2, next);
    }
    for &(t1, t2, _, _) in &g.magnets {
        arc(&mut s, "AIMANTS", g.r_m, t1, t2);
        arc(&mut s, "AIMANTS", g.r_r, t1, t2);
        line(&mut s, "AIMANTS", g.r_m, g.r_r, t1);
        line(&mut s, "AIMANTS", g.r_m, g.r_r, t2);
    }
    circle(&mut s, "ROTOR", g.r_r);
    circle(&mut s, "ROTOR", g.r_o);
    s.push_str("0\nENDSEC\n0\nEOF\n");
    s
}

// ---------------------------------------------------------------------
// SVG — gabarit de pose des aimants, échelle 1:1 (imprimer à 100 %)
// ---------------------------------------------------------------------

/// Gabarit de pose des aimants à l'échelle 1:1 (SVG en millimètres).
pub fn svg_gabarit(spec: &MotorSpec) -> String {
    let g = geo(spec, 0.0);
    let m = g.r_o + 14.0;
    let size = 2.0 * m;
    let mut s = String::new();
    let _ = writeln!(s, r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size:.2}mm" height="{:.2}mm" viewBox="{:.2} {:.2} {size:.2} {:.2}" font-family="sans-serif">"#, size + 12.0, -m, -m, size + 12.0);
    let _ = writeln!(s, r#"<title>FractLab — gabarit de pose {} ({}N{}P)</title>"#, spec.name, spec.n_slots, spec.n_poles);
    let _ = writeln!(s, r##"<circle cx="0" cy="0" r="{:.4}" fill="none" stroke="#000" stroke-width="0.15"/>"##, g.r_r);
    let _ = writeln!(s, r##"<circle cx="0" cy="0" r="{:.4}" fill="none" stroke="#000" stroke-width="0.15" stroke-dasharray="1 1"/>"##, g.r_m);
    let _ = writeln!(s, r##"<circle cx="0" cy="0" r="{:.4}" fill="none" stroke="#888" stroke-width="0.1"/>"##, g.r_o);
    // graduations tous les degrés (longues tous les 10°)
    for d in 0..360 {
        let a = (d as f64).to_radians();
        let l = if d % 10 == 0 { 3.0 } else if d % 5 == 0 { 2.0 } else { 1.0 };
        let (x1, y1) = pol(g.r_o + 0.5, a);
        let (x2, y2) = pol(g.r_o + 0.5 + l, a);
        let _ = writeln!(s, r##"<line x1="{x1:.3}" y1="{:.3}" x2="{x2:.3}" y2="{:.3}" stroke="#000" stroke-width="0.12"/>"##, -y1, -y2);
        if d % 30 == 0 {
            let (x, y) = pol(g.r_o + 6.5, a);
            let _ = writeln!(s, r#"<text x="{x:.3}" y="{:.3}" font-size="2.2" text-anchor="middle" dominant-baseline="middle">{d}°</text>"#, -y);
        }
    }
    let angles = spec.magnet_angles_deg();
    let pitch = 360.0 / angles.len() as f64;
    for (k, &(t1, t2, sgn, _)) in g.magnets.iter().enumerate() {
        let col = if sgn > 0.0 { "#d33" } else { "#36c" };
        let (x1, y1) = pol(g.r_m, t1);
        let (x2, y2) = pol(g.r_r, t1);
        let (x3, y3) = pol(g.r_r, t2);
        let (x4, y4) = pol(g.r_m, t2);
        let _ = writeln!(s, r#"<path d="M{x1:.4},{:.4} L{x2:.4},{:.4} A{r:.4},{r:.4} 0 0 0 {x3:.4},{:.4} L{x4:.4},{:.4} A{rm:.4},{rm:.4} 0 0 1 {x1:.4},{:.4} Z" fill="{col}" fill-opacity="0.25" stroke="{col}" stroke-width="0.2"/>"#,
            -y1, -y2, -y3, -y4, -y1, r = g.r_r, rm = g.r_m);
        if let Some(a) = angles.get(k) {
            let tc = a.to_radians();
            let (xl, yl) = pol(g.r_m - 0.3, tc);
            let (xo, yo) = pol(g.r_o + 0.5, tc);
            let _ = writeln!(s, r#"<line x1="{xl:.3}" y1="{:.3}" x2="{xo:.3}" y2="{:.3}" stroke="{col}" stroke-width="0.15"/>"#, -yl, -yo);
            let (xt, yt) = pol(g.r_m - 2.6, tc);
            let _ = writeln!(s, r#"<text x="{xt:.3}" y="{:.3}" font-size="1.6" text-anchor="middle" dominant-baseline="middle" fill="{col}">{} {}</text>"#, -yt, k + 1, if sgn > 0.0 { "N" } else { "S" });
            let (xa, ya) = pol(g.r_m - 4.6, tc);
            let _ = writeln!(s, r#"<text x="{xa:.3}" y="{:.3}" font-size="1.1" text-anchor="middle" dominant-baseline="middle">{:.2}° ({:+.2})</text>"#, -ya, a, a - k as f64 * pitch);
        }
    }
    // croix de centrage + règle de contrôle d'échelle
    let _ = writeln!(s, r##"<path d="M-3,0 H3 M0,-3 V3" stroke="#000" stroke-width="0.15"/>"##);
    let y = m + 4.0;
    let _ = writeln!(s, r##"<rect x="{:.2}" y="{y:.2}" width="50" height="2" fill="none" stroke="#000" stroke-width="0.2"/>"##, -m + 2.0);
    for k in 0..=5 {
        let x = -m + 2.0 + 10.0 * k as f64;
        let _ = writeln!(s, r##"<line x1="{x:.2}" y1="{y:.2}" x2="{x:.2}" y2="{:.2}" stroke="#000" stroke-width="0.2"/>"##, y + 3.0);
    }
    let _ = writeln!(s, r##"<text x="{:.2}" y="{:.2}" font-size="2.2">50 mm — imprimer à 100 % (vérifier cette règle) · {} · Alain VALETTE-CLARY · ΩiA-Group</text>"##, -m + 55.0, y + 2.0, spec.name);
    s.push_str("</svg>\n");
    s
}

// ---------------------------------------------------------------------
// STL binaire (mm) — stator, aimants, cloche
// ---------------------------------------------------------------------

/// Solide STL binaire fermé (mm) : `part` = "stator", "aimants" ou "cloche".
pub fn stl(spec: &MotorSpec, part: &str) -> Vec<u8> {
    let g = geo(spec, 0.0);
    let mut tris: Vec<[[f32; 3]; 3]> = Vec::new();
    let mut ring = |ra: f64, rb: f64, a0: f64, a1: f64, z0: f64, z1: f64, n: usize| {
        let (r1, r2) = (ra.min(rb), ra.max(rb));
        let p = |r: f64, a: f64, z: f64| [(r * a.cos()) as f32, (r * a.sin()) as f32, z as f32];
        let mut quad = |a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]| {
            tris.push([a, b, c]);
            tris.push([a, c, d]);
        };
        for k in 0..n {
            let t0 = a0 + (a1 - a0) * k as f64 / n as f64;
            let t1 = a0 + (a1 - a0) * (k + 1) as f64 / n as f64;
            quad(p(r1, t0, z1), p(r2, t0, z1), p(r2, t1, z1), p(r1, t1, z1));
            quad(p(r1, t0, z0), p(r1, t1, z0), p(r2, t1, z0), p(r2, t0, z0));
            quad(p(r2, t0, z0), p(r2, t1, z0), p(r2, t1, z1), p(r2, t0, z1));
            quad(p(r1, t0, z0), p(r1, t0, z1), p(r1, t1, z1), p(r1, t1, z0));
        }
        if (a1 - a0 - 2.0 * PI).abs() > 1e-9 {
            quad(p(r1, a0, z0), p(r2, a0, z0), p(r2, a0, z1), p(r1, a0, z1));
            quad(p(r1, a1, z0), p(r1, a1, z1), p(r2, a1, z1), p(r2, a1, z0));
        }
    };
    let (z0, z1) = (0.0, g.len);
    if spec.topology == crate::geometry::Topology::Axial {
        // machine axiale : disques réels (rayons r_in…r_out, cotes axiales)
        let (ri, ro) = (spec.axial.r_in, spec.axial.r_out);
        let (yk, dep, gp, hm, ry) = (spec.yoke_th, spec.slot_depth_m() * 1e3, spec.airgap, spec.magnet_th, spec.rotor_yoke_th);
        let gaps = spec.axial.gaps.clamp(1, 2);
        // stator : culasse centrale, dents de part et d'autre (côtés : +z, et −z si deux entrefers)
        let sides: Vec<f64> = if gaps == 2 { vec![1.0, -1.0] } else { vec![1.0] };
        match part {
            "stator" => {
                ring(ri, ro, 0.0, 2.0 * PI, -yk / 2.0, yk / 2.0, 180);
                for &sg in &sides {
                    let q = g.slots.len();
                    let rm = 0.5 * (ri + ro);
                    let bw = spec.slot_w.max(spec.slot_open) / rm;
                    for i in 0..q {
                        let c0 = (i as f64 + 0.5) * 2.0 * PI / q as f64;
                        let c1 = (i as f64 + 1.5) * 2.0 * PI / q as f64;
                        // les dents pénètrent de 0,05 mm dans la culasse : volumes fermés qui se recouvrent
                        let (za, zb) = if sg > 0.0 { (yk / 2.0 - 0.05, yk / 2.0 + dep) } else { (-yk / 2.0 - dep, -yk / 2.0 + 0.05) };
                        ring(ri, ro, c0 + bw / 2.0, c1 - bw / 2.0, za, zb, 12);
                    }
                }
            }
            "aimants" => {
                for &sg in &sides {
                    let z = yk / 2.0 + dep + gp;
                    let (za, zb) = if sg > 0.0 { (z, z + hm) } else { (-z - hm, -z) };
                    for &(t1, t2, _, _) in &g.magnets {
                        ring(ri, ro, t1, t2, za, zb, 12);
                    }
                }
            }
            _ => {
                for &sg in &sides {
                    let z = yk / 2.0 + dep + gp + hm;
                    let (za, zb) = if sg > 0.0 { (z, z + ry) } else { (-z - ry, -z) };
                    ring(ri * 0.8, ro, 0.0, 2.0 * PI, za, zb, 180);
                }
            }
        }
    } else {
    match part {
        "stator" => {
            // Solide unique et étanche : culasse + dents partagent leurs sommets
            // sur le cercle de fond d'encoche (aucune face interne).
            drop(ring);
            let p = |r: f64, a: f64, z: f64| [(r * a.cos()) as f32, (r * a.sin()) as f32, z as f32];
            let mut quad = |a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]| {
                tris.push([a, b, c]);
                tris.push([a, c, d]);
            };
            let q = g.slots.len();
            // segments angulaires : (a0, a1, est_une_dent)
            let mut segs: Vec<(f64, f64, bool)> = Vec::new();
            for i in 0..q {
                let (s1, s2) = g.slots[i];
                let next = g.slots[(i + 1) % q].0 + if i + 1 == q { 2.0 * PI } else { 0.0 };
                for (a, b, tooth, n) in [(s1, s2, false, 6usize), (s2, next, true, 24usize)] {
                    for k in 0..n {
                        segs.push((a + (b - a) * k as f64 / n as f64, a + (b - a) * (k + 1) as f64 / n as f64, tooth));
                    }
                }
            }
            for (k, &(t0, t1, tooth)) in segs.iter().enumerate() {
                // culasse : dessus, dessous, face intérieure
                quad(p(g.r_in, t0, z1), p(g.r_sb, t0, z1), p(g.r_sb, t1, z1), p(g.r_in, t1, z1));
                quad(p(g.r_in, t0, z0), p(g.r_in, t1, z0), p(g.r_sb, t1, z0), p(g.r_sb, t0, z0));
                quad(p(g.r_in, t0, z0), p(g.r_in, t0, z1), p(g.r_in, t1, z1), p(g.r_in, t1, z0));
                if tooth {
                    quad(p(g.r_sb, t0, z1), p(g.r_s, t0, z1), p(g.r_s, t1, z1), p(g.r_sb, t1, z1));
                    quad(p(g.r_sb, t0, z0), p(g.r_sb, t1, z0), p(g.r_s, t1, z0), p(g.r_s, t0, z0));
                    quad(p(g.r_s, t0, z0), p(g.r_s, t1, z0), p(g.r_s, t1, z1), p(g.r_s, t0, z1));
                    // flancs de dent aux transitions encoche ↔ dent
                    let prev_tooth = segs[(k + segs.len() - 1) % segs.len()].2;
                    let next_tooth = segs[(k + 1) % segs.len()].2;
                    if !prev_tooth {
                        quad(p(g.r_sb, t0, z0), p(g.r_s, t0, z0), p(g.r_s, t0, z1), p(g.r_sb, t0, z1));
                    }
                    if !next_tooth {
                        quad(p(g.r_sb, t1, z0), p(g.r_sb, t1, z1), p(g.r_s, t1, z1), p(g.r_s, t1, z0));
                    }
                } else {
                    // fond d'encoche
                    quad(p(g.r_sb, t0, z0), p(g.r_sb, t1, z0), p(g.r_sb, t1, z1), p(g.r_sb, t0, z1));
                }
            }
        }
        "aimants" => {
            for &(t1, t2, _, _) in &g.magnets {
                ring(g.r_m, g.r_r, t1, t2, z0, z1, 16);
            }
        }
        _ => ring(g.r_r, g.r_o, 0.0, 2.0 * PI, z0, z1 + 1.0, 240),
    }
    }
    // orientation : normales sortantes (volume signé positif)
    let vol: f64 = tris.iter().map(|t| {
        let (a, b, c) = (t[0], t[1], t[2]);
        (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0]) + a[2] * (b[0] * c[1] - b[1] * c[0])) as f64
    }).sum();
    if vol < 0.0 {
        for t in tris.iter_mut() {
            t.swap(1, 2);
        }
    }
    let mut out = vec![0u8; 80];
    let head = format!("FractLab v{} {} {}", env!("CARGO_PKG_VERSION"), spec.name, part);
    out[..head.len().min(80)].copy_from_slice(&head.as_bytes()[..head.len().min(80)]);
    out.extend_from_slice(&(tris.len() as u32).to_le_bytes());
    for t in &tris {
        let u = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
        let v = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-20);
        for c in [n[0] / l, n[1] / l, n[2] / l] {
            out.extend_from_slice(&c.to_le_bytes());
        }
        for p in t {
            for c in p {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        out.extend_from_slice(&[0, 0]);
    }
    out
}

// ---------------------------------------------------------------------
// LaTeX, HTML, Octave, Python
// ---------------------------------------------------------------------

fn tex_esc(s: &str) -> String {
    s.replace('_', "\\_").replace('%', "\\%").replace('&', "\\&").replace('#', "\\#")
}

/// Tableau LaTeX (booktabs) du rapport ou de la comparaison.
pub fn latex(r: &Results, cmp: &[VariantResult]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "% FractLab — tableau généré automatiquement ({})", r.spec.name);
    let _ = writeln!(s, "% nécessite \\usepackage{{booktabs}}");
    let _ = writeln!(s, "\\begin{{table}}[h]\\centering\\small");
    let rows: Vec<&Report> = if cmp.is_empty() { vec![&r.report] } else { cmp.iter().map(|v| &v.report).collect() };
    let refname = crate::repartition::reference_name();
    let b0 = cmp.iter().find(|v| v.name == refname).map(|v| v.report.cogging.cogging_pp_mnm);
    let _ = writeln!(s, "\\begin{{tabular}}{{lrrrrrrr}}\\toprule");
    let _ = writeln!(s, "Variante & Cogging pp [mN$\\cdot$m] & vs {} & Part dom. & Entropie & $F_{{max}}$ [N] & THD FEM & Autocorr. \\\\\\midrule", tex_esc(refname));
    for rep in rows {
        let vs = b0.map(|b| format!("{:+.1}\\,\\%", 100.0 * (rep.cogging.cogging_pp_mnm / b - 1.0))).unwrap_or("--".into());
        let _ = writeln!(
            s,
            "{} & {:.3} & {} & {:.1}\\,\\% & {:.3} & {:.3} & {:.2}\\,\\% & {} \\\\",
            tex_esc(&rep.variant),
            rep.cogging.cogging_pp_mnm,
            vs,
            rep.cogging.dominant_share_pct,
            rep.cogging.spectral_entropy,
            rep.force_max_n,
            rep.emf_thd_pct,
            rep.autocorr_max.map(|a| format!("{a:.3}")).unwrap_or("--".into())
        );
    }
    let _ = writeln!(s, "\\bottomrule\\end{{tabular}}");
    let _ = writeln!(
        s,
        "\\caption{{Moteur {}N{}P, entrefer {} mm, aimants {} mm (Br = {} T). Modèle 2D en sous-domaines polaires (FractLab), fenêtre de cogging {:.2}$^\\circ$.}}",
        r.spec.n_slots, r.spec.n_poles, r.spec.airgap, r.spec.magnet_th, r.spec.br_magnet, r.cog.metrics.span_deg
    );
    let _ = writeln!(s, "\\end{{table}}");
    s
}

fn svg_curves(series: &[(&str, &[f64], &[f64], &str)], xl: &str, yl: &str) -> String {
    let (w, h, ml, mb) = (720.0, 260.0, 60.0, 34.0);
    let mut xs = (f64::MAX, f64::MIN);
    let mut ys = (f64::MAX, f64::MIN);
    for (_, x, y, _) in series {
        for v in x.iter() {
            xs = (xs.0.min(*v), xs.1.max(*v));
        }
        for v in y.iter() {
            ys = (ys.0.min(*v), ys.1.max(*v));
        }
    }
    let sx = |v: f64| ml + (v - xs.0) / (xs.1 - xs.0).max(1e-30) * (w - ml - 10.0);
    let sy = |v: f64| 10.0 + (ys.1 - v) / (ys.1 - ys.0).max(1e-30) * (h - mb - 10.0);
    let mut s = format!(r##"<svg viewBox="0 0 {w} {h}" width="100%" xmlns="http://www.w3.org/2000/svg" font-size="11"><rect x="{ml}" y="10" width="{}" height="{}" fill="none" stroke="#999"/>"##, w - ml - 10.0, h - mb - 20.0);
    for k in 0..=4 {
        let v = ys.0 + (ys.1 - ys.0) * k as f64 / 4.0;
        let _ = write!(s, r##"<text x="{}" y="{:.1}" text-anchor="end">{:.4}</text>"##, ml - 4.0, sy(v) + 4.0, v);
        let v = xs.0 + (xs.1 - xs.0) * k as f64 / 4.0;
        let _ = write!(s, r##"<text x="{:.1}" y="{}" text-anchor="middle">{:.2}</text>"##, sx(v), h - mb + 14.0, v);
    }
    let _ = write!(s, r##"<text x="{}" y="{}" text-anchor="middle">{xl}</text><text x="12" y="{}" transform="rotate(-90 12 {})" text-anchor="middle">{yl}</text>"##, (w + ml) / 2.0, h - 4.0, h / 2.0, h / 2.0);
    for (k, (name, x, y, col)) in series.iter().enumerate() {
        let pts: Vec<String> = x.iter().zip(y.iter()).map(|(a, b)| format!("{:.1},{:.1}", sx(*a), sy(*b))).collect();
        let _ = write!(s, r##"<polyline fill="none" stroke="{col}" stroke-width="1.5" points="{}"/><text x="{}" y="{}" fill="{col}">{name}</text>"##, pts.join(" "), w - 120.0, 24.0 + 14.0 * k as f64);
    }
    s.push_str("</svg>");
    s
}

/// Rapport HTML autonome avec graphes SVG intégrés.
pub fn html_report(r: &Results, cmp: &[VariantResult]) -> String {
    let rep = &r.report;
    let mut s = String::new();
    let _ = write!(s, r##"<!doctype html><html lang="fr"><head><meta charset="utf-8"><title>FractLab — {n}</title>
<style>body{{font-family:system-ui,sans-serif;max-width:980px;margin:24px auto;padding:0 16px;color:#18202c;background:#f6f8fc}}
h1{{color:#0077aa}}h2{{color:#c06000;border-bottom:1px solid #ddd}}table{{border-collapse:collapse}}td,th{{padding:4px 10px;border-bottom:1px solid #ddd;text-align:right}}th:first-child,td:first-child{{text-align:left}}
.ok{{color:#1a8f4a}}.ko{{color:#c62828}}.card{{background:#fff;border-radius:10px;padding:12px 16px;margin:10px 0;box-shadow:0 1px 3px #0002}}footer{{color:#778;font-size:12px;margin-top:30px}}</style></head><body>
<div style="display:flex;align-items:center;gap:12px"><img alt="FractLab" width="56" height="56" src="data:image/png;base64,{logo}"/><div><b>FractLab</b> v{ver}</div></div><h1>FractLab — {n}</h1><p>Moteur {q}N{p}P · bobinage {w} · calcul {sec:.2} s</p>"##,
        n = r.spec.name, q = r.spec.n_slots, p = r.spec.n_poles, w = rep.winding, sec = r.seconds, logo = base64(crate::doc::LOGO_PNG), ver = env!("CARGO_PKG_VERSION"));
    let crit = crate::repartition::criteria();
    let cls = |b: Option<bool>| match b { Some(true) => "ok", Some(false) => "ko", None => "" };
    let _ = write!(s, r##"<div class="card"><h2>Synthèse</h2><table>
<tr><th>Cogging crête-à-crête</th><td>{:.4} mN·m</td></tr><tr><th>Cogging efficace</th><td>{:.4} mN·m</td></tr>
<tr><th>Harmonique dominante</th><td>{:.4} mN·m @ {:.0}/tour</td></tr><tr><th>Part de la dominante</th><td>{:.1} %</td></tr>
<tr><th>Entropie spectrale</th><td>{:.3}</td></tr><tr><th>Force radiale max{}</th><td class="{}">{:.4} N</td></tr>
<tr><th>Balourd{}</th><td class="{}">{:.4} mg·cm</td></tr><tr><th>THD FEM{}</th><td class="{}">{:.2} %</td></tr>
<tr><th>Interharmoniques FEM</th><td>{:.2} %</td></tr><tr><th>KV estimé (2D)</th><td>{:.0} tr/min/V</td></tr></table></div>"##,
        rep.cogging.cogging_pp_mnm, rep.cogging.cogging_rms_mnm, rep.cogging.dominant_mnm, rep.cogging.dominant_order_per_rev,
        rep.cogging.dominant_share_pct, rep.cogging.spectral_entropy,
        crit.force_max_n.map(|m| format!(" (seuil {m} N)")).unwrap_or_default(), cls(crit.force_max_n.map(|m| rep.force_max_n < m)), rep.force_max_n,
        crit.balourd_max_mg_cm.map(|m| format!(" (seuil {m} mg·cm)")).unwrap_or_default(), cls(crit.balourd_max_mg_cm.map(|m| rep.balourd_mg_cm < m)), rep.balourd_mg_cm,
        crit.thd_max_pct.map(|m| format!(" (seuil {m} %)")).unwrap_or_default(), cls(crit.thd_max_pct.map(|m| rep.emf_thd_pct <= m)), rep.emf_thd_pct,
        rep.emf_interharm_pct, rep.kv_estime);
    let t: Vec<f64> = r.cog.torque_nm.iter().map(|v| v * 1e3).collect();
    let _ = write!(s, r#"<div class="card"><h2>Couple de détente</h2>{}</div>"#, svg_curves(&[(&r.spec.name, &r.cog.angles_deg, &t, "#0099cc")], "θr [°]", "mN·m"));
    let _ = write!(
        s,
        r#"<div class="card"><h2>FEM à vide ({} tr/min)</h2>{}</div>"#,
        r.spec.rpm,
        svg_curves(&[("A", &r.emf.angles_deg, &r.emf.emf[0], "#0099cc"), ("B", &r.emf.angles_deg, &r.emf.emf[1], "#ff8800"), ("C", &r.emf.angles_deg, &r.emf.emf[2], "#44aa44")], "θr [°]", "V")
    );
    if !cmp.is_empty() {
        let _ = write!(s, r#"<div class="card"><h2>Comparaison des variantes</h2><table><tr><th>Variante</th><th>Cogging pp</th><th>Part dom.</th><th>Force max</th><th>THD</th></tr>"#);
        for v in cmp {
            let _ = write!(s, "<tr><td>{}</td><td>{:.3} mN·m</td><td>{:.1} %</td><td>{:.3} N</td><td>{:.2} %</td></tr>", v.name, v.report.cogging.cogging_pp_mnm, v.report.cogging.dominant_share_pct, v.report.force_max_n, v.report.emf_thd_pct);
        }
        s.push_str("</table></div>");
    }
    let spec_json = serde_json::to_string_pretty(&r.spec).unwrap_or_default();
    let _ = write!(s, r#"<div class="card"><h2>Paramètres</h2><pre>{}</pre></div>"#, spec_json.replace('<', "&lt;"));
    let _ = write!(s, "<footer>FractLab v{} — logiciel libre (GPL-3.0) · Alain VALETTE-CLARY · ΩiA-Group · INPI DSO2025023838 · modèle 2D en sous-domaines, fer idéal.</footer></body></html>", env!("CARGO_PKG_VERSION"));
    s
}

/// Encodage base64 (images intégrées aux rapports HTML).
pub fn base64(b: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::with_capacity(b.len() * 4 / 3 + 4);
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            if k <= c.len() {
                o.push(T[(n >> (18 - 6 * k) & 63) as usize] as char);
            } else {
                o.push('=');
            }
        }
    }
    o
}

fn arr(v: &[f64]) -> String {
    v.iter().map(|x| format!("{x:.9e}")).collect::<Vec<_>>().join(", ")
}

/// Script Octave/MATLAB contenant les données et les tracés.
pub fn octave_script(r: &Results) -> String {
    format!(
        "% FractLab -> Octave/MATLAB : {n}\n% Données calculées par FractLab (modèle en sous-domaines polaires).\ntheta_cog = [{a}];\ncouple_Nm = [{t}];\ntheta_fem = [{ae}];\neA = [{ea}]; eB = [{eb}]; eC = [{ec}];\n\nfigure; plot(theta_cog, couple_Nm*1e3); grid on;\nxlabel('\\theta_r [deg]'); ylabel('couple de détente [mN.m]'); title('{n}');\nfigure; plot(theta_fem, [eA; eB; eC]); grid on; legend('A','B','C');\nxlabel('\\theta_r [deg]'); ylabel('FEM [V]');\nfprintf('Cogging crête-à-crête : %.4f mN.m\\n', (max(couple_Nm)-min(couple_Nm))*1e3);\n",
        n = r.spec.name,
        a = arr(&r.cog.angles_deg),
        t = arr(&r.cog.torque_nm),
        ae = arr(&r.emf.angles_deg),
        ea = arr(&r.emf.emf[0]),
        eb = arr(&r.emf.emf[1]),
        ec = arr(&r.emf.emf[2])
    )
}

/// Script Python (numpy, matplotlib) contenant les données et les tracés.
pub fn python_script(r: &Results) -> String {
    format!(
        "#!/usr/bin/env python3\n# FractLab -> Python : {n}\nimport numpy as np\nimport matplotlib.pyplot as plt\n\ntheta_cog = np.array([{a}])\ncouple_Nm = np.array([{t}])\ntheta_fem = np.array([{ae}])\nemf = np.array([[{ea}], [{eb}], [{ec}]])\nspec = {spec}\n\nfig, ax = plt.subplots(2, 1, figsize=(9, 7))\nax[0].plot(theta_cog, couple_Nm * 1e3); ax[0].set_xlabel('θr [°]'); ax[0].set_ylabel('cogging [mN·m]'); ax[0].grid()\nfor k, p in enumerate('ABC'):\n    ax[1].plot(theta_fem, emf[k], label=p)\nax[1].set_xlabel('θr [°]'); ax[1].set_ylabel('FEM [V]'); ax[1].legend(); ax[1].grid()\nfig.suptitle('{n}'); plt.tight_layout(); plt.show()\nprint('cogging pp = %.4f mN·m' % ((couple_Nm.max() - couple_Nm.min()) * 1e3))\n",
        n = r.spec.name,
        a = arr(&r.cog.angles_deg),
        t = arr(&r.cog.torque_nm),
        ae = arr(&r.emf.angles_deg),
        ea = arr(&r.emf.emf[0]),
        eb = arr(&r.emf.emf[1]),
        ec = arr(&r.emf.emf[2]),
        spec = serde_json::to_string(&r.spec).unwrap_or_default().replace("null", "None").replace("true", "True").replace("false", "False")
    )
}

// ---------------------------------------------------------------------
// Imports
// ---------------------------------------------------------------------

fn numbers(line: &str) -> Vec<f64> {
    line.split(|c: char| c == ';' || c == ',' || c == '\t' || c.is_whitespace())
        .filter_map(|t| t.trim().parse::<f64>().ok())
        .collect()
}

/// CSV d'angles d'aimants : un angle par ligne, ou « n ; angle » (degrés).
pub fn import_angles_csv(text: &str) -> Result<Vec<f64>, String> {
    let mut out = Vec::new();
    for l in text.lines() {
        let v = numbers(l);
        match v.len() {
            0 => {}
            1 => out.push(v[0]),
            _ => out.push(*v.last().unwrap()),
        }
    }
    if out.len() < 2 {
        return Err("aucun angle lisible".into());
    }
    if out.len() % 2 != 0 {
        return Err(format!("{} angles : il faut un nombre pair d'aimants", out.len()));
    }
    Ok(out)
}

/// Courbe mesurée (banc d'essai) : deux colonnes x ; y, entête facultative.
pub fn import_curve_csv(text: &str) -> Result<Vec<[f64; 2]>, String> {
    let pts: Vec<[f64; 2]> = text.lines().map(numbers).filter(|v| v.len() >= 2).map(|v| [v[0], v[1]]).collect();
    if pts.len() < 2 {
        return Err("moins de deux points lisibles".into());
    }
    Ok(pts)
}

/// Résultats d'une simulation externe (JSON : `angles_deg`, `torque_Nm`,
/// `spec.variant`, `spec.magnet_angles`) : angles des aimants + courbe de couple.
pub struct PhiSimResult {
    /// Variante indiquée dans le fichier.
    pub variant: String,
    /// Angles des aimants [°], s'ils sont présents.
    pub magnet_angles: Option<Vec<f64>>,
    /// Courbe (angle [°], couple [N·m]).
    pub curve: Vec<[f64; 2]>,
}

/// Lit un fichier de résultats de simulation externe (JSON).
pub fn import_phi_sim_json(text: &str) -> Result<PhiSimResult, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let arrf = |x: &serde_json::Value| x.as_array().map(|a| a.iter().filter_map(|y| y.as_f64()).collect::<Vec<f64>>());
    let ang = v.get("angles_deg").and_then(arrf).ok_or("champ angles_deg absent")?;
    let tq = v.get("torque_Nm").and_then(arrf).ok_or("champ torque_Nm absent")?;
    let spec = v.get("spec");
    Ok(PhiSimResult {
        variant: spec.and_then(|s| s.get("variant")).and_then(|x| x.as_str()).unwrap_or("?").to_string(),
        magnet_angles: spec.and_then(|s| s.get("magnet_angles")).and_then(arrf),
        curve: ang.iter().zip(&tq).map(|(a, t)| [*a, *t]).collect(),
    })
}

/// Projet SIMUMSAP (.txt, Latin-1 accepté) : blocs « libellé / valeur ».
pub struct SimumsapImport {
    /// Spécification reconstituée (topologie, géométrie, bobinage, aimants).
    pub spec: MotorSpec,
    /// Tous les champs lus (libellé, valeur brute).
    pub fields: Vec<(String, String)>,
    /// Avertissement éventuel.
    pub warning: Option<String>,
    /// Matrice d'inductances calculée par SIMUMSAP [H], si présente.
    pub inductances: Option<Vec<Vec<f64>>>,
    /// Vitesse du projet [tr/min].
    pub rpm: f64,
}

/// Lit un projet SIMUMSAP (.txt, UTF-8 ou Latin-1) et reconstruit la machine.
pub fn import_simumsap(bytes: &[u8]) -> Result<SimumsapImport, String> {
    use crate::geometry::{CoilDef, Topology};
    let text: String = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(), // Latin-1
    };
    let text = text.replace('\r', "");
    let mut fields: Vec<(String, String)> = Vec::new();
    for block in text.split("\n\n") {
        let mut lines = block.lines().filter(|l| !l.trim().is_empty());
        if let Some(k) = lines.next() {
            let v: Vec<&str> = lines.collect();
            if !v.is_empty() {
                fields.push((k.trim().to_string(), v.join("\n")));
            }
        }
    }
    if fields.len() < 5 {
        return Err("format SIMUMSAP non reconnu".into());
    }
    let raw = |pat: &str| fields.iter().find(|(k, _)| k.to_lowercase().contains(&pat.to_lowercase())).map(|(_, v)| v.clone());
    let get = |pat: &str| raw(pat).and_then(|v| numbers(&v).first().copied());
    let mut s = MotorSpec { name: "SIMUMSAP".into(), variant: crate::repartition::REGULIERE.into(), ..Default::default() };
    let typ = get("Type de la machine").unwrap_or(0.0) as i32;
    let pp = get("paires de pôles").or(get("paires de p")).unwrap_or(7.0);
    s.n_poles = (2.0 * pp) as usize;
    s.n_slots = get("encoches statoriques").unwrap_or(12.0) as usize;
    s.airgap = get("Longueur de l'entrefer").unwrap_or(1e-3) * 1e3;
    s.br_magnet = get("Induction rémanente").or(get("Induction r")).unwrap_or(1.2);
    s.mur_magnet = 1.0; // hypothèse de SIMUMSAP : perméabilité des aimants = celle de l'air
    s.load.parallel_paths = get("enroulements en parall").unwrap_or(1.0).max(1.0);
    s.load.fill_factor = get("foisonnement").unwrap_or(0.45);
    let rpm = get("Vitesse de rotation").map(|w| w * 30.0 / PI).unwrap_or(1000.0);
    s.rpm = rpm;
    let isth_th = get("Epaisseur de l'isthme").unwrap_or(0.0);
    let mut warning = None;
    if typ == 1 {
        // flux axial, multi-tranches
        s.topology = Topology::Axial;
        let ri = get("Rayon interne du stator").unwrap_or(0.05);
        let re = get("Rayon externe du stator").unwrap_or(0.1);
        s.axial.r_in = ri * 1e3;
        s.axial.r_out = re * 1e3;
        s.axial.slices = get("subdivisions").unwrap_or(5.0).max(1.0) as usize;
        s.axial.gaps = (get("disques rotoriques").unwrap_or(1.0) as usize).clamp(1, 2);
        if get("disques rotoriques").unwrap_or(1.0) > 2.0 {
            warning = Some("Plus de deux disques rotoriques : FractLab modélise deux entrefers identiques ; les étages supplémentaires sont à additionner.".into());
        }
        s.magnet_arc_ratio = get("arc polaire des aimants au rayon interne").unwrap_or(0.8);
        s.axial.arc_ratio_out = get("arc polaire des aimants au rayon externe").unwrap_or(s.magnet_arc_ratio);
        s.magnet_th = get("Epaisseur axiale des aimants").unwrap_or(5e-3) * 1e3;
        s.slot_depth = get("Profondeur axiale d'une encoche").unwrap_or(0.02) * 1e3;
        // ouvertures données en radians au rayon interne → largeurs constantes
        let body = get("Ouverture d'une encoche au niveau du rayon interne").unwrap_or(0.1) * ri * 1e3;
        let open = get("Ouverture de l'isthme au niveau du rayon interne").unwrap_or(0.0) * ri * 1e3;
        s.slot_w = body;
        s.slot_open = if open > 0.0 && isth_th > 0.0 { open } else { body };
        s.tip_h = if open > 0.0 { isth_th * 1e3 } else { 0.0 };
        s.yoke_th = 0.3 * s.slot_depth;
        s.stack_len = (re - ri) * 1e3;
    } else {
        // flux radial (rotor intérieur dans les projets SIMUMSAP)
        s.topology = Topology::RadialInner;
        s.stack_len = get("Longueur axiale").unwrap_or(0.05) * 1e3;
        s.magnet_th = get("Epaisseur radiale des aimants").unwrap_or(5e-3) * 1e3;
        s.magnet_arc_ratio = get("Coefficient d'arc polaire").unwrap_or(0.8);
        let rfer = get("Rayon du fer rotorique").unwrap_or(0.05) * 1e3;
        s.stator_or = rfer + s.magnet_th + s.airgap; // alésage
        let h = get("Hauteur d'une encoche").unwrap_or(0.02) * 1e3;
        s.slot_depth = h;
        s.yoke_th = 0.3 * h;
        s.stator_ir = s.stator_or + h + s.yoke_th; // rayon extérieur du stator
        let body = get("Ouverture d'une encoche").unwrap_or(0.05) * s.stator_or;
        let open = get("Ouverture de l'isthme").unwrap_or(0.0) * s.stator_or;
        s.slot_w = body;
        s.slot_open = if open > 1e-3 && isth_th > 0.0 { open } else { body };
        s.tip_h = if open > 1e-3 { isth_th * 1e3 } else { 0.0 };
    }
    // bobinage : matrice topologique [phase, enroulement, bobine, aller, retour, sens, plan, spires, …]
    // Comme SIMUMSAP (SlotCurrDensity) : +N·i dans l'encoche « retour », −N·i
    // dans l'encoche « aller » ; la matrice ne décrit qu'une période machine
    // de Q/PGCD(Q, p) encoches, répétée PGCD(Q, p) fois.
    let q = s.n_slots;
    let periods = crate::geometry::gcd(q, (pp as usize).max(1)).max(1);
    let qp = q / periods;
    if let Some(m) = raw("Matrice topologique") {
        let mut base = Vec::new();
        for line in m.lines() {
            let v = numbers(line);
            if v.len() >= 8 && v[0] >= 1.0 && v[3] >= 1.0 && v[4] >= 1.0 {
                let (ph, aller, retour, n) = (v[0] as usize - 1, v[3] as usize - 1, v[4] as usize - 1, v[7]);
                if aller < qp && retour < qp && ph <= 2 {
                    base.push((ph, retour, aller, n));
                }
            }
        }
        let mut coils = Vec::new();
        for k in 0..periods {
            for &(ph, go, ret, n) in &base {
                let (go, ret) = (go + k * qp, ret + k * qp);
                // côtés : bobine autour d'une dent → moitiés voisines ; sinon encoche entière
                let (sa, sb) = if ret == (go + 1) % q { (1u8, 0u8) } else if go == (ret + 1) % q { (0, 1) } else { (2, 2) };
                coils.push(CoilDef { phase: ph, go, ret, turns: n, go_side: sa, ret_side: sb });
            }
        }
        if !coils.is_empty() {
            s.turns_per_coil = coils[0].turns;
            s.coils = coils;
        }
    }
    let inductances = raw("Matrice des inductances").map(|m| m.lines().map(numbers).filter(|r| !r.is_empty()).take(3).collect::<Vec<_>>()).filter(|m| m.len() == 3);
    s.magnetisation = Magnetisation::Radiale;
    s.custom_angles_deg = None;
    Ok(SimumsapImport { spec: s, fields, warning, inductances, rpm })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stl_bien_forme() {
        let s = MotorSpec::irregular_example();
        for part in ["stator", "aimants", "cloche"] {
            let b = stl(&s, part);
            let n = u32::from_le_bytes([b[80], b[81], b[82], b[83]]) as usize;
            assert!(n > 100);
            assert_eq!(b.len(), 84 + 50 * n, "{part}");
        }
    }

    #[test]
    fn dxf_et_svg_et_lua_non_vides() {
        let s = MotorSpec::irregular_example();
        let d = dxf(&s);
        assert!(d.starts_with("0\nSECTION") && d.ends_with("EOF\n"));
        assert_eq!(d.matches("\nARC\n").count(), s.n_slots * 2 + s.n_poles * 2);
        let g = svg_gabarit(&s);
        assert!(g.contains("width=") && g.matches("<path").count() >= s.n_poles);
        let l = femm_lua(&s, 360.0 / s.n_slots as f64, 0.25);
        assert!(l.contains("mi_analyze") && l.contains("Aimant"));
        let geo = gmsh_geo(&s);
        assert!(geo.contains("Physical Surface(\"aimants_N\")"));
    }

    #[test]
    fn import_angles() {
        let a = import_angles_csv("n;angle\n1;0\n2;25.7\n3;51.4\n4;77.1\n").unwrap();
        assert_eq!(a, vec![0.0, 25.7, 51.4, 77.1]);
        assert!(import_angles_csv("0\n10\n20\n").is_err());
    }

    #[test]
    fn import_courbe() {
        let c = import_curve_csv("theta;T\n0;0.001\n0.5;0.002\n").unwrap();
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn import_phi_sim() {
        let j = r#"{"spec": {"variant": "essai", "magnet_angles": [0, 25.7]}, "angles_deg": [0, 0.25], "torque_Nm": [0.001, -0.002]}"#;
        let r = import_phi_sim_json(j).unwrap();
        assert_eq!(r.variant, "essai");
        assert_eq!(r.curve.len(), 2);
        assert_eq!(r.magnet_angles.unwrap().len(), 2);
    }

    #[test]
    fn import_simumsap_latin1() {
        let txt = b"Type de la machine\r\n2\r\n\r\nLongueur de l'entrefer\r\n1.000000e-002\r\n\r\nNombre de paires de p\xf4les\r\n88\r\n\r\nNombre d'encoches statoriques\r\n165\r\n\r\nEpaisseur radiale des aimants\r\n2.500000e-002\r\n\r\nRayon du fer rotorique\r\n3.000000e+000\r\n\r\nInduction r\xe9manente des aimants (T)\r\n1.210000e+000\r\n";
        let r = import_simumsap(txt).unwrap();
        assert_eq!(r.spec.n_poles, 176);
        assert_eq!(r.spec.n_slots, 165);
        assert!((r.spec.airgap - 10.0).abs() < 1e-9);
        assert!((r.spec.br_magnet - 1.21).abs() < 1e-12);
        assert_eq!(r.spec.topology, crate::geometry::Topology::RadialInner);
    }
}
