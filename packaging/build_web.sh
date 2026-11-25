#!/bin/bash
# Construit la version navigateur : dist/FractLab-<version>.html (un seul
# fichier, fonctionne hors ligne) et dist/web/index.html (version en ligne).
# Nécessite : cible wasm32-unknown-unknown et wasm-bindgen-cli de la même
# version que la crate wasm-bindgen (voir Cargo.lock).
set -euo pipefail
# version publique : le module privé (../prive/phi.rs) n'est jamais compilé dans les paquets
export FRACTLAB_PUBLIC=1
cd "$(dirname "$0")/.."
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
# chemins locaux masqués dans le module publié
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$PWD=fractlab"$'\x1f'"--remap-path-prefix=$HOME=~"
cargo build --release --lib --target wasm32-unknown-unknown
bash packaging/verifier_public.sh target/wasm32-unknown-unknown/release/fractlab.wasm
rm -rf target/web
wasm-bindgen --target no-modules --no-typescript --out-dir target/web target/wasm32-unknown-unknown/release/fractlab.wasm
mkdir -p dist/web
python3 packaging/web_page.py target/web assets/icon.png "$VERSION" "dist/FractLab-$VERSION.html"
cp "dist/FractLab-$VERSION.html" dist/web/index.html
ls -la dist/ dist/web/
