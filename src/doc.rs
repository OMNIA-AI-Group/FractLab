//! Rédaction : fiche du projet, document structuré, sortie LaTeX, PDF
//! (compilation LaTeX si disponible, sinon générateur PDF intégré) et
//! sauvegarde MATLAB (.mat niveau 5).
//!
//! Le document est décrit une seule fois (`Doc`, suite de blocs) puis émis
//! en LaTeX, en PDF et affiché dans l'interface.

use crate::compute::{Results, VariantResult};
use crate::i18n::{Lang, T};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

/// Fiche descriptive du projet (tous les champs sont facultatifs).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectSheet {
    /// Titre du document.
    pub title: String,
    /// Sous-titre.
    pub subtitle: String,
    /// Auteurs.
    pub authors: String,
    /// Organisation.
    pub organisation: String,
    /// Dépôt INPI.
    pub inpi: String,
    /// Autres dépôts (brevets, enveloppes Soleau…).
    pub deposits: String,
    /// Licence.
    pub licence: String,
    /// Identifiant arXiv.
    pub arxiv: String,
    /// DOI.
    pub doi: String,
    /// Dépôt GitHub.
    pub github: String,
    /// Site web.
    pub website: String,
    /// Adresses électroniques.
    pub emails: String,
    /// LinkedIn.
    pub linkedin: String,
    /// Mots-clés.
    pub keywords: String,
    /// Résumé.
    pub abstract_: String,
    /// Remarques libres.
    pub notes: String,
    /// Textes de la fiche dans les autres langues (clé « EN », « ES », « DE ») ;
    /// les champs principaux servent pour le français et de repli.
    pub translations: std::collections::BTreeMap<String, SheetText>,
}

/// Textes de la fiche propres à une langue (champ vide = texte principal repris).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SheetText {
    /// Titre.
    pub title: String,
    /// Sous-titre.
    pub subtitle: String,
    /// Mots-clés.
    pub keywords: String,
    /// Résumé.
    pub abstract_: String,
    /// Remarques.
    pub notes: String,
}

impl ProjectSheet {
    /// Fiche dont les textes sont ceux de la langue `lang` (repli sur les
    /// champs principaux quand une traduction est vide).
    pub fn localized(&self, lang: Lang) -> ProjectSheet {
        let mut s = self.clone();
        if lang != Lang::Fr
            && let Some(t) = self.translations.get(lang.code()) {
                let pick = |dst: &mut String, v: &String| {
                    if !v.trim().is_empty() {
                        *dst = v.clone();
                    }
                };
                pick(&mut s.title, &t.title);
                pick(&mut s.subtitle, &t.subtitle);
                pick(&mut s.keywords, &t.keywords);
                pick(&mut s.abstract_, &t.abstract_);
                pick(&mut s.notes, &t.notes);
            }
        s
    }
}

/// Séparateur décimal de la langue : virgule sauf en anglais.
pub fn localize_number(s: &str, lang: Lang) -> String {
    if lang == Lang::En {
        return s.to_string();
    }
    let c: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &ch) in c.iter().enumerate() {
        let digit_before = i > 0 && c[i - 1].is_ascii_digit();
        let digit_after = i + 1 < c.len() && c[i + 1].is_ascii_digit();
        // « 3.0 » dans un identifiant (GPL-3.0, V2.1) précédé d'une lettre ou d'un tiret : inchangé
        let ident = {
            let mut j = i;
            while j > 0 && c[j - 1].is_ascii_digit() {
                j -= 1;
            }
            j > 0 && (c[j - 1].is_alphabetic() || c[j - 1] == '-')
        };
        if ch == '.' && digit_before && digit_after && !ident {
            out.push(',');
        } else {
            out.push(ch);
        }
    }
    out
}

impl Default for ProjectSheet {
    fn default() -> Self {
        ProjectSheet {
            title: "Étude FractLab".into(),
            subtitle: String::new(),
            authors: "Alain VALETTE-CLARY".into(),
            organisation: "ΩiA-Group".into(),
            inpi: "DSO2025023838".into(),
            deposits: String::new(),
            licence: "GPL-3.0-or-later".into(),
            arxiv: String::new(),
            doi: String::new(),
            github: String::new(),
            website: String::new(),
            emails: String::new(),
            linkedin: "https://www.linkedin.com/in/alain-valette-clary-524b28352/".into(),
            keywords: "moteur à aimants permanents, couple de détente, répartition des aimants, modèle analytique".into(),
            abstract_: String::new(),
            notes: String::new(),
            translations: [
                ("EN", "FractLab study", "permanent-magnet motor, cogging torque, magnet layout, analytical model"),
                ("ES", "Estudio FractLab", "motor de imanes permanentes, par de cogging, distribución de los imanes, modelo analítico"),
                ("DE", "FractLab-Studie", "Permanentmagnetmotor, Rastmoment, Magnetanordnung, analytisches Modell"),
            ]
            .into_iter()
            .map(|(k, t, kw)| (k.to_string(), SheetText { title: t.into(), keywords: kw.into(), ..Default::default() }))
            .collect(),
        }
    }
}

/// Bloc du document.
#[derive(Clone, Debug)]
pub enum Block {
    /// Titre de section.
    Heading(String),
    /// Paragraphe.
    Para(String),
    /// Tableau : en-têtes, lignes.
    Table(Vec<String>, Vec<Vec<String>>),
    /// Courbes : titre, légende x, légende y, séries (nom, points).
    Plot(String, String, String, Vec<(String, Vec<[f64; 2]>)>),
}

/// Document complet.
#[derive(Clone, Debug)]
pub struct Doc {
    /// Fiche.
    pub sheet: ProjectSheet,
    /// Corps.
    pub blocks: Vec<Block>,
    /// Langue des textes fixes du document.
    pub lang: Lang,
}

fn dec(v: &[[f64; 2]], max: usize) -> Vec<[f64; 2]> {
    let step = (v.len() / max).max(1);
    v.iter().step_by(step).cloned().collect()
}

/// Construit le document à partir des résultats.
pub fn build(sheet: &ProjectSheet, r: &Results, cmp: &[VariantResult], sim: Option<&crate::dynsim::SimResult>, lang: Lang) -> Doc {
    let loc = sheet.localized(lang);
    let sheet = &loc;
    let tt = |k: &'static str| -> String { T(lang, k).to_string() };
    let topo = |tp: crate::geometry::Topology| -> String {
        match tp {
            crate::geometry::Topology::RadialOuter => tt("Flux radial, rotor extérieur"),
            crate::geometry::Topology::RadialInner => tt("Flux radial, rotor intérieur"),
            crate::geometry::Topology::Axial => tt("Flux axial (multi-tranches)"),
        }
    };
    let s = &r.spec;
    let rep = &r.report;
    let mut b = Vec::new();
    let abs = if sheet.abstract_.trim().is_empty() {
        tt("Ce document présente l'analyse du moteur « {nom} » ({q} encoches, {p} pôles, {topo}) réalisée avec FractLab : champ magnétique par modèle analytique en sous-domaines, couple de détente, force électromotrice, forces, fonctionnement en charge et grandeurs dépendant de la vitesse.")
            .replace("{nom}", &s.name)
            .replace("{q}", &s.n_slots.to_string())
            .replace("{p}", &s.n_poles.to_string())
            .replace("{topo}", &topo(s.topology))
    } else {
        sheet.abstract_.clone()
    };
    b.push(Block::Heading(tt("Résumé")));
    b.push(Block::Para(abs));
    b.push(Block::Heading(tt("Machine étudiée")));
    let mut rows = vec![
        vec![tt("Topologie"), topo(s.topology)],
        vec![tt("Encoches / pôles"), format!("{} / {}", s.n_slots, s.n_poles)],
        vec![tt("Bobinage"), rep.winding.clone()],
        vec![tt("Entrefer"), format!("{} mm", s.airgap)],
        vec![tt("Aimants : épaisseur, arc, Br"), format!("{} mm, {:.3}, {} T", s.magnet_th, s.magnet_arc_ratio, s.br_magnet)],
        vec![tt("Ouverture d'encoche, becs"), format!("{} mm, {} mm", s.slot_open, s.tip_h)],
        vec![tt("Variante de répartition"), s.variant.clone()],
    ];
    if s.topology == crate::geometry::Topology::Axial {
        rows.push(vec![tt("Rayons actifs, tranches, entrefers"), format!("{}–{} mm, {}, {}", s.axial.r_in, s.axial.r_out, s.axial.slices, s.axial.gaps)]);
    } else {
        rows.push(vec![tt("Alésage, longueur active"), format!("{} mm, {} mm", s.stator_or, s.stack_len)]);
    }
    b.push(Block::Table(vec![tt("Grandeur"), tt("Valeur")], rows));
    b.push(Block::Heading(tt("Méthode")));
    b.push(Block::Para(tt("Le champ est obtenu par la solution formelle des équations de Maxwell en magnétostatique (séparation des variables), par sous-domaines : encoches, isthmes, entrefer et aimants ; les séries de Fourier couvrent 360° complets, ce qui admet toute répartition d'aimants. Les machines à flux axial sont traitées par la technique multi-tranches. Le fer est supposé de perméabilité infinie. Le modèle est validé contre un solveur volumes finis indépendant (écart typique 1 à 3 %).")));
    b.push(Block::Heading(tt("Résultats à vide")));
    b.push(Block::Table(
        vec![tt("Indicateur"), tt("Valeur")],
        vec![
            vec![tt("Couple de détente crête-à-crête"), format!("{:.4} mN·m", rep.cogging.cogging_pp_mnm)],
            vec![tt("Harmonique dominante"), format!("{:.4} mN·m, {} {:.0}", rep.cogging.dominant_mnm, tt("ordre"), rep.cogging.dominant_order_per_rev)],
            vec![tt("Part de la dominante"), format!("{:.1} %", rep.cogging.dominant_share_pct)],
            vec![tt("Force radiale maximale"), format!("{:.4} N", rep.force_max_n)],
            vec![tt("THD de la FEM"), format!("{:.2} %", rep.emf_thd_pct)],
            vec![tt("Écart minimal entre aimants"), format!("{:+.3}°", rep.min_magnet_gap_deg)],
            vec![tt("KV estimé"), format!("{:.0} {}/V", rep.kv_estime, tt("tr/min"))],
        ],
    ));
    let cog: Vec<[f64; 2]> = r.cog.angles_deg.iter().zip(&r.cog.torque_nm).map(|(a, t)| [*a, t * 1e3]).collect();
    b.push(Block::Plot(tt("Couple de détente"), "θr [°]".into(), "mN·m".into(), vec![(s.name.clone(), dec(&cog, 200))]));
    let emf: Vec<(String, Vec<[f64; 2]>)> = (0..3)
        .map(|k| (format!("{} {}", tt("phase"), ["A", "B", "C"][k]), dec(&r.emf.angles_deg.iter().zip(&r.emf.emf[k]).map(|(a, e)| [*a, *e]).collect::<Vec<_>>(), 200)))
        .collect();
    b.push(Block::Plot(format!("{} ({:.0} {})", tt("FEM à vide"), s.rpm, tt("tr/min")), "θr [°]".into(), "V".into(), emf));
    b.push(Block::Heading(tt("Fonctionnement en charge")));
    let l = &r.load;
    b.push(Block::Table(
        vec![tt("Indicateur"), tt("Valeur")],
        vec![
            vec![tt("Courant crête"), format!("{:.2} A", s.load.i_peak)],
            vec![tt("Couple moyen"), format!("{:.5} N·m", l.avg_torque)],
            vec![tt("Ondulation"), format!("{:.2} %", l.ripple_pct)],
            vec![tt("Inductance synchrone"), format!("{:.4} mH", l.ls * 1e3)],
            vec![tt("dont têtes de bobines"), format!("{:.4} mH", l.l_end * 1e3)],
            vec![tt("Induction dents / culasse stator / culasse rotor"), format!("{:.2} / {:.2} / {:.2} T", r.sat.at_load.b_tooth, r.sat.at_load.b_yoke, r.sat.at_load.b_rotor)],
            vec![tt("Effet de la saturation sur le flux"), format!("{:+.1} % ({} {:.3})", 100.0 * (r.sat.at_load.s - 1.0), "k_sat", r.sat.at_load.k_sat)],
            vec![tt("Résistance de phase"), format!("{:.4} Ω", l.r_phase)],
            vec![tt("Puissance mécanique"), format!("{:.2} W", l.p_mech)],
        ],
    ));
    let lt: Vec<[f64; 2]> = l.angles_deg.iter().zip(&l.torque).map(|(a, t)| [*a, *t]).collect();
    b.push(Block::Plot(tt("Couple en charge"), "θr [°]".into(), "N·m".into(), vec![("Maxwell".into(), dec(&lt, 200))]));
    if !cmp.is_empty() {
        b.push(Block::Heading(tt("Comparaison des variantes")));
        b.push(Block::Table(
            vec![tt("Variante"), tt("Cogging pp [mN·m]"), tt("Part dom. [%]"), tt("Force max [N]"), tt("THD [%]")],
            cmp.iter()
                .map(|v| vec![v.name.clone(), format!("{:.3}", v.report.cogging.cogging_pp_mnm), format!("{:.1}", v.report.cogging.dominant_share_pct), format!("{:.3}", v.report.force_max_n), format!("{:.2}", v.report.emf_thd_pct)])
                .collect(),
        ));
    }
    if let Some(o) = sim
        && o.t.len() > 2 {
            b.push(Block::Heading(tt("Simulation dynamique")));
            let w: Vec<[f64; 2]> = o.t.iter().zip(&o.rpm).map(|(a, v)| [*a, *v]).collect();
            b.push(Block::Plot(tt("Vitesse"), "t [s]".into(), tt("tr/min"), vec![(tt("vitesse"), dec(&w, 300))]));
        }
    if !sheet.notes.trim().is_empty() {
        b.push(Block::Heading(tt("Remarques")));
        b.push(Block::Para(sheet.notes.clone()));
    }
    b.push(Block::Heading(tt("Références")));
    b.push(Block::Para(tt("H. Tiegna, A. Bellara, Y. Amara, G. Barakat, « SIMUMSAP — An Analytical Design Tool for Permanent Magnet Electrical Machines », GREAH, Université du Havre. — FractLab, logiciel libre (GPL-3.0), Alain VALETTE-CLARY, ΩiA-Group.")));
    for blk in &mut b {
        if let Block::Table(_, rows) = blk {
            for row in rows.iter_mut() {
                for (k, cell) in row.iter_mut().enumerate() {
                    if k > 0 {
                        *cell = localize_number(cell, lang);
                    }
                }
            }
        }
    }
    Doc { sheet: sheet.clone(), blocks: b, lang }
}

fn tex(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\textbackslash{}"),
            '&' | '%' | '$' | '#' | '_' | '{' | '}' => {
                o.push('\\');
                o.push(c);
            }
            '~' => o.push_str("\\textasciitilde{}"),
            '^' => o.push_str("\\textasciicircum{}"),
            'Φ' => o.push_str("$\\Phi$"),
            'Ω' => o.push_str("$\\Omega$"),
            'θ' => o.push_str("$\\theta$"),
            'μ' | 'µ' => o.push_str("$\\mu$"),
            'Σ' => o.push_str("$\\Sigma$"),
            'λ' => o.push_str("$\\lambda$"),
            '·' => o.push_str("$\\cdot$"),
            '≤' => o.push_str("$\\leq$"),
            '≥' => o.push_str("$\\geq$"),
            '±' => o.push_str("$\\pm$"),
            '–' => o.push_str("--"),
            '—' => o.push_str("---"),
            '°' => o.push_str("$^\\circ$"),
            _ => o.push(c),
        }
    }
    o
}

/// Source LaTeX complète du document.
pub fn to_latex(d: &Doc) -> String {
    let sh = &d.sheet;
    let tt = |k: &'static str| T(d.lang, k);
    let babel = match d.lang {
        Lang::Fr => "french",
        Lang::En => "english",
        Lang::Es => "spanish",
        Lang::De => "ngerman",
    };
    let mut s = String::new();
    let _ = writeln!(s, "% FractLab v{} — {}", env!("CARGO_PKG_VERSION"), tex(&sh.title));
    s.push_str("\\documentclass[11pt,a4paper]{article}\n\\usepackage[utf8]{inputenc}\n\\usepackage[T1]{fontenc}\n\\usepackage[babel]{babel}\n\\usepackage[margin=2.2cm]{geometry}\n\\usepackage{booktabs}\n\\usepackage{pgfplots}\n\\pgfplotsset{compat=1.17}\n\\usepackage[hidelinks]{hyperref}\n\\usepackage{graphicx}\n\\usepackage{fancyhdr}\n\\pagestyle{fancy}\n\\fancyhf{}\n");
    let _ = writeln!(s, "\\fancyhead[L]{{\\IfFileExists{{fractlab_logo.png}}{{\\includegraphics[height=0.7cm]{{fractlab_logo.png}}}}{{}}\\ \\small\\textbf{{FractLab}} v{}}}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(s, "\\fancyhead[R]{{\\small {}}}", tex(&sh.title));
    let _ = writeln!(s, "\\fancyfoot[L]{{\\scriptsize FractLab v{} --- Alain VALETTE-CLARY $\\cdot$ $\\Omega$iA-Group $\\cdot$ INPI DSO2025023838}}", env!("CARGO_PKG_VERSION"));
    s.push_str("\\fancyfoot[R]{\\small\\thepage}\n\\renewcommand{\\headrulewidth}{0.4pt}\n\n");
    s = s.replace("[babel]{babel}", &format!("[{babel}]{{babel}}"));
    let _ = writeln!(s, "\\title{{{}{}}}", tex(&sh.title), if sh.subtitle.is_empty() { String::new() } else { format!("\\\\\\large {}", tex(&sh.subtitle)) });
    let _ = writeln!(s, "\\author{{{}{}}}", tex(&sh.authors), if sh.organisation.is_empty() { String::new() } else { format!("\\\\{}", tex(&sh.organisation)) });
    s.push_str("\\date{\\today}\n\\begin{document}\n\\maketitle\n\n");
    // identifiants
    let ids: Vec<(&str, &String)> = vec![
        ("INPI", &sh.inpi),
        (tt("Autres dépôts"), &sh.deposits),
        (tt("Licence"), &sh.licence),
        ("arXiv", &sh.arxiv),
        ("DOI", &sh.doi),
        ("GitHub", &sh.github),
        (tt("Site web"), &sh.website),
        (tt("Adresses mail"), &sh.emails),
        ("LinkedIn", &sh.linkedin),
        (tt("Mots-clés"), &sh.keywords),
    ];
    let present: Vec<_> = ids.into_iter().filter(|(_, v)| !v.trim().is_empty()).collect();
    if !present.is_empty() {
        s.push_str("\\begin{center}\\small\n\\begin{tabular}{ll}\n\\toprule\n");
        for (k, v) in present {
            let val = if v.starts_with("http") { format!("\\url{{{}}}", v) } else { tex(v) };
            let _ = writeln!(s, "{} & {} \\\\", tex(k), val);
        }
        s.push_str("\\bottomrule\n\\end{tabular}\n\\end{center}\n\n");
    }
    for blk in &d.blocks {
        match blk {
            Block::Heading(h) => {
                let _ = writeln!(s, "\\section{{{}}}", tex(h));
            }
            Block::Para(p) => {
                let _ = writeln!(s, "{}\n", tex(p));
            }
            Block::Table(h, rows) => {
                let _ = writeln!(s, "\\begin{{center}}\\small\n\\begin{{tabular}}{{{}}}\n\\toprule", "l".repeat(h.len()));
                let _ = writeln!(s, "{} \\\\\n\\midrule", h.iter().map(|x| tex(x)).collect::<Vec<_>>().join(" & "));
                for r in rows {
                    let _ = writeln!(s, "{} \\\\", r.iter().map(|x| tex(x)).collect::<Vec<_>>().join(" & "));
                }
                s.push_str("\\bottomrule\n\\end{tabular}\n\\end{center}\n\n");
            }
            Block::Plot(t, xl, yl, series) => {
                let _ = writeln!(s, "\\begin{{center}}\n\\begin{{tikzpicture}}\n\\begin{{axis}}[width=0.9\\linewidth,height=5.5cm,title={{{}}},xlabel={{{}}},ylabel={{{}}},grid=major,legend pos=outer north east{}]", tex(t), tex(xl), tex(yl), if d.lang == Lang::En { "" } else { ",/pgf/number format/use comma" });
                for (n, pts) in series {
                    s.push_str("\\addplot[thick,mark=none] coordinates {");
                    for p in pts {
                        let _ = write!(s, "({:.6e},{:.6e})", p[0], p[1]);
                    }
                    let _ = writeln!(s, "}};\n\\addlegendentry{{{}}}", tex(n));
                }
                s.push_str("\\end{axis}\n\\end{tikzpicture}\n\\end{center}\n\n");
            }
        }
    }
    s.push_str("\\end{document}\n");
    s
}

// --------------------------------------------------------------------
// Générateur PDF intégré (aucun outil externe) : texte Helvetica en
// codage WinAnsi, tableaux et courbes vectorielles.
// --------------------------------------------------------------------

/// Lettre grecque → code de la police Symbol.
fn symbol_code(c: char) -> Option<u8> {
    Some(match c {
        'Φ' => b'F',
        'Ω' => b'W',
        'θ' => b'q',
        'μ' => b'm',
        'λ' => b'l',
        'Σ' => b'S',
        'π' => b'p',
        'Δ' => b'D',
        'ω' => b'w',
        '≤' => 0xA3,
        '≥' => 0xB3,
        _ => return None,
    })
}

fn winansi(s: &str) -> Vec<u8> {
    let mut o = Vec::new();
    for c in s.chars() {
        let rep: &str = match c {
            'Φ' => "Phi",
            'Ω' => "Ohm",
            'θ' => "theta",
            'Σ' => "Somme",
            'λ' => "lambda",
            'μ' => "\u{b5}",
            '≤' => "<=",
            '≥' => ">=",
            '–' | '—' => "-",
            '“' | '”' => "\"",
            '…' => "\u{85}",
            '’' => "'",
            _ => "",
        };
        if !rep.is_empty() {
            for ch in rep.chars() {
                o.push(ch as u32 as u8);
            }
            continue;
        }
        let u = c as u32;
        match c {
            '(' | ')' | '\\' => {
                o.push(b'\\');
                o.push(u as u8);
            }
            _ if u < 256 => o.push(u as u8),
            _ => o.push(b'?'),
        }
    }
    o
}

struct PdfW {
    pages: Vec<Vec<u8>>,
    cur: Vec<u8>,
    y: f64,
}

const PW: f64 = 595.0;
const PH: f64 = 842.0;
const ML: f64 = 56.0;

impl PdfW {
    fn new() -> PdfW {
        PdfW { pages: Vec::new(), cur: Vec::new(), y: PH - 60.0 }
    }
    fn need(&mut self, h: f64) {
        if self.y - h < 50.0 {
            self.pages.push(std::mem::take(&mut self.cur));
            self.y = PH - 60.0;
        }
    }
    /// En-tête (logo + nom + version) et pied de page de chaque page.
    fn decorate(&self, page: usize, total: usize, title: &str) -> Vec<u8> {
        let mut d = Vec::new();
        d.extend_from_slice(format!("q 26 0 0 26 {ML} {} cm /Im1 Do Q\n", PH - 40.0).as_bytes());
        let mut tmp = PdfW { pages: vec![], cur: vec![], y: 0.0 };
        tmp.text(ML + 32.0, PH - 30.0, 10.0, true, &format!("FractLab v{}", env!("CARGO_PKG_VERSION")));
        let t: String = title.chars().take(70).collect();
        tmp.text(PW - ML - t.chars().count() as f64 * 4.2, PH - 30.0, 8.0, false, &t);
        tmp.line(ML, PH - 44.0, PW - ML, PH - 44.0, 0.4, (0.5, 0.5, 0.5));
        tmp.text(ML, 26.0, 7.0, false, &format!("FractLab v{} — Alain VALETTE-CLARY · ΩiA-Group · INPI DSO2025023838", env!("CARGO_PKG_VERSION")));
        tmp.text(PW - ML - 30.0, 26.0, 8.0, false, &format!("{} / {}", page + 1, total));
        d.extend(tmp.cur);
        d
    }
    fn text(&mut self, x: f64, y: f64, size: f64, bold: bool, s: &str) {
        // segments alternés Helvetica / Symbol (lettres grecques)
        let font = if bold { "F2" } else { "F1" };
        self.cur.extend_from_slice(format!("BT {:.2} {:.2} Td ", x, y).as_bytes());
        let mut run = String::new();
        let flush = |cur: &mut Vec<u8>, run: &mut String| {
            if !run.is_empty() {
                cur.extend_from_slice(format!("/{font} {size} Tf (").as_bytes());
                cur.extend(winansi(run));
                cur.extend_from_slice(b") Tj ");
                run.clear();
            }
        };
        for c in s.chars() {
            if let Some(code) = symbol_code(c) {
                flush(&mut self.cur, &mut run);
                self.cur.extend_from_slice(format!("/F3 {size} Tf (").as_bytes());
                if code == b'(' || code == b')' || code == b'\\' {
                    self.cur.push(b'\\');
                }
                self.cur.push(code);
                self.cur.extend_from_slice(b") Tj ");
            } else {
                run.push(c);
            }
        }
        flush(&mut self.cur, &mut run);
        self.cur.extend_from_slice(b"ET\n");
    }
    fn para(&mut self, s: &str, size: f64, bold: bool) {
        let maxc = ((PW - 2.0 * ML) / (size * 0.5)) as usize;
        let mut line = String::new();
        let mut lines = Vec::new();
        for w in s.split_whitespace() {
            if line.chars().count() + w.chars().count() + 1 > maxc && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(w);
        }
        if !line.is_empty() {
            lines.push(line);
        }
        for l in lines {
            self.need(size * 1.4);
            self.y -= size * 1.4;
            let y = self.y;
            self.text(ML, y, size, bold, &l);
        }
    }
    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, w: f64, rgb: (f64, f64, f64)) {
        self.cur.extend_from_slice(format!("{:.3} {:.3} {:.3} RG {:.2} w {:.2} {:.2} m {:.2} {:.2} l S\n", rgb.0, rgb.1, rgb.2, w, x1, y1, x2, y2).as_bytes());
    }
    fn finish(mut self, title: &str) -> Vec<u8> {
        if !self.cur.is_empty() {
            self.pages.push(std::mem::take(&mut self.cur));
        }
        let total = self.pages.len();
        for i in 0..total {
            let mut d = self.decorate(i, total, title);
            d.extend(std::mem::take(&mut self.pages[i]));
            self.pages[i] = d;
        }
        let mut objs: Vec<Vec<u8>> = Vec::new();
        let n = self.pages.len();
        // 1 catalogue, 2 pages, 3 police, 4 police grasse, puis (contenu, page) par page
        objs.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
        // objets : 1 catalogue, 2 pages, 3-4 Helvetica, 5 Symbol, 6 logo, puis (contenu, page)
        let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 8 + 2 * i)).collect();
        objs.push(format!("<< /Type /Pages /Count {} /Kids [{}] >>", n, kids.join(" ")).into_bytes());
        objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec());
        objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec());
        objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Symbol >>".to_vec());
        objs.push(logo_xobject());
        for (i, c) in self.pages.iter().enumerate() {
            let mut st = format!("<< /Length {} >>\nstream\n", c.len()).into_bytes();
            st.extend_from_slice(c);
            st.extend_from_slice(b"\nendstream");
            objs.push(st);
            objs.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PW} {PH}] /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R >> /XObject << /Im1 6 0 R >> >> /Contents {} 0 R >>", 7 + 2 * i).into_bytes());
        }
        let mut out = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
        let mut offs = Vec::new();
        for (i, o) in objs.iter().enumerate() {
            offs.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            out.extend_from_slice(o);
            out.extend_from_slice(b"\nendobj\n");
        }
        let xref = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
        for o in offs {
            out.extend_from_slice(format!("{:010} 00000 n \n", o).as_bytes());
        }
        out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", objs.len() + 1, xref).as_bytes());
        out
    }
}

/// PDF produit par le générateur intégré.
pub fn to_pdf_builtin(d: &Doc) -> Vec<u8> {
    let sh = &d.sheet;
    let mut w = PdfW::new();
    w.para(&sh.title, 18.0, true);
    if !sh.subtitle.is_empty() {
        w.para(&sh.subtitle, 12.0, false);
    }
    w.para(&format!("{} — {}", sh.authors, sh.organisation), 11.0, false);
    let tt = |k: &'static str| T(d.lang, k);
    for (k, v) in [("INPI", &sh.inpi), (tt("Licence"), &sh.licence), (tt("Autres dépôts"), &sh.deposits), ("arXiv", &sh.arxiv), ("DOI", &sh.doi), ("GitHub", &sh.github), (tt("Site web"), &sh.website), (tt("Adresses mail"), &sh.emails), ("LinkedIn", &sh.linkedin)] {
        if !v.trim().is_empty() {
            w.para(&format!("{k} : {v}"), 9.0, false);
        }
    }
    w.y -= 8.0;
    let palette = [(0.0, 0.47, 0.67), (0.9, 0.45, 0.05), (0.2, 0.6, 0.25), (0.75, 0.2, 0.5)];
    for blk in &d.blocks {
        match blk {
            Block::Heading(h) => {
                w.need(30.0);
                w.y -= 10.0;
                w.para(h, 13.0, true);
            }
            Block::Para(p) => w.para(p, 10.0, false),
            Block::Table(h, rows) => {
                let cols = h.len().max(1);
                let cw = (PW - 2.0 * ML) / cols as f64;
                let row = |w: &mut PdfW, cells: &Vec<String>, bold: bool| {
                    w.need(14.0);
                    w.y -= 14.0;
                    for (i, c) in cells.iter().enumerate() {
                        let txt: String = c.chars().take((cw / 5.0) as usize).collect();
                        let y = w.y;
                        w.text(ML + i as f64 * cw, y, 9.0, bold, &txt);
                    }
                };
                row(&mut w, h, true);
                let y = w.y - 3.0;
                w.line(ML, y, PW - ML, y, 0.6, (0.0, 0.0, 0.0));
                for r in rows {
                    row(&mut w, r, false);
                }
                w.y -= 6.0;
            }
            Block::Plot(t, xl, yl, series) => {
                let (pw, ph) = (PW - 2.0 * ML - 30.0, 150.0);
                w.need(ph + 50.0);
                w.y -= 16.0;
                let y = w.y;
                w.text(ML, y, 10.0, true, t);
                let (x0, y0) = (ML + 30.0, w.y - ph - 8.0);
                let all: Vec<&[f64; 2]> = series.iter().flat_map(|(_, p)| p.iter()).collect();
                let (xmn, xmx) = all.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p[0]), b.max(p[0])));
                let (ymn, ymx) = all.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p[1]), b.max(p[1])));
                let sx = |x: f64| x0 + (x - xmn) / (xmx - xmn).max(1e-30) * pw;
                let sy = |v: f64| y0 + (v - ymn) / (ymx - ymn).max(1e-30) * ph;
                for (a, b, c, d2) in [(x0, y0, x0 + pw, y0), (x0, y0, x0, y0 + ph), (x0 + pw, y0, x0 + pw, y0 + ph), (x0, y0 + ph, x0 + pw, y0 + ph)] {
                    w.line(a, b, c, d2, 0.5, (0.5, 0.5, 0.5));
                }
                for (k, (_, pts)) in series.iter().enumerate() {
                    for s2 in pts.windows(2) {
                        w.line(sx(s2[0][0]), sy(s2[0][1]), sx(s2[1][0]), sy(s2[1][1]), 1.0, palette[k % 4]);
                    }
                }
                w.text(x0, y0 - 12.0, 8.0, false, &localize_number(&format!("{xl} : {xmn:.3} … {xmx:.3}"), d.lang));
                w.text(x0 + pw * 0.55, y0 - 12.0, 8.0, false, &localize_number(&format!("{yl} : {ymn:.4} … {ymx:.4}"), d.lang));
                let legend: Vec<String> = series.iter().map(|(n, _)| n.clone()).collect();
                let tw = t.chars().count() as f64 * 5.6;
                w.text(ML + tw + 16.0, y, 7.0, false, &legend.join("  ·  "));
                w.y = y0 - 22.0;
            }
        }
    }
    w.y -= 10.0;
    w.para("Document généré par FractLab — logiciel libre (GPL-3.0) — Alain VALETTE-CLARY · ΩiA-Group · INPI DSO2025023838", 8.0, false);
    w.finish(&sh.title)
}

/// Logo FractLab (PNG intégré).
pub const LOGO_PNG: &[u8] = include_bytes!("../assets/icon.png");

/// Logo en objet image PDF (RVB 8 bits compressé).
fn logo_xobject() -> Vec<u8> {
    use std::io::Write;
    let img = image::load_from_memory(LOGO_PNG).map(|i| i.thumbnail(128, 128).to_rgb8()).ok();
    let Some(img) = img else { return b"<< >>".to_vec() };
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    let _ = z.write_all(img.as_raw());
    let data = z.finish().unwrap_or_default();
    let mut o = format!("<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n", img.width(), img.height(), data.len()).into_bytes();
    o.extend(data);
    o.extend_from_slice(b"\nendstream");
    o
}

/// Compile la source LaTeX avec un moteur installé (tectonic, pdflatex,
/// xelatex, lualatex) ; renvoie le PDF et le nom du moteur, ou None.
pub fn compile_latex(src: &str) -> Option<(Vec<u8>, String)> {
    // navigateur : pas de LaTeX installé, le générateur PDF intégré prend le relais
    if crate::platform::WEB {
        return None;
    }
    let dir = std::env::temp_dir().join(format!("fractlab_tex_{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    std::fs::write(dir.join("doc.tex"), src).ok()?;
    let _ = std::fs::write(dir.join("fractlab_logo.png"), LOGO_PNG);
    for (cmd, args) in [("tectonic", vec!["doc.tex"]), ("pdflatex", vec!["-interaction=nonstopmode", "doc.tex"]), ("xelatex", vec!["-interaction=nonstopmode", "doc.tex"]), ("lualatex", vec!["-interaction=nonstopmode", "doc.tex"])] {
        let ok = std::process::Command::new(cmd).args(&args).current_dir(&dir).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map(|s| s.success()).unwrap_or(false);
        if ok
            && let Ok(b) = std::fs::read(dir.join("doc.pdf")) {
                return Some((b, cmd.to_string()));
            }
    }
    None
}

// --------------------------------------------------------------------
// Écriture MATLAB niveau 5 (.mat) : tableaux réels et chaînes.
// --------------------------------------------------------------------

/// Variable à écrire dans un fichier .mat.
pub enum MatVar {
    /// Tableau réel (lignes, colonnes, données en ordre colonne).
    Num(usize, usize, Vec<f64>),
    /// Chaîne.
    Text(String),
}

fn el(ty: u32, data: &[u8]) -> Vec<u8> {
    let mut v = ty.to_le_bytes().to_vec();
    v.extend_from_slice(&(data.len() as u32).to_le_bytes());
    v.extend_from_slice(data);
    while !v.len().is_multiple_of(8) {
        v.push(0);
    }
    v
}

/// Fichier MATLAB niveau 5 non compressé (lisible par MATLAB, Octave, scipy).
pub fn write_mat(vars: &[(&str, MatVar)]) -> Vec<u8> {
    let mut out = vec![b' '; 128];
    let head = b"MATLAB 5.0 MAT-file, Platform: FractLab, Created by FractLab";
    out[..head.len()].copy_from_slice(head);
    out[124] = 0;
    out[125] = 1;
    out[126] = b'I';
    out[127] = b'M';
    for (name, v) in vars {
        let mut m = Vec::new();
        let (class, rows, cols): (u8, usize, usize) = match v {
            MatVar::Num(r, c, _) => (6, *r, *c),
            MatVar::Text(t) => (4, 1, t.encode_utf16().count()),
        };
        m.extend(el(6, &[class, 0, 0, 0, 0, 0, 0, 0]));
        let dims: Vec<u8> = [rows as i32, cols as i32].iter().flat_map(|d| d.to_le_bytes()).collect();
        m.extend(el(5, &dims));
        m.extend(el(1, name.as_bytes()));
        match v {
            MatVar::Num(_, _, d) => m.extend(el(9, &d.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>())),
            MatVar::Text(t) => m.extend(el(17, &t.encode_utf16().flat_map(|c| c.to_le_bytes()).collect::<Vec<u8>>())),
        }
        out.extend(el(14, &m));
    }
    out
}

/// Variables MATLAB décrivant le projet et ses résultats.
pub fn results_mat(r: &Results, sim: Option<&crate::dynsim::SimResult>) -> Vec<u8> {
    let col = |v: &Vec<f64>| MatVar::Num(v.len(), 1, v.clone());
    let s = &r.spec;
    let mut vars: Vec<(&str, MatVar)> = vec![
        ("nom_projet", MatVar::Text(s.name.clone())),
        ("projet_json", MatVar::Text(serde_json::to_string(s).unwrap_or_default())),
        ("nb_encoches", MatVar::Num(1, 1, vec![s.n_slots as f64])),
        ("nb_poles", MatVar::Num(1, 1, vec![s.n_poles as f64])),
        ("angles_aimants_deg", col(&s.magnet_angles_deg())),
        ("cogging_theta_deg", col(&r.cog.angles_deg)),
        ("cogging_couple_Nm", col(&r.cog.torque_nm)),
        ("fem_theta_deg", col(&r.emf.angles_deg)),
        ("fem_A_V", col(&r.emf.emf[0])),
        ("fem_B_V", col(&r.emf.emf[1])),
        ("fem_C_V", col(&r.emf.emf[2])),
        ("flux_A_Wb", col(&r.emf.flux[0])),
        ("force_theta_deg", col(&r.forces.angles_deg)),
        ("force_x_N", col(&r.forces.fx)),
        ("force_y_N", col(&r.forces.fy)),
        ("charge_theta_deg", col(&r.load.angles_deg)),
        ("charge_couple_Nm", col(&r.load.torque)),
        ("inductances_H", MatVar::Num(3, 3, (0..9).map(|k| r.load.inductances[k % 3][k / 3]).collect())),
        ("resistance_phase_Ohm", MatVar::Num(1, 1, vec![r.load.r_phase])),
    ];
    if let Some(o) = sim {
        vars.push(("sim_t_s", col(&o.t)));
        vars.push(("sim_vitesse_rpm", col(&o.rpm)));
        vars.push(("sim_couple_Nm", col(&o.te)));
        vars.push(("sim_iA_A", col(&o.i[0])));
    }
    write_mat(&vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat_relisible_par_notre_lecteur() {
        let b = write_mat(&[("x", MatVar::Num(3, 1, vec![1.0, 2.5, -3.0])), ("nom", MatVar::Text("Φ-essai".into()))]);
        let v = crate::matfile::read(&b).unwrap();
        assert_eq!(v[0].0, "x");
        assert_eq!(v[0].1.nums(), &[1.0, 2.5, -3.0]);
        assert_eq!(v[1].1.text(), "Φ-essai");
    }

    #[test]
    fn pdf_bien_forme() {
        assert_eq!(localize_number("0.85 mm, GPL-3.0, V2.1, 12.5", Lang::Fr), "0,85 mm, GPL-3.0, V2.1, 12,5");
        assert_eq!(localize_number("0.85", Lang::En), "0.85");
        let mut sh = ProjectSheet::default();
        sh.abstract_ = "résumé".into();
        assert_eq!(sh.localized(Lang::En).title, "FractLab study");
        assert_eq!(sh.localized(Lang::En).abstract_, "résumé");
        assert_eq!(sh.localized(Lang::Fr).title, ProjectSheet::default().title);
        let d = Doc { lang: Lang::Fr, sheet: ProjectSheet::default(), blocks: vec![Block::Heading("Test é à".into()), Block::Para("Texte (avec parenthèses) Φ".into()), Block::Plot("p".into(), "x".into(), "y".into(), vec![("s".into(), vec![[0.0, 0.0], [1.0, 1.0]])])] };
        let p = to_pdf_builtin(&d);
        assert!(p.starts_with(b"%PDF-1.4"));
        assert!(p.ends_with(b"%%EOF\n"));
        let l = to_latex(&d);
        assert!(l.contains("\\begin{document}") && l.contains("$\\Phi$"));
    }
}
