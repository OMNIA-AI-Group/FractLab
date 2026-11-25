#!/bin/bash
# Refuse un binaire qui contiendrait le module privé facultatif
# (../prive/phi.rs) : chaque paquet publié doit être la version publique.
# Usage : verifier_public.sh fichier…
set -euo pipefail
for f in "$@"; do
    if grep -a -q "FRACTLAB-MODULE-PRIVE" "$f"; then
        echo "ERREUR : $f contient le module privé — construire avec FRACTLAB_PUBLIC=1" >&2
        exit 1
    fi
done
