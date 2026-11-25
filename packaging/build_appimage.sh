#!/bin/bash
# Construit dist/FractLab-x86_64.AppImage. Usage : packaging/build_appimage.sh [chemin/appimagetool]
set -euo pipefail
# version publique : le module privé (../prive/phi.rs) n'est jamais compilé dans les paquets
export FRACTLAB_PUBLIC=1
cd "$(dirname "$0")/.."
TOOL=${1:-${APPIMAGETOOL:-appimagetool}}
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
# chemins locaux masqués dans les binaires publiés
# (CARGO_ENCODED_RUSTFLAGS : séparateur 0x1f, le chemin du projet peut contenir des espaces)
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$PWD=fractlab"$'\x1f'"--remap-path-prefix=$HOME=~"
cargo build --release
bash packaging/verifier_public.sh target/release/fractlab
APPDIR=target/FractLab.AppDir
rm -rf "$APPDIR"; mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/icons/hicolor/256x256/apps" "$APPDIR/usr/share/doc/fractlab"
install -m755 target/release/fractlab "$APPDIR/usr/bin/fractlab"
strip "$APPDIR/usr/bin/fractlab"
cp packaging/fractlab.desktop "$APPDIR/fractlab.desktop"
cp packaging/fractlab.desktop "$APPDIR/usr/share/applications/"
cp assets/icon.png "$APPDIR/fractlab.png"
cp assets/icon.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/fractlab.png"
cp README*.md CHANGELOG.md LICENSE docs/*.md "$APPDIR/usr/share/doc/fractlab/"
cat > "$APPDIR/AppRun" <<'RUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/fractlab" "$@"
RUN
chmod +x "$APPDIR/AppRun"
mkdir -p dist
# construit dans un fichier temporaire puis le met à la place : fonctionne
# même si l'ancienne AppImage est en cours d'exécution
ARCH=x86_64 VERSION="$VERSION" "$TOOL" --no-appstream "$APPDIR" "dist/.FractLab-$VERSION-x86_64.AppImage.tmp"
mv -f "dist/.FractLab-$VERSION-x86_64.AppImage.tmp" "dist/FractLab-$VERSION-x86_64.AppImage"
# icône de l'AppImage dans le gestionnaire de fichiers (vignette à jour)
python3 packaging/vignette.py "dist/FractLab-$VERSION-x86_64.AppImage" assets/icon.png || true
ls -la dist/
