#!/usr/bin/env python3
"""Assemble la version navigateur de FractLab en UN SEUL fichier HTML :
module WebAssembly compressé (gzip, base64), code de liaison JavaScript et
écran de chargement. La page s'ouvre par double-clic (sans serveur ni
connexion) dans Chrome, Edge ou Firefox récents, et sert aussi telle quelle
pour la version en ligne.
Usage : web_page.py dossier_wasm_bindgen icone.png version sortie.html"""
import base64, gzip, json, sys

src, icon, version, out = sys.argv[1:5]
glue = open(f"{src}/fractlab.js", encoding="utf-8").read()
glue_json = json.dumps(glue)
assert "</script" not in glue
wasm = open(f"{src}/fractlab_bg.wasm", "rb").read()
wasm_b64 = base64.b64encode(gzip.compress(wasm, 9)).decode()
icon_b64 = base64.b64encode(open(icon, "rb").read()).decode()

html = f"""<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>FractLab {version} — Magnetic Engine Simulator</title>
<meta name="description" content="FractLab — simulateur libre de machines à aimants permanents. Alain VALETTE-CLARY, ΩiA-Group. GPL-3.0.">
<link rel="icon" type="image/png" href="data:image/png;base64,{icon_b64}">
<style>
  html, body {{ margin: 0; padding: 0; height: 100%; overflow: hidden; background: #0b1220; }}
  #fractlab_canvas {{ position: absolute; inset: 0; width: 100%; height: 100%; outline: none; }}
  #chargement {{ position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 18px; color: #cfe8ff; font: 16px system-ui, sans-serif; background: #0b1220; }}
  #chargement img {{ width: 128px; height: 128px; border-radius: 22px; box-shadow: 0 0 40px #00c8ff55; }}
  #chargement .barre {{ width: 260px; height: 6px; border-radius: 3px; background: #1c2a40; overflow: hidden; }}
  #chargement .barre div {{ width: 40%; height: 100%; background: #00c8ff; animation: va 1.2s ease-in-out infinite; }}
  @keyframes va {{ 0% {{ transform: translateX(-100%); }} 100% {{ transform: translateX(260%); }} }}
  #erreur {{ color: #ff8a80; max-width: 640px; text-align: center; }}
</style>
</head>
<body>
<canvas id="fractlab_canvas"></canvas>
<div id="chargement">
  <img alt="FractLab" src="data:image/png;base64,{icon_b64}">
  <div><b>FractLab {version}</b> — Magnetic Engine Simulator</div>
  <div class="barre"><div></div></div>
  <div id="etat">Chargement…</div>
  <div id="erreur"></div>
</div>
<script>
{glue}
</script>
<script type="module">
const WASM_GZ_B64 = "{wasm_b64}";
const GLUE = {glue_json};
const etat = document.getElementById("etat");
try {{
  if (!("DecompressionStream" in window)) throw new Error("navigateur trop ancien (DecompressionStream absent) : utilisez Chrome, Edge ou Firefox récents");
  // module WebAssembly : base64 → gzip → octets → compilation (une seule fois, partagée avec les Workers)
  const gz = await (await fetch("data:application/octet-stream;base64," + WASM_GZ_B64)).blob();
  const bytes = await new Response(gz.stream().pipeThrough(new DecompressionStream("gzip"))).arrayBuffer();
  etat.textContent = "Compilation…";
  const module = await WebAssembly.compile(bytes);
  await wasm_bindgen({{ module_or_path: module }});
  // Workers de calcul (même module) : l'interface ne gèle jamais pendant un calcul.
  // Test au démarrage ; sans Worker, les calculs se font dans la page.
  const workerSrc = GLUE + `
let pret = false; const attente = [];
self.onmessage = async (e) => {{
  if (e.data && e.data.module) {{ await wasm_bindgen({{ module_or_path: e.data.module }}); pret = true; for (const q of attente) wasm_bindgen.worker_entry(q); attente.length = 0; return; }}
  if (e.data === "ping") {{ self.postMessage("pong"); return; }}
  const q = new Uint8Array(e.data);
  if (!pret) {{ attente.push(q); return; }}
  wasm_bindgen.worker_entry(q);
}};`;
  const workerUrl = URL.createObjectURL(new Blob([workerSrc], {{ type: "text/javascript" }}));
  const nouveau = () => {{ const w = new Worker(workerUrl); w.postMessage({{ module }}); return w; }};
  const ok = await new Promise((res) => {{
    let w;
    try {{ w = nouveau(); }} catch (e) {{ return res(false); }}
    const t = setTimeout(() => {{ w.terminate(); res(false); }}, 8000);
    w.onerror = () => {{ clearTimeout(t); w.terminate(); res(false); }};
    w.onmessage = (e) => {{ if (e.data === "pong") {{ clearTimeout(t); w.terminate(); res(true); }} }};
    w.postMessage("ping");
  }});
  if (ok) window.__fractlab_worker = nouveau;
  else console.warn("FractLab : Web Workers indisponibles, calculs dans la page");
  etat.textContent = "Démarrage…";
  await wasm_bindgen.start("fractlab_canvas");
  document.getElementById("chargement").remove();
}} catch (e) {{
  etat.textContent = "FractLab n'a pas pu démarrer.";
  document.getElementById("erreur").textContent = String(e && e.message || e);
  console.error(e);
}}
</script>
</body>
</html>
"""
open(out, "w", encoding="utf-8").write(html)
print(f"{out} : {len(html)/1e6:.1f} Mo (wasm {len(wasm)/1e6:.1f} Mo, compressé {len(wasm_b64)*3/4/1e6:.1f} Mo)")
