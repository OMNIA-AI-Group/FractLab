# Architecture de FractLab

FractLab est une bibliothèque Rust (`src/lib.rs`) et une application graphique
(`src/main.rs`, interface egui). Le calcul est entièrement séparé de
l'interface : chaque module de calcul est testable seul.

## Modules

| Module | Rôle |
|---|---|
| `geometry.rs` | `MotorSpec` (le projet), positions des aimants, balourd, autocorrélation, **contrôle de chevauchement des aimants**, empilement axial, contrôle de la géométrie avant calcul. |
| `repartition.rs` | Passerelle des répartitions d'aimants, du projet de départ et des seuils des critères. Un module privé facultatif (`../prive/phi.rs`, détecté par `build.rs`, ignoré avec `FRACTLAB_PUBLIC=1`) peut y ajouter ses répartitions, ses seuils, ses traductions et son aide ; sans lui, l'application fonctionne de la même façon (répartition régulière et positions personnalisées). |
| `demo.rs` | Moteurs de démonstration génériques (drone 9N12P, servomoteur 12N8P, moteur-roue 36N42P, génératrice axiale 18N16P). |
| `magnets.rs` | Découpage de la couronne en morceaux d'aimant (radial, parallèle, Halbach) et séries de Fourier de l'aimantation sur 360°. |
| `section.rs` | **Cœur physique** : solveur général d'une section 2D en sous-domaines (encoches, isthmes, entrefer, aimants), en géométrie polaire ou plane (tranche axiale). Système des encoches factorisé une fois, puis élimination 4×4 par harmonique. Champ, couple, forces, potentiel moyen d'encoche. |
| `subdomain.rs` | Machine complète au-dessus de `section.rs` : rotor extérieur ou intérieur, flux axial multi-tranches (tranches construites en parallèle), nombre d'harmoniques adapté à la géométrie, courants d'induit, flux de phase, inductances, force transverse et attraction axiale, bobinage général (`CoilDef`). |
| `legacy.rs` | Ancien modèle rotor extérieur, conservé pour les tests de non-régression. |
| `par.rs` | Calcul parallèle sur tous les cœurs (bibliothèque standard seule) : chaque position du rotor est calculée par le premier fil libre. |
| `saturation.rs` | Saturation par la méthode du facteur de saturation : courbes B(H) (M270-35A, M400-50A, SMC, acier 1010), point de fonctionnement du circuit magnétique, FEM et couple corrigés, courbe couple / courant. |
| `sweep.rs` | Étude paramétrique : paramètres, grandeurs de sortie, export CSV. |
| `recalage.rs` | Recalage sur mesures (FEM ou cogging) : Br, entrefer ou longueur effective ajustés par simplexe, origine angulaire recalée. |
| `platform.rs` | Ce qui dépend du système : fichiers (dialogues natifs, ou sélecteur et téléchargements du navigateur, plusieurs fichiers en ZIP), son, ouverture de PDF, impression, réglages persistants. Les fichiers choisis arrivent par une boîte aux lettres relevée à chaque image. |
| `bg.rs` | Requêtes et réponses des calculs de fond sous forme sérialisée (bincode) ; c'est ce qui circule entre l'interface et le Web Worker. Testé sur ordinateur par un aller-retour complet. |
| `web.rs` | Version navigateur (WebAssembly) : démarrage dans un `<canvas>`, Web Workers de calcul, repli dans la page si les Workers sont indisponibles. |
| `tips.rs` | Bulles d'aide au passage de la souris (4 langues), retrouvées depuis le libellé affiché. |
| `linalg.rs` | Factorisation LU dense à pivot partiel, boucles vectorisées (tranches contiguës, produit scalaire à quatre accumulateurs). |
| `winding.rs` | Bobinage concentré déduit de l'étoile des encoches. |
| `dynsim.rs` | Simulation dynamique dans le domaine des phases (RK4) : réseau, onduleur MLI, commutation 120°, redresseur à diodes ; charges mécaniques ; défauts (phase ouverte, court-circuit entre spires). |
| `optimize.rs` | Ajustement I.A : Nelder–Mead multi-départs avec pénalités, 5 meilleures solutions distinctes en direct ; chaque essai est tracé (étape, contraintes violées, nouveau meilleur) avec les événements de la résolution ; culasse rotor dimensionnée contre la saturation. |
| `script.rs` | Console de script : commandes (variante, réglage, calcul, ondulation, inductance, balayage, comparaison, Ajustement I.A tracé, application au projet) exécutées en tâche de fond ; mise en forme des traces du moteur I.A. |
| `matfile.rs` | Lecture des fichiers MATLAB niveau 5 (.mat, .fig) sans MATLAB : zlib, UTF-16, struct/cell/objet, courbes et légendes des figures. |
| `doc.rs` | Rédaction : fiche du projet (textes par langue), document structuré, LaTeX, PDF intégré (logo et version en en-tête), écriture .mat. |
| `dynamics.rs` | Grandeurs selon la vitesse : pertes fer (Bertotti), roulements (Palmgren), ventilation, thermique, Br(T), vibration radiale et de torsion, raies acoustiques, synthèse audio WAV. |
| `analysis.rs` | Balayages (cogging, FEM, forces), fonctionnement en charge, excentricité et défauts d'alignement axiaux (méthode de perméance), spectres, métriques du couple de détente, cartes de champ (polaires, tranche axiale développée, entrefer sur tout le disque), rapport. |
| `fdcheck.rs` | **Solveur de contrôle indépendant** : volumes finis polaires ou plans (tranche de machine axiale : rayon moyen comme rayon métrique), toute section, becs de dents, courants ; gradient conjugué préconditionné par Cholesky en bande. |
| `compute.rs` | Tâches de fond (calcul principal, comparaison, cartes 3D, validation) communiquant par canal. |
| `exchange.rs` | Imports / exports (FEMM, Gmsh, DXF, SVG, STL pour les 3 topologies, LaTeX, HTML, Octave, Python, projets SIMUMSAP, mesures). |
| `view3d.rs` | Rendu 3D logiciel (caméra orbitale, surfaces, vue moteur) dans egui. |
| `app/` | Interface, découpée par rôle : `mod.rs` (état, propagation dynamique des calculs, boucle), `menu.rs`, `parametres.rs`, `vues.rs` (onglets, plein écran), `onglets_mesure.rs` (synthèse, géométrie, champ, cogging, FEM, forces, charge + saturation), `onglets_etude.rs` (dynamique, comparaison, moteur 3D, cartes 3D, validation), `onglet_parametrique.rs`, `recalage.rs`, `axial.rs` (flux axial : vue de face superposée, coupe axiale, moteur 3D en disques, onglet Champ par tranche, entrefer sur tout le disque), `criteres.rs` (bandeau des critères), `console.rs` (console : déroulé du moteur I.A en direct, console de script), `fenetres.rs` (préférences, aide, développeur, SIMUMSAP, figures, Ajustement I.A, licence, à propos), `redaction.rs` (document, impression, sauvegarde générale), `fichiers.rs` (imports / exports), `widgets.rs` (cartes, vignettes, courbes). |
| `i18n.rs` | Traductions (clé = texte français ; anglais, espagnol, allemand). |
| `help.rs` | Guide d'utilisation en 4 langues. |
| `units.rs` | Unités d'affichage (calculs toujours en SI). |

## Version navigateur

`packaging/build_web.sh` compile la bibliothèque pour `wasm32-unknown-unknown`,
génère la liaison JavaScript (`wasm-bindgen --target no-modules`) puis
`packaging/web_page.py` assemble **un seul fichier HTML** : module
WebAssembly compressé (gzip + base64, décompressé par `DecompressionStream`),
liaison et écran de chargement. La page compile le module une fois et le
transmet à chaque Web Worker ; chaque tâche de fond (`compute::launch`,
`optimize::run`, `recalage::run`, `dynsim::run`) part dans son Worker, et
le Worker est arrêté quand la tâche est abandonnée. Le même fichier sert
hors ligne (double-clic) et en ligne (GitHub Pages).

## Propagation dynamique

Après chaque calcul principal, `App::propagate` met à jour l'excentricité,
relance la simulation dynamique et remet en file les vues lourdes déjà
calculées (comparaison, cartes 3D, validation, étude paramétrique) ;
`App::poll_followers` les relance dès que le projet est stable (pas de
saisie, pas de calcul ni d'Ajustement I.A en cours). L'Ajustement I.A peut
appliquer en direct sa meilleure proposition : chaque nouvelle meilleure
solution déclenche ce même enchaînement.

## Flux d'un calcul

1. L'utilisateur modifie un paramètre → `App::touched` marque le projet modifié.
2. Après 0,35 s sans saisie, `compute::run_main` lance un fil de calcul.
3. `Model::new` assemble et factorise le système des encoches (≈ 40 ms).
4. `analysis::cogging / emf / forces` appellent `Model::solve` pour chaque
   position (≈ 0,5 ms par position).
5. Le résultat revient par `Msg::Main` ; les vues concernées sont marquées
   « nouveau » (vignettes clignotantes).

## Tests

`cargo test --release` couvre :
- l'identité des angles d'aimants avec le script Python d'origine ;
- la cohérence physique (couple indépendant du rayon d'intégration,
  continuité du potentiel, périodicité, force nulle par symétrie, FEM
  équilibrée) ;
- le contrôle de chevauchement ;
- les imports/exports (STL fermé, DXF, SVG, Lua, Gmsh, CSV, JSON, SIMUMSAP) ;
- l'équivalence du nouveau solveur avec l'ancien, la limite plane du polaire,
  l'inductance positive et réciproque, le bilan énergétique en charge ;
- la simulation dynamique (démarrage, éolienne sur redresseur, phase ouverte)
  et l'optimisation (cogging divisé par deux, solution admissible) ;
- la lecture .mat / .fig, la sortie LaTeX/PDF, l'écriture .mat ;
- le calcul parallèle (ordre conservé), la saturation (courbes B(H)
  monotones, saturation croissante avec l'induction), l'étude paramétrique
  (tendances physiques), le recalage (paramètres retrouvés sur une mesure
  synthétique bruitée) ;
- la complétude des traductions : le test lit directement les fichiers de
  l'interface et du document, toute chaîne affichée doit être traduite ;
- la complétude du guide.

Les bancs `examples/validation.rs` et `validation2.rs` comparent le modèle
analytique au solveur volumes finis (encoches semi-fermées, rotor intérieur,
courants) ; `examples/simumsap_compare.rs` compare un projet SIMUMSAP à ses figures ; `examples/script.rs` exécute un script de console hors interface ; `examples/exports.rs` génère tous les
exports pour contrôle par des outils externes (Gmsh, lecteurs DXF/STL/SVG).
