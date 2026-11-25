# Modèle physique de FractLab

## Topologies

| Topologie | Géométrie de calcul | Coordonnées |
|---|---|---|
| Flux radial, rotor extérieur | une section 2D (stator encoché à l'intérieur, entrefer, aimants, cloche) | polaires (r, θ) |
| Flux radial, rotor intérieur | une section 2D (stator à l'extérieur) | polaires (r, θ) |
| Flux axial | N **tranches** annulaires (technique multi-tranches de SIMUMSAP), chacune développée en une section plane au rayon moyen r0 | planes (y = cote axiale, x = r0·θ) |

En flux axial, chaque tranche a sa largeur radiale, son rayon moyen et son
arc d'aimant (interpolé entre le rayon intérieur et le rayon extérieur).
Avec deux entrefers, le stator est au centre et un disque rotor de chaque
côté : les grandeurs des deux côtés identiques s'additionnent (couple, flux).
L'empilement axial (culasses, dents, entrefers, aimants) est donné par
`MotorSpec::axial_stack` ; le dessin, le moteur 3D et l'export STL
l'utilisent tous.

## Équations

Magnétostatique 2D, potentiel vecteur `A_z`, `B = rot A` :

- encoches, isthmes (becs de dents) et entrefer : `∇²A = 0` (plus `−μ0 J`
  dans les encoches parcourues par un courant) ;
- aimants : terme source de l'aimantation (radiale/axiale, parallèle ou
  Halbach).

L'aimantation est développée en série de Fourier sur **360° complets**
(rangs n = 1…N), sans hypothèse de périodicité : chaque aimant contribue
selon sa position réelle. C'est ce qui permet d'étudier les répartitions apériodiques.

Solution par sous-domaines (séparation des variables) : entrefer et
aimants par harmonique (élimination 4×4), encoches et isthmes par leurs
fonctions propres. Le système des encoches ne dépend pas de la position du
rotor : il est factorisé une seule fois (LU), puis chaque position ne coûte
qu'une descente-remontée. Conditions : `Hθ = 0` au contact du fer
(perméabilité infinie), continuité de A et de Hθ aux interfaces.

### Nombre d'harmoniques adapté à la géométrie

La précision dépend du rapport entre la circonférence et l'ouverture
d'encoche. FractLab utilise au moins **deux fois** ce rapport pour chaque
tronçon (plafond 2400), même si la valeur choisie dans « Précision du
calcul » est plus faible ; le nombre effectivement utilisé y est affiché.
Sans cette règle, le couple de détente d'une grande machine axiale à
ouvertures étroites (r0 ≈ 157 mm, ouverture 1,8 mm) était faux d'un facteur
2,5 avec 400 harmoniques ; les petites machines radiales ne sont pas
concernées.

## Grandeurs dérivées

- **Couple** : tenseur de Maxwell au milieu de l'entrefer.
- **Force sur le rotor** :
  - radial : force résultante (contraintes normales et tangentielles) ;
  - axial : **force transverse** dans le plan du disque (somme des
    cisaillements `Bn·Bt/μ0` projetés), nulle par symétrie pour un rotor
    régulier ; l'**attraction axiale** d'un disque est calculée à part.
- **Flux de dent** : différence des potentiels moyens des demi-encoches
  voisines ; FEM par dérivée sur un tour mécanique ; bobinage général
  (bobine par bobine).
- **Charge** : courants sinusoïdaux alignés sur la FEM ; contrôle
  énergétique `Σ i·dλ/dθ + couple de détente`.
- **Inductances** : flux de réaction d'induit (hors têtes de bobines) plus
  une inductance de têtes de bobines `μ0·N²·l_têtes·λ` (λ réglable, 0 comme
  SIMUMSAP).
- **Saturation** : méthode du facteur de saturation (circuit magnétique
  équivalent, courbes B(H) réelles). En flux axial, l'induction des dents
  utilise la largeur réelle de la dent (pas d'encoche au rayon moyen moins
  la largeur d'encoche) et le flux d'un seul côté du stator.
- **Défauts d'alignement** : excentricité (radial) ; décalage axial et
  inclinaison du disque (axial), par la méthode de perméance appliquée
  tranche par tranche (`Bn / (1 ± δ/g')`, pression `Bn²/2μ0`).

## Validation

`fdcheck.rs` résout le même problème par une méthode différente : volumes
finis, fer maillé (μr = 10⁵), aimantation moyennée par cellule, couple
d'Arkkio. En géométrie plane (tranche de machine axiale), le rayon métrique
est remplacé par le rayon moyen de la tranche.

| Cas | Écart sur le couple | Écart sur le champ |
|---|---|---|
| Rotor extérieur, encoches droites (régulier, décalé, Halbach) | 1,45 à 2,04 % | — |
| Rotor extérieur, encoches semi-fermées | 0,78 % | 0,06 % |
| Rotor intérieur, encoches semi-fermées | 1,08 % | 0,05 % |
| Aimants + courants (en charge) | 0,30 % | 0,05 % |
| Tranche plane de machine axiale (à vide) | 1,07 % | 0,09 % |
| Tranche plane de machine axiale (en charge) | 0,42 % | 0,08 % |

## Limites

- éléments finis non linéaires non couverts (la saturation est estimée par
  le facteur de saturation) ;
- effets d'extrémité 3D d'une machine radiale négligés (KV estimé) ;
- simulation dynamique à inductance constante ;
- machines asynchrones non couvertes.

## Point d'attention : chevauchement des aimants

Quand les aimants sont décalés de leur position régulière, un arc trop grand
fait se chevaucher des aimants voisins. Le modèle additionne alors les
aimantations dans la zone commune : les résultats n'y sont pas physiques.
FractLab signale le cas et propose l'arc maximal admissible.
