//! Liste les courbes de toutes les figures MATLAB d'un dossier (contrôle du lecteur).
fn main() {
    let dir = std::env::args().nth(1).expect("dossier");
    let mut files: Vec<_> = walk(std::path::Path::new(&dir));
    files.sort();
    for f in files {
        let b = std::fs::read(&f).unwrap();
        match fractlab::matfile::figure_curves(&b) {
            Ok(ax) => {
                let n: usize = ax.iter().map(|a| a.curves.len()).sum();
                let first = ax.first().and_then(|a| a.curves.first());
                println!("{} | {} axes, {} courbes | x:{:?} y:{:?} | {:?} / {:?}", f.file_name().unwrap().to_string_lossy(), ax.len(), n,
                    first.map(|c| (c.x.len(), c.x.first().copied(), c.x.last().copied())), first.map(|c| (c.y.iter().cloned().fold(f64::MAX, f64::min), c.y.iter().cloned().fold(f64::MIN, f64::max))),
                    ax.first().map(|a| a.xlabel.clone()), ax.first().map(|a| a.ylabel.clone()));
            }
            Err(e) => println!("{} | ERREUR {e}", f.display()),
        }
    }
}
fn walk(p: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut v = Vec::new();
    for e in std::fs::read_dir(p).unwrap().flatten() {
        let q = e.path();
        if q.is_dir() { v.extend(walk(&q)); } else if q.extension().map(|x| x == "fig").unwrap_or(false) { v.push(q); }
    }
    v
}
#[allow(dead_code)]
fn _unused() {}
