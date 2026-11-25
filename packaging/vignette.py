#!/usr/bin/env python3
"""Met à jour la vignette (icône) de l'AppImage dans le cache des
gestionnaires de fichiers (spécification freedesktop des vignettes) :
après chaque reconstruction, l'ancienne vignette n'est plus valide et le
gestionnaire de fichiers affichait une icône générique.
Usage : vignette.py fichier.AppImage icone.png"""
import hashlib, os, pathlib, struct, sys, zlib

app = pathlib.Path(sys.argv[1]).resolve()
png = open(sys.argv[2], "rb").read()
uri = app.as_uri()
mtime = str(int(app.stat().st_mtime))
h = hashlib.md5(uri.encode()).hexdigest()

def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

# insertion des métadonnées Thumb::URI et Thumb::MTime après l'entête IHDR
ihdr_end = 8 + 8 + 13 + 4
meta = chunk(b"tEXt", b"Thumb::URI\0" + uri.encode()) + chunk(b"tEXt", b"Thumb::MTime\0" + mtime.encode())
out = png[:ihdr_end] + meta + png[ihdr_end:]
base = pathlib.Path(os.environ.get("XDG_CACHE_HOME", pathlib.Path.home() / ".cache")) / "thumbnails"
for d in ("normal", "large", "x-large", "xx-large"):
    p = base / d / f"{h}.png"
    if d in ("normal", "large") or p.exists():
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(out)
        os.chmod(p, 0o600)
for f in (base / "fail").glob(f"*/{h}.png"):
    f.unlink()
print(f"vignette à jour : {h}.png")
