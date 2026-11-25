//! Génère le document (LaTeX + PDF intégré) et la sauvegarde MATLAB d'un projet.
use fractlab::compute::{self, Msg, Settings};
use fractlab::doc;
use fractlab::geometry::MotorSpec;
fn main() {
    let out = std::env::args().nth(1).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    let job = compute::run_main(MotorSpec::irregular_example(), Settings::default());
    let Msg::Main(Ok(r)) = job.rx.recv().unwrap() else { panic!() };
    let lang = match std::env::args().nth(2).as_deref() { Some("en") => fractlab::i18n::Lang::En, Some("es") => fractlab::i18n::Lang::Es, Some("de") => fractlab::i18n::Lang::De, _ => fractlab::i18n::Lang::Fr };
    let d = doc::build(&r.spec.sheet, &r, &[], None, lang);
    std::fs::write(format!("{out}/doc.tex"), doc::to_latex(&d)).unwrap();
    std::fs::write(format!("{out}/doc.pdf"), doc::to_pdf_builtin(&d)).unwrap();
    std::fs::write(format!("{out}/donnees.mat"), doc::results_mat(&r, None)).unwrap();
    println!("ok");
}
