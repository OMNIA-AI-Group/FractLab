//! Icône et informations de version de l'exécutable Windows ; détection du
//! module privé facultatif.
//!
//! Si le fichier `../prive/phi.rs` existe à côté du dépôt, il est compilé
//! dans l'application (option `phi`) : c'est la copie de travail de l'auteur.
//! Les versions publiées sont construites avec `FRACTLAB_PUBLIC=1`, qui
//! ignore ce fichier ; un clone du dépôt ne l'a de toute façon pas.
fn main() {
    println!("cargo:rerun-if-changed=packaging/windows/fractlab.rc");
    println!("cargo:rerun-if-changed=packaging/windows/fractlab.ico");
    println!("cargo:rustc-check-cfg=cfg(phi)");
    println!("cargo:rerun-if-env-changed=FRACTLAB_PUBLIC");
    let prive = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../prive/phi.rs");
    println!("cargo:rerun-if-changed={}", prive.display());
    if std::env::var_os("FRACTLAB_PUBLIC").is_none() && prive.exists() {
        println!("cargo:rustc-cfg=phi");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("packaging/windows/fractlab.rc", embed_resource::NONE).manifest_optional().unwrap();
    }
}
