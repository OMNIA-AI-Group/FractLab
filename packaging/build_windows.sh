#!/bin/bash
# Construit l'installateur Windows et la version portable (compilation croisée
# depuis Linux : cible x86_64-pc-windows-gnu, mingw-w64, NSIS).
#   dist/FractLab-<version>-Setup.exe
#   dist/FractLab-<version>-windows-portable.zip
set -euo pipefail
# version publique : le module privé (../prive/phi.rs) n'est jamais compilé dans les paquets
export FRACTLAB_PUBLIC=1
cd "$(dirname "$0")/.."
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
# chemins locaux masqués dans les binaires publiés
# (CARGO_ENCODED_RUSTFLAGS : séparateur 0x1f, le chemin du projet peut contenir des espaces)
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$PWD=fractlab"$'\x1f'"--remap-path-prefix=$HOME=~"
cargo build --release --target x86_64-pc-windows-gnu
bash packaging/verifier_public.sh target/x86_64-pc-windows-gnu/release/fractlab.exe
STAGE=packaging/windows/winstage
rm -rf "$STAGE"; mkdir -p "$STAGE" dist
x86_64-w64-mingw32-strip -o "$STAGE/fractlab.exe" target/x86_64-pc-windows-gnu/release/fractlab.exe
cp packaging/windows/fractlab.ico "$STAGE/"
sed 's/$/\r/' LICENSE > "$STAGE/LICENSE.txt"
for f in README*.md CHANGELOG.md; do sed "s/$/\r/" "$f" > "$STAGE/$f"; done
(cd packaging/windows && makensis -V2 -DVERSION="$VERSION" fractlab.nsi)
# version portable : le même contenu, sans installation
PORT=$(mktemp -d)
cp -r "$STAGE" "$PORT/FractLab"
rm -f "dist/FractLab-$VERSION-windows-portable.zip"
(cd "$PORT" && zip -qr9 "$OLDPWD/dist/FractLab-$VERSION-windows-portable.zip" FractLab)
rm -rf "$PORT"
ls -la dist/
