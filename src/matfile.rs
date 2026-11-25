//! Lecteur de fichiers MATLAB niveau 5 (`.mat`, `.fig`), sans MATLAB.
//!
//! Format documenté par MathWorks (« MAT-File Format », niveau 5) : en-tête de
//! 128 octets puis éléments étiquetés ; les éléments `miCOMPRESSED` sont des
//! flux zlib contenant eux-mêmes des éléments. Les classes prises en charge
//! couvrent les données et les figures : tableaux numériques (réels), chaînes,
//! structures, objets, cellules ; les classes non gérées sont ignorées
//! proprement.
//!
//! Une figure `.fig` est un fichier MAT contenant la structure `hgS_070000`
//! (arbre type / propriétés / enfants) : [`figure_curves`] en extrait les
//! courbes (lignes et séries) avec titres et légendes d'axes.

use flate2::read::ZlibDecoder;
use std::io::Read;

/// Valeur MATLAB décodée.
#[derive(Clone, Debug)]
pub enum Value {
    /// Tableau numérique réel (ordre colonne), dimensions.
    Num {
        /// Dimensions.
        dims: Vec<usize>,
        /// Valeurs (partie réelle).
        data: Vec<f64>,
    },
    /// Chaîne de caractères.
    Char(String),
    /// Structure ou objet : noms de champs et valeurs par élément.
    Struct {
        /// Dimensions.
        dims: Vec<usize>,
        /// Noms des champs.
        fields: Vec<String>,
        /// Valeurs : `elems[i][f]` = champ f de l'élément i.
        elems: Vec<Vec<Value>>,
        /// Nom de classe pour un objet (vide pour une structure).
        class: String,
    },
    /// Tableau de cellules.
    Cell {
        /// Dimensions.
        dims: Vec<usize>,
        /// Éléments.
        elems: Vec<Value>,
    },
    /// Valeur non prise en charge (classe ignorée).
    Other,
}

impl Value {
    /// Champ `name` du premier élément d'une structure.
    pub fn field(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Struct { fields, elems, .. } => {
                let i = fields.iter().position(|f| f == name)?;
                elems.first().map(|e| &e[i])
            }
            _ => None,
        }
    }
    /// Données numériques (vide sinon).
    pub fn nums(&self) -> &[f64] {
        match self {
            Value::Num { data, .. } => data,
            _ => &[],
        }
    }
    /// Texte (chaîne, ou concaténation d'une cellule de chaînes).
    pub fn text(&self) -> String {
        match self {
            Value::Char(s) => s.clone(),
            Value::Cell { elems, .. } => elems.iter().map(|e| e.text()).collect::<Vec<_>>().join(" "),
            _ => String::new(),
        }
    }
}

const MI_INT8: u32 = 1;
const MI_UINT8: u32 = 2;
const MI_INT16: u32 = 3;
const MI_UINT16: u32 = 4;
const MI_INT32: u32 = 5;
const MI_UINT32: u32 = 6;
const MI_SINGLE: u32 = 7;
const MI_DOUBLE: u32 = 9;
const MI_INT64: u32 = 12;
const MI_UINT64: u32 = 13;
const MI_MATRIX: u32 = 14;
const MI_COMPRESSED: u32 = 15;
const MI_UTF8: u32 = 16;
const MI_UTF16: u32 = 17;
const MI_UTF32: u32 = 18;

struct Rd<'a> {
    b: &'a [u8],
    p: usize,
}

impl<'a> Rd<'a> {
    fn u32(&mut self) -> Result<u32, String> {
        let s = self.b.get(self.p..self.p + 4).ok_or("fin de fichier inattendue")?;
        self.p += 4;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    /// Lit une étiquette d'élément (forme normale ou compacte) : (type, octets, données).
    fn tag(&mut self) -> Result<(u32, &'a [u8]), String> {
        let first = self.u32()?;
        if first >> 16 != 0 {
            // élément compact : 2 octets de taille, 2 de type, 4 de données
            let ty = first & 0xffff;
            let n = (first >> 16) as usize;
            let d = self.b.get(self.p..self.p + n).ok_or("élément compact tronqué")?;
            self.p += 4;
            return Ok((ty, d));
        }
        let n = self.u32()? as usize;
        let d = self.b.get(self.p..self.p + n).ok_or("élément tronqué")?;
        self.p += n;
        if !self.p.is_multiple_of(8) && first != MI_COMPRESSED {
            self.p += 8 - self.p % 8;
        }
        Ok((first, d))
    }
}

fn nums_of(ty: u32, d: &[u8]) -> Vec<f64> {
    macro_rules! conv {
        ($t:ty, $n:expr) => {
            d.chunks_exact($n).map(|c| <$t>::from_le_bytes(c.try_into().unwrap()) as f64).collect()
        };
    }
    match ty {
        MI_INT8 => d.iter().map(|&x| x as i8 as f64).collect(),
        MI_UINT8 | MI_UTF8 => d.iter().map(|&x| x as f64).collect(),
        MI_INT16 => conv!(i16, 2),
        MI_UINT16 => conv!(u16, 2),
        MI_INT32 => conv!(i32, 4),
        MI_UINT32 => conv!(u32, 4),
        MI_SINGLE => conv!(f32, 4),
        MI_DOUBLE => conv!(f64, 8),
        MI_INT64 => conv!(i64, 8),
        MI_UINT64 => conv!(u64, 8),
        _ => Vec::new(),
    }
}

/// Décode le contenu d'un élément miMATRIX ; renvoie (nom, valeur).
fn matrix(d: &[u8], depth: usize) -> Result<(String, Value), String> {
    if d.is_empty() {
        return Ok((String::new(), Value::Other));
    }
    if depth > 64 {
        return Err("imbrication trop profonde".into());
    }
    let mut r = Rd { b: d, p: 0 };
    let (_, flags) = r.tag()?;
    let class = flags.first().copied().unwrap_or(0) as u32;
    let (_, dimb) = r.tag()?;
    let dims: Vec<usize> = nums_of(MI_INT32, dimb).iter().map(|&x| x as usize).collect();
    let (_, nameb) = r.tag()?;
    let name = String::from_utf8_lossy(nameb).to_string();
    let count: usize = dims.iter().product();
    let v = match class {
        // 1 cellule, 2 structure, 3 objet, 4 char, 6..15 numériques
        1 => {
            let mut elems = Vec::with_capacity(count);
            for _ in 0..count {
                let (_, md) = r.tag()?;
                elems.push(matrix(md, depth + 1)?.1);
            }
            Value::Cell { dims, elems }
        }
        2 | 3 => {
            let mut cname = String::new();
            if class == 3 {
                let (_, cn) = r.tag()?;
                cname = String::from_utf8_lossy(cn).to_string();
            }
            let (_, fl) = r.tag()?;
            let flen = nums_of(MI_INT32, fl).first().copied().unwrap_or(1.0).max(1.0) as usize;
            let (_, fnames) = r.tag()?;
            let fields: Vec<String> = fnames
                .chunks(flen)
                .map(|c| String::from_utf8_lossy(c).trim_end_matches('\0').to_string())
                .collect();
            let mut elems = Vec::with_capacity(count);
            for _ in 0..count {
                let mut e = Vec::with_capacity(fields.len());
                for _ in 0..fields.len() {
                    let (_, md) = r.tag()?;
                    e.push(matrix(md, depth + 1)?.1);
                }
                elems.push(e);
            }
            Value::Struct { dims, fields, elems, class: cname }
        }
        4 => {
            let (ty, cd) = r.tag()?;
            let s: String = match ty {
                MI_UTF8 | MI_UINT8 | MI_INT8 => String::from_utf8_lossy(cd).to_string(),
                MI_UTF16 => String::from_utf16_lossy(&cd.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>()),
                MI_UTF32 => cd.as_chunks::<4>().0.iter().filter_map(|c| char::from_u32(u32::from_le_bytes([c[0], c[1], c[2], c[3]]))).collect(),
                _ => nums_of(ty, cd).iter().filter_map(|&c| char::from_u32(c as u32)).collect(),
            };
            Value::Char(s)
        }
        6..=15 => {
            let (ty, rd) = r.tag()?;
            Value::Num { dims, data: nums_of(ty, rd) }
        }
        _ => Value::Other,
    };
    Ok((name, v))
}

/// Lit toutes les variables d'un fichier MAT niveau 5.
pub fn read(bytes: &[u8]) -> Result<Vec<(String, Value)>, String> {
    if bytes.len() < 128 || !bytes.starts_with(b"MATLAB 5.0 MAT-file") {
        return Err("ce n'est pas un fichier MATLAB niveau 5 (.mat / .fig)".into());
    }
    if &bytes[126..128] != b"IM" {
        return Err("fichier MATLAB gros-boutiste non pris en charge".into());
    }
    let mut out = Vec::new();
    let mut r = Rd { b: &bytes[128..], p: 0 };
    while r.p + 8 <= r.b.len() {
        let (ty, d) = r.tag()?;
        match ty {
            MI_COMPRESSED => {
                let mut z = ZlibDecoder::new(d);
                let mut buf = Vec::new();
                z.read_to_end(&mut buf).map_err(|e| format!("décompression : {e}"))?;
                let mut r2 = Rd { b: &buf, p: 0 };
                while r2.p + 8 <= r2.b.len() {
                    let (t2, d2) = r2.tag()?;
                    if t2 == MI_MATRIX {
                        out.push(matrix(d2, 0)?);
                    }
                }
            }
            MI_MATRIX => out.push(matrix(d, 0)?),
            _ => {}
        }
    }
    Ok(out)
}

/// Courbe extraite d'une figure.
#[derive(Clone, Debug, Default)]
pub struct Curve {
    /// Nom (légende) éventuel.
    pub name: String,
    /// Abscisses.
    pub x: Vec<f64>,
    /// Ordonnées.
    pub y: Vec<f64>,
}

/// Axes d'une figure.
#[derive(Clone, Debug, Default)]
pub struct Axes {
    /// Titre.
    pub title: String,
    /// Légende de l'axe x.
    pub xlabel: String,
    /// Légende de l'axe y.
    pub ylabel: String,
    /// Courbes de données.
    pub curves: Vec<Curve>,
}

/// Extrait les axes et courbes d'une figure MATLAB (`.fig`).
pub fn figure_curves(bytes: &[u8]) -> Result<Vec<Axes>, String> {
    let vars = read(bytes)?;
    let root = vars
        .iter()
        .find(|(n, _)| n.starts_with("hgS"))
        .map(|(_, v)| v)
        .ok_or("aucune figure (hgS_…) dans ce fichier")?;
    let mut axes = Vec::new();
    walk(root, &mut axes, None);
    Ok(axes.into_iter().filter(|a: &Axes| !a.curves.is_empty()).collect())
}

fn children(v: &Value) -> Vec<Value> {
    match v.field("children") {
        // tableau de structures : chaque élément devient un nœud
        Some(Value::Struct { fields, elems, .. }) => elems
            .iter()
            .map(|e| Value::Struct { dims: vec![1, 1], fields: fields.clone(), elems: vec![e.clone()], class: String::new() })
            .collect(),
        Some(Value::Cell { elems, .. }) => elems.clone(),
        _ => Vec::new(),
    }
}

fn walk(node: &Value, out: &mut Vec<Axes>, cur: Option<usize>) {
    let ty = node.field("type").map(|t| t.text()).unwrap_or_default();
    let props = node.field("properties");
    let kids = children(node);
    let mut cur = cur;
    if ty == "axes" {
        let mut a = Axes::default();
        let _ = props;
        // `special` des axes = indices (base 1) des enfants texte : [?, x, y, titre]
        let sp = node.field("special").map(|v| v.nums().to_vec()).unwrap_or_default();
        let txt = |k: usize| -> String {
            sp.get(k)
                .and_then(|&i| kids.get((i as usize).wrapping_sub(1)))
                .and_then(|c| c.field("properties"))
                .and_then(|p| p.field("String"))
                .map(|s| s.text().trim().to_string())
                .unwrap_or_default()
        };
        a.xlabel = txt(1);
        a.ylabel = txt(2);
        a.title = txt(3);
        out.push(a);
        cur = Some(out.len() - 1);
    }
    if let (Some(i), Some(p)) = (cur, props)
        && (ty == "line" || ty.ends_with("lineseries") || ty.ends_with("stairseries")) {
            let x = p.field("XData").map(|v| v.nums().to_vec()).unwrap_or_default();
            let y = p.field("YData").map(|v| v.nums().to_vec()).unwrap_or_default();
            if y.len() > 2 {
                let x = if x.len() == y.len() { x } else { (0..y.len()).map(|k| (k + 1) as f64).collect() };
                let name = p.field("DisplayName").map(|v| v.text()).unwrap_or_default();
                out[i].curves.push(Curve { name, x, y });
            }
        }
    for c in &kids {
        walk(c, out, cur);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(ty: u32, data: &[u8]) -> Vec<u8> {
        let mut v = ty.to_le_bytes().to_vec();
        v.extend_from_slice(&(data.len() as u32).to_le_bytes());
        v.extend_from_slice(data);
        while !v.len().is_multiple_of(8) {
            v.push(0);
        }
        v
    }

    #[test]
    fn lit_un_tableau_double() {
        // fichier MAT minimal construit à la main : x = [1.5 2.5 3.5]
        let mut m = Vec::new();
        m.extend(element(MI_UINT32, &[6, 0, 0, 0, 0, 0, 0, 0])); // classe double
        let dims: Vec<u8> = [1i32, 3].iter().flat_map(|d| d.to_le_bytes()).collect();
        m.extend(element(MI_INT32, &dims));
        m.extend(element(MI_INT8, b"x"));
        let data: Vec<u8> = [1.5f64, 2.5, 3.5].iter().flat_map(|d| d.to_le_bytes()).collect();
        m.extend(element(MI_DOUBLE, &data));
        let mut f = vec![b' '; 128];
        f[..19].copy_from_slice(b"MATLAB 5.0 MAT-file");
        f[124] = 0;
        f[125] = 1;
        f[126] = b'I';
        f[127] = b'M';
        f.extend(element(MI_MATRIX, &m));
        let v = read(&f).unwrap();
        assert_eq!(v[0].0, "x");
        assert_eq!(v[0].1.nums(), &[1.5, 2.5, 3.5]);
    }
}
