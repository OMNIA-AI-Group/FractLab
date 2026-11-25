#!/bin/bash
# Construit dist/FractLab-x86_64.AppImage. Usage : packaging/build_appimage.sh [chemin/appimagetool]
# FRACTLAB_PRIVE=1 : copie de travail de l'auteur, avec le module privé
# (../prive/phi.rs), rangée dans ../prive/ et jamais dans dist/.
set -euo pipefail
PRIVE=${FRACTLAB_PRIVE:-}
if [ -n "$PRIVE" ]; then
    unset FRACTLAB_PUBLIC
else
    # version publique : le module privé n'est jamais compilé dans les paquets
    export FRACTLAB_PUBLIC=1
fi
cd "$(dirname "$0")/.."
TOOL=${1:-${APPIMAGETOOL:-appimagetool}}
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
# chemins locaux masqués dans les binaires publiés
# (CARGO_ENCODED_RUSTFLAGS : séparateur 0x1f, le chemin du projet peut contenir des espaces)
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$PWD=fractlab"$'\x1f'"--remap-path-prefix=$HOME=~"
cargo build --release
if [ -n "$PRIVE" ]; then
    grep -a -q "FRACTLAB-MODULE-PRIVE" target/release/fractlab || { echo "ERREUR : module privé absent" >&2; exit 1; }
else
    bash packaging/verifier_public.sh target/release/fractlab
fi
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
if [ -n "$PRIVE" ]; then OUT=../prive; NOM="FractLab-$VERSION-prive-x86_64.AppImage"; else OUT=dist; NOM="FractLab-$VERSION-x86_64.AppImage"; fi
mkdir -p "$OUT"
# construit dans un fichier temporaire puis le met à la place : fonctionne
# même si l'ancienne AppImage est en cours d'exécution
ARCH=x86_64 VERSION="$VERSION" "$TOOL" --no-appstream "$APPDIR" "$OUT/.$NOM.tmp"
mv -f "$OUT/.$NOM.tmp" "$OUT/$NOM"
# icône de l'AppImage dans le gestionnaire de fichiers (vignette à jour)
python3 packaging/vignette.py "$OUT/$NOM" assets/icon.png || true
ls -la "$OUT/$NOM"
