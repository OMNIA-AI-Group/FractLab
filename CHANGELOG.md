# Journal des versions de FractLab

Format : une section par version, la plus récente en premier.
*English summary at the end of each section.*

## 0.1.10 — interface professionnelle : espace de travail, annuler, portable et 4K

- **Espace de travail** : Ajustement I.A, console et comparaison des solutions
  deviennent des onglets de la zone centrale (plus de fenêtres flottantes qui
  se chevauchent).
- **Navigation par étapes d'étude** : Conception, Analyse, Optimisation,
  Validation ; chaque étape retient son dernier onglet.
- **Projet** : annuler / rétablir (⟲ ⟳, Ctrl+Z, Ctrl+Y), « ● modifié » et
  liste des modifications depuis l'enregistrement, projets récents.
- **Recherche dans les paramètres** (loupe du panneau) : champs encadrés en
  jaune, sections dépliées.
- **Jauge de qualité de conception** dans la Synthèse (critères tenus sur
  critères évalués, détail au survol) ; **accueil** au premier lancement.
- **Portable et 4K** : panneau des paramètres proportionné à l'écran, barre
  d'actions qui passe à la ligne, cartes de la Synthèse sans débordement,
  comparaison des solutions en colonnes sur grand écran et superposée sur
  portable, zoom automatique jusqu'à ×1,7 en 4K.
- Couleurs harmonisées (une couleur d'action principale), pastilles
  « nouveau » sans clignotement.
- **Démo** : l'étape Ajustement I.A n'est plus sautée quand l'affichage 3D
  ralentit les images.
- Guide : nouvelle section « Espace de travail et gestion du projet »,
  section Ajustement I.A réécrite (objectifs, contraintes, console,
  comparaison).

*English — Workspace tabs instead of floating windows, study-step
navigation, undo/redo, modified-project indicator and change list, recent
projects, parameter search, design-quality gauge, first-launch welcome;
layouts checked on a laptop (1366×768) and in 4K; demo A.I. step fixed;
guide updated.*

## 0.1.9 — comparaison des solutions, contraintes et objectifs spectraux de l'Ajustement I.A

- **Comparaison des solutions** (après un Ajustement I.A, bouton
  « ⚖ Comparer les solutions » dans la fenêtre de l'ajustement et dans la
  console) : le moteur original et les 5 meilleures propositions (plus
  l'essai choisi dans le graphe 3D) recalculés en calcul complet ; tableau
  de 18 métriques avec l'écart à l'original (vert / rouge selon le sens
  favorable) et la meilleure valeur de chaque ligne ; graphiques superposés
  (couple de détente, spectre, FEM, force, couple en charge) et profil
  normalisé ; « Appliquer » ou « Garder l'original » ; tableau copiable en
  CSV.
- **Journal des essais** : fixé en bas de l'onglet Moteur I.A (il n'est plus
  poussé hors de la fenêtre par le graphe) ; déplié, au moins 5 lignes, et
  35 % de la hauteur quand la fenêtre est agrandie.
- **Menu Calcul** : « Ajustement I.A… » et « Comparaison des solutions… ».
- **Console** : la fenêtre ne s'allonge plus d'elle-même jusqu'au bas de l'écran (le bouton du journal restait inaccessible) ; taille proportionnelle à l'écran, recalculée quand l'application change nettement de taille.
- **Comparaison des solutions, nouvelle mise en page** : elle s'ouvre
  d'elle-même à la fin de chaque Ajustement I.A ; métriques à gauche (écart
  à l'original sous chaque valeur), six graphiques visibles en même temps à
  droite (couple de détente, spectre réduit aux raies significatives, FEM,
  force, couple en charge, profil normalisé en barres horizontales) ; un
  clic sur un titre agrandit le graphique ; fenêtre à la taille de l'écran ;
  message et bouton « Lancer un Ajustement I.A » quand il n'y a encore rien
  à comparer.
- **Contraintes de l'Ajustement I.A** : « pertes Joule ≤ original + x % » et
  « couple moyen ≥ x % de l'original » (script : `joule=10`, `couple=0.95`).
  Le couple moyen de l'estimation rapide est calculé par la puissance
  électromagnétique (exact même avec une forte ondulation).
- **Historique des comparaisons** : chaque comparaison est gardée (date à
  l'heure de France, projet, objectif, réglages, métriques, solution
  retenue) et se rouvre d'une séance à l'autre ; tableau et profil tout de
  suite, courbes recalculées à la demande.
- **Ajustement I.A** : amplitude minimale des décalages réglable quand
  l'amplitude est un réglage libre (empêche l'ajustement de revenir à un
  rotor régulier) ; commande de script `ia … amin=2`. Trois objectifs
  spectraux : spectre le plus étalé (raie dominante minimale), entropie
  spectrale maximale, compromis spectre étalé / cogging faible ; part de la
  dominante et entropie affichées pour chaque essai et dans le bilan.
- **Paquets** : `FRACTLAB_PRIVE=1 packaging/build_appimage.sh` construit la copie de travail avec le module privé, hors de `dist/`.
- **Inductance** : banc `examples/inductance_fv.rs` (modèle analytique
  contre volumes finis, tranche par tranche) ; accord à moins de 1 %, avec
  ou sans becs de dents. Le modèle à encoches ouvertes de SIMUMSAP surestime
  l'inductance d'environ 20 % sur des encoches larges (voir docs/MODELE.md).

*English — Copper-loss and torque constraints for A.I. tuning; persistent comparison history. Solution comparison window after A.I. tuning (original and proposals fully recomputed, 18 metrics, overlaid charts, normalised profile, apply or keep the original, CSV); trial log shows at least 5 lines. Minimum offset amplitude and three spectral goals (most spread spectrum, maximum entropy, spread/low-cogging trade-off) for A.I. tuning; inductance checked
against finite volumes (< 1 %); SIMUMSAP's open-slot model overestimates
inductance by about 20 % on wide slots.*

## 0.1.8 — console du moteur I.A, graphe 3D des essais, démo

- **Graphe 3D des possibilités explorées** (console, onglet Moteur I.A) :
  chaque essai de l'Ajustement I.A est un nœud relié au meilleur essai de
  son départ ; le graphe se construit pendant le calcul et tourne sur 360°
  pendant et après. Clic sur un nœud : détail et « Appliquer cet essai ».
  Le journal des essais passe dans un menu déroulant sous le graphe ; le
  bilan des résultats reste toujours visible.
- **Démo** (menu Aide) : confirmation (le projet en cours est mis de côté),
  choix entre quatre moteurs de démonstration (drone 9N12P, servomoteur
  12N8P, moteur-roue 36N42P, génératrice axiale 18N16P), paramétrage champ
  par champ encadré en rouge, calcul, visite des onglets, moteur 3D avec le
  champ, Ajustement I.A. « ÉCHAP — sortir de la DÉMO » clignote en rouge ;
  Échap l'arrête. Rubrique « Démo » dans le guide.
- Le projet de départ est désormais un moteur de démonstration générique ;
  les seuils de la synthèse sont génériques (THD de la FEM ≤ 5 %).

- **Console** (bouton 🖥 de la barre principale, et depuis l'Ajustement I.A) :
  - onglet **Moteur I.A** : le déroulé de la résolution en direct. On y voit
    chaque essai (départ, étape du simplexe, valeurs, score, cogging, force,
    FEM, ondulation, induction de la culasse rotor), les contraintes violées,
    les ★ nouveaux meilleurs, les départs et leur convergence, et un bilan
    départ → meilleure solution faisable avec l'évolution de chaque grandeur.
    Filtres (tous, nouveaux meilleurs, rejetés) et copie du journal.
  - onglet **Script** : des commandes enchaînées sur une copie du projet
    (`variante`, `regle`, `montre`, `calcul`, `ondulation`, `inductance`,
    `balaye`, `compare`, `ia`, `applique`, `initial`, `aide`), en script
    complet ou ligne par ligne avec historique. Les mêmes scripts tournent
    hors interface : `cargo run --release --example script -- fichier.txt`.
- **Ajustement I.A** : nouvel objectif « Ondulation de couple minimale »
  (couple moyen conservé). Nouvelle option « Culasse rotor non saturée » :
  pour chaque essai, l'épaisseur de culasse est portée à la valeur qui ramène
  son induction à vide au coude du matériau. L'ondulation et l'induction de
  culasse s'affichent pour chaque proposition.
- **Charge** : l'ondulation de couple est décomposée en couple de détente,
  couple mutuel (FEM × courant) et reste (réluctance et harmoniques).
*English — Rotating 3D graph of the explored A.I. trials above a collapsible log; guided demo (Help menu) on four generic demonstration motors, Esc to exit; the start-up project is now a generic demonstration motor. New console: live trace of the A.I. engine (every trial, simplex
step, violated constraints, new bests, start → best summary) and a script
console (commands chained on a copy of the project, also runnable from the
command line). A.I. tuning: new “minimum torque ripple” goal and automatic
rotor-yoke sizing to avoid saturation. On-load ripple broken down into
cogging, mutual and reluctance parts.*

## 0.1.7 — changement de type de machine

- **Moteur 3D** : après un changement de type de machine (ou de toute
  cote), la nouvelle forme est dessinée tout de suite, sans attendre la fin
  du calcul (plusieurs secondes pour une machine axiale) ; un bandeau
  indique que les grandeurs seront mises à jour à la fin du calcul.
- **Retour du flux axial au flux radial** : le calcul échouait (« matrice
  singulière ») parce que la profondeur d'encoche réglée pour le disque
  axial ne tenait pas dans le stator radial. Corrigé.
- **Retour du rotor intérieur au rotor extérieur** : « géométrie
  impossible », le rayon intérieur du stator restait celui du rotor
  intérieur. Corrigé : chaque type de machine retrouve ses propres cotes.
- Contrôle de la géométrie avant calcul : une géométrie impossible donne un
  message clair au lieu d'une erreur numérique.

*English — 3D motor redraws the new machine shape immediately after a
change of machine type; fixed computation failures when switching back
from axial to radial and from inner to outer rotor; each machine type keeps
its own dimensions; clear message for impossible geometries.*

## 0.1.6 — le champ 3D mis en avant

- Bouton violet **« 👁 Champ 3D »** dans la barre principale : un clic ouvre
  le moteur 3D en vue transparente avec le champ adapté à la machine
  (carte + lignes de flux en radial, carte + vecteurs Bz en axial).
- Grand bouton en tête de l'onglet Moteur 3D (« Voir le champ magnétique à
  travers le moteur », puis « désactiver »).
- Icônes posées sur la vue 3D : 👁 transparence, 🌈 carte de l'induction,
  🌀 lignes de flux, ➡ vecteurs B.
- Bandeau d'accès direct dans la Synthèse.
- Badge « NOUVEAU » clignotant sur le bouton et l'onglet Moteur 3D jusqu'au
  premier essai (mémorisé dans les réglages).

*English — The 3D field view is now prominent: purple “👁 3D field” button
in the main bar, large button at the top of the 3D motor tab, icons on the
3D view, Summary banner, blinking “NEW” badge until first use.*

## 0.1.5 — vue transparente et champ magnétique en 3D

- **Moteur 3D** : « Vue transparente » avec opacité réglable ; les pièces du
  moteur passent en teintes neutres quand le champ est affiché, pour que
  seul le champ porte la couleur.
- **Champ magnétique en 3D**, à la position courante du rotor (il suit
  l'animation) :
  - flux radial : carte de |B| et lignes de flux dans le plan médian de la
    machine (sur la face avant sans transparence), vecteurs B dans
    l'entrefer ;
  - flux axial : induction axiale Bz au milieu de chaque entrefer, une
    couronne par tranche, et flèches le long de l'axe.
- Légende des couleurs du champ dans la vue 3D ; aide et bulles d'aide en
  4 langues.
- Rendu 3D : les faces transparentes gardent leur teinte (ombrage sur les
  couleurs non prémultipliées).
- Option de lancement `FRACTLAB_TRANSPARENT=1` (ou `tout` pour ajouter les
  vecteurs).

*English — 3D motor: transparent view with adjustable opacity; magnetic
field inside the machine at the current rotor position (|B| map, flux
lines and B vectors in the mid plane for radial machines; axial Bz map and
arrows in each air gap for axial machines).*

## 0.1.4 — machines à flux axial complètes

### Affichage
- **Onglet Champ** (flux axial) :
  - sélecteur de **tranche** (rayon moyen et largeur radiale) ;
  - courbes d'entrefer : induction **axiale** Bz et tangentielle Bθ de la
    tranche choisie, spectre ;
  - **carte du champ de la tranche développée** : circonférence à l'échelle
    réelle (r0·θ) en abscisse, cote axiale en ordonnée, culasses, encoches,
    entrefer et aimants, lignes de flux, lecture exacte au survol ; molette
    pour zoomer, glisser pour parcourir le tour ;
  - nouvelle vue **« Entrefer sur tout le disque »** : induction axiale au
    milieu de l'entrefer sur toute la face, une couronne par tranche, pôles
    N et S en couleurs.
- **Onglet Forces** (flux axial) : la force affichée est la **force
  transverse** (dans le plan du disque) ; nouveaux **défauts d'alignement**
  — décalage axial et inclinaison du rotor — avec attraction d'un disque,
  force axiale nette, raideur axiale, moment de basculement et raideur
  angulaire.
- Tableau de bord, bandeau des critères et étude paramétrique : « force
  transverse » au lieu de « force radiale » pour une machine axiale.
- Cartes 3D : la carte d'induction d'une machine axiale est en Bz.

### Calcul
- **Nombre d'harmoniques adapté à la géométrie** (au moins deux fois le
  rapport circonférence / ouverture d'encoche, tronçon par tronçon) : le
  couple de détente d'une grande machine axiale à ouvertures étroites était
  faux d'un facteur 2,5 avec la précision par défaut. Le nombre utilisé est
  affiché dans « Précision du calcul ». Les petites machines radiales ne
  changent pas.
- **Validation des machines axiales** : le solveur volumes finis traite
  maintenant une tranche plane (circonférence développée). Écarts : 1,07 %
  (à vide) et 0,42 % (en charge) sur des tranches de référence, 0,09 % sur
  le champ ; l'onglet Validation valide la tranche de référence du projet.
- Solveur volumes finis : plus de valeur invalide (NaN) quand une itération
  converge exactement.
- Construction du modèle axial en parallèle (une tranche par cœur) et
  algèbre linéaire vectorisée. Le calcul d'une grande machine axiale reste
  plus long qu'avant (26 s au lieu de 17 s sur une machine de 24 encoches à
  5 tranches) parce qu'il est maintenant convergé (1 245 harmoniques au
  lieu de 400) ; sans ces optimisations, il prendrait environ 40 s.

*English — Axial-flux machines: slice selector, axial Bz curves, unrolled
slice field map, whole-disc air-gap map; transverse force and rotor
misalignment (axial offset, tilt) with stiffnesses; harmonic count adapted
to the geometry (cogging of large axial machines with narrow openings was
off by a factor 2.5; such machines now take longer to compute because the
result is converged); finite-volume validation of planar slices (1.07 % /
0.42 %).*

## 0.1.3 — machines à flux axial dessinées en disques

- Onglet Géométrie : vue de face le long de l'axe (stator et aimants du
  rotor superposés, dents trapézoïdales) et coupe axiale (empilement des
  disques et leurs épaisseurs).
- Moteur 3D : disques empilés le long de l'axe, stator au centre.
- Induction des dents d'une machine axiale corrigée (23,5 T au lieu de
  1,7 T) : pertes fer et températures réalistes.
- Force transverse et attraction axiale séparées ; l'attraction ne charge
  plus les roulements comme une force radiale.
- Option de lancement `FRACTLAB_PROJET=fichier.json`.

*English — Axial machines drawn as discs (front view and axial section,
stacked-disc 3D); tooth flux density, iron losses and temperatures fixed;
transverse force and axial attraction separated.*

## 0.1.2 — correctif Windows

- L'interface tremblait puis l'application pouvait s'arrêter sous Windows
  avec une mise à l'échelle de 150 % : le zoom automatique oscillait. Il
  est calculé une seule fois, hors zoom de l'application.
- Cause d'un plantage ou d'un échec au démarrage écrite dans `crash.txt`
  (dossier des réglages).

*English — Windows fix: flickering interface with 150 % display scaling;
crash causes written to crash.txt.*

## 0.1.1 — version navigateur

- FractLab dans le navigateur (Chrome, Edge, Firefox) : un seul fichier
  HTML, hors ligne, ou en ligne sur GitHub Pages ; calculs dans des Web
  Workers ; fichiers téléchargés (plusieurs dans une archive ZIP).
- Couche système commune (fichiers, son, PDF, impression, réglages).

*English — Browser version (single HTML file, Web Workers).*

## 0.1.0 — première version publique

Modèle analytique en sous-domaines pour toute répartition d'aimants (rotor
extérieur, rotor intérieur, flux axial multi-tranches), charge,
inductances, saturation, simulation dynamique, moteur 3D animé, Ajustement
I.A, étude paramétrique, recalage sur mesures, validation par volumes
finis, projets SIMUMSAP et fichiers MATLAB sans MATLAB, rédaction LaTeX /
PDF en 4 langues, interface en français, anglais, espagnol et allemand.

*English — First public release.*
