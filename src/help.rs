//! Guide d'utilisation intégré (menu Aide), en français, anglais, espagnol et
//! allemand. Chaque section a un titre et un corps par langue, dans l'ordre
//! [FR, EN, ES, DE]. Les lignes commençant par « • » sont des puces ; une
//! ligne terminée par « : » est un intertitre.

use crate::i18n::Lang;

/// Section du guide d'utilisation.
pub struct Section {
    /// Icône affichée dans le sommaire.
    pub icon: &'static str,
    /// Titre dans chaque langue [FR, EN, ES, DE].
    pub title: [&'static str; 4],
    /// Corps dans chaque langue [FR, EN, ES, DE].
    pub body: [&'static str; 4],
}

impl Section {
    /// Titre dans la langue `l`.
    pub fn title(&self, l: Lang) -> &'static str {
        self.title[l as usize]
    }
    /// Corps dans la langue `l`.
    pub fn body(&self, l: Lang) -> &'static str {
        self.body[l as usize]
    }
}

/// Sections du guide (publiques puis celles d'un module privé éventuel).
pub fn sections() -> Vec<&'static Section> {
    SECTIONS.iter().chain(crate::repartition::extra_help()).collect()
}

/// Sections du guide, dans l'ordre du sommaire.
pub static SECTIONS: &[Section] = &[
    Section {
        icon: "▶",
        title: ["Démarrage rapide", "Quick start", "Inicio rápido", "Schnellstart"],
        body: [
"FractLab calcule le champ magnétique, le couple de détente (cogging), la force électromotrice (FEM) et les forces radiales d'une machine à aimants permanents (rotor extérieur ou intérieur, flux radial ou axial), pour n'importe quelle répartition des aimants, régulière ou non.
En trois gestes :
• Réglez les dimensions dans le panneau de gauche, ou partez d'un moteur de la démo (menu Aide).
• Le calcul se relance seul environ un tiers de seconde après chaque modification (case « auto »). Le bouton « Calcul » force un calcul immédiat.
• Lisez la Synthèse : chaque carte affiche une grandeur, son seuil quand il y en a un (✔ conforme, ✖ hors seuil) et s'ouvre en plein écran d'un clic.
Le projet courant (toutes les données) s'enregistre et se rouvre par le menu Fichier ; le dernier projet est rouvert au démarrage.
Aide au passage de la souris : laissez la souris un instant sur un réglage, un onglet ou un bouton pour voir à quoi il sert.
Version navigateur (fichier FractLab.html ou en ligne) : mêmes fonctions ; les calculs tournent en arrière-plan et sont plus lents qu'en version de bureau ; les fichiers enregistrés sont téléchargés (plusieurs fichiers dans une archive ZIP), l'impression passe par l'aperçu PDF du navigateur.",
"FractLab computes the magnetic field, cogging torque, back-EMF and radial forces of a permanent-magnet machine (outer or inner rotor, radial or axial flux), for any magnet layout, regular or not.
In three steps:
• Set the dimensions in the left panel, or start from a demo motor (Help menu).
• The computation restarts by itself about a third of a second after each change (“auto” box). The “Compute” button forces an immediate run.
• Read the Summary: each card shows a quantity, its limit when there is one (✔ met, ✖ failed) and opens full screen with one click.
The current project (all data) is saved and reopened from the File menu; the last project is reopened at start-up.
Hover help: rest the mouse on a setting, tab or button for a moment to see what it does.
Browser version (FractLab.html file or online): same features; calculations run in the background and are slower than in the desktop version; saved files are downloaded (several files as a ZIP archive), printing goes through the browser's PDF preview.",
"FractLab calcula el campo magnético, el par de cogging, la fuerza contraelectromotriz (FCEM) y las fuerzas radiales de una máquina de imanes permanentes (rotor exterior o interior, flujo radial o axial), para cualquier distribución de imanes, regular o no.
En tres pasos:
• Ajuste las dimensiones en el panel izquierdo, o parta de un motor de la demo (menú Ayuda).
• El cálculo se relanza solo un tercio de segundo después de cada cambio (casilla «auto»). El botón «Cálculo» fuerza un cálculo inmediato.
• Lea el Resumen: cada tarjeta muestra una magnitud, su umbral cuando lo hay (✔ cumple, ✖ fuera de umbral) y se abre en pantalla completa con un clic.
El proyecto actual (todos los datos) se guarda y se abre desde el menú Archivo; el último proyecto se reabre al iniciar.
Ayuda al pasar el ratón: deje el ratón un instante sobre un ajuste, una pestaña o un botón para ver para qué sirve.
Versión de navegador (archivo FractLab.html o en línea): mismas funciones; los cálculos se ejecutan en segundo plano y son más lentos que en la versión de escritorio; los archivos guardados se descargan (varios archivos en un ZIP), la impresión pasa por la vista previa PDF del navegador.",
"FractLab berechnet Magnetfeld, Rastmoment, Gegen-EMK und Radialkräfte einer Permanentmagnetmaschine (Außen- oder Innenläufer, Radial- oder Axialfluss) für jede beliebige Magnetverteilung, regelmäßig oder nicht.
In drei Schritten:
• Stellen Sie die Abmessungen links ein oder starten Sie mit einem Demo-Motor (Menü Hilfe).
• Die Berechnung startet etwa eine Drittelsekunde nach jeder Änderung automatisch neu (Kästchen „auto“). „Berechnung“ erzwingt einen sofortigen Lauf.
• Lesen Sie die Übersicht: Jede Karte zeigt eine Größe, ihre Grenze, sofern vorhanden (✔ erfüllt, ✖ verfehlt) und öffnet sich per Klick im Vollbild.
Das aktuelle Projekt (alle Daten) wird über das Menü Datei gespeichert und geöffnet; das letzte Projekt wird beim Start wieder geladen.
Hilfe beim Überfahren: die Maus kurz auf einer Einstellung, einem Reiter oder einer Schaltfläche ruhen lassen, um ihre Funktion zu sehen.
Browser-Version (Datei FractLab.html oder online): gleiche Funktionen; die Berechnungen laufen im Hintergrund und sind langsamer als in der Desktop-Version; gespeicherte Dateien werden heruntergeladen (mehrere Dateien als ZIP-Archiv), gedruckt wird über die PDF-Vorschau des Browsers.",
        ],
    },
    Section {
        icon: "🧭",
        title: ["Espace de travail et gestion du projet", "Workspace and project management", "Espacio de trabajo y gestión del proyecto", "Arbeitsbereich und Projektverwaltung"],
        body: [
"Les vues sont rangées par étape d'étude, dans la barre du haut : Conception (Synthèse, Géométrie, Moteur 3D), Analyse (Champ, Cogging, FEM, Forces, Charge, Dynamique, Cartes 3D), Optimisation (Ajustement I.A, Console, Comparaison des solutions, Paramétrique), Validation. Chaque étape retient le dernier onglet ouvert.
Projet :
• Annuler / rétablir : boutons ⟲ ⟳, menu Édition, Ctrl+Z et Ctrl+Y (ou Ctrl+Maj+Z). Les saisies rapprochées comptent pour une seule annulation.
• « ● modifié » sous le titre (et dans le titre de la fenêtre) : le projet diffère du fichier enregistré ; un clic liste les modifications (valeur enregistrée → valeur actuelle). Même liste par Édition > Modifications depuis l'enregistrement.
• Fichier > Projets récents : les 8 derniers projets ouverts ou enregistrés.
• Loupe en haut du panneau de gauche : tapez une partie du nom d'un paramètre ; les champs trouvés sont encadrés en jaune, la vue va au premier et les sections se déplient.
Jauge de qualité de conception (Synthèse, à côté du titre) : critères tenus sur critères évalués ; le survol détaille ceux qui ne le sont pas.
Au premier lancement, l'accueil présente les étapes et propose la démo guidée.
L'interface s'adapte à l'écran : sur un portable, la comparaison des solutions place le tableau au-dessus des graphiques ; sur un écran 4K, le zoom monte automatiquement (réglable dans les Préférences).",
"Views are organised by study step in the top bar: Design (Summary, Geometry, 3D motor), Analysis (Field, Cogging, Back-EMF, Forces, Load, Dynamics, 3D maps), Optimisation (A.I. tuning, Console, Solution comparison, Parametric), Validation. Each step remembers its last open tab.
Project:
• Undo / redo: ⟲ ⟳ buttons, Edit menu, Ctrl+Z and Ctrl+Y (or Ctrl+Shift+Z). Quick successive edits count as a single undo.
• “● modified” under the title (and in the window title): the project differs from the saved file; a click lists the changes (saved value → current value). Same list via Edit > Changes since last save.
• File > Recent projects: the last 8 projects opened or saved.
• Magnifier at the top of the left panel: type part of a parameter name; matching fields are framed in yellow, the view jumps to the first one and sections expand.
Design-quality gauge (Summary, next to the title): criteria met over criteria evaluated; hovering details those that are not met.
At first launch, the welcome window presents the steps and offers the guided demo.
The interface adapts to the screen: on a laptop, the solution comparison puts the table above the charts; on a 4K screen, the zoom rises automatically (adjustable in Preferences).",
"Las vistas se ordenan por etapa del estudio en la barra superior: Diseño (Resumen, Geometría, Motor 3D), Análisis (Campo, Cogging, FCEM, Fuerzas, Carga, Dinámica, Mapas 3D), Optimización (Ajuste I.A, Consola, Comparación de las soluciones, Paramétrico), Validación. Cada etapa recuerda la última pestaña abierta.
Proyecto:
• Deshacer / rehacer: botones ⟲ ⟳, menú Edición, Ctrl+Z y Ctrl+Y (o Ctrl+Mayús+Z). Las ediciones seguidas cuentan como una sola.
• «● modificado» bajo el título (y en el título de la ventana): el proyecto difiere del archivo guardado; un clic lista los cambios (valor guardado → valor actual). Misma lista en Edición > Cambios desde el guardado.
• Archivo > Proyectos recientes: los 8 últimos proyectos abiertos o guardados.
• Lupa arriba del panel izquierdo: escriba parte del nombre de un parámetro; los campos encontrados se enmarcan en amarillo, la vista va al primero y las secciones se despliegan.
Indicador de calidad del diseño (Resumen, junto al título): criterios cumplidos sobre criterios evaluados; al pasar el ratón se detallan los no cumplidos.
En el primer inicio, la bienvenida presenta las etapas y propone la demo guiada.
La interfaz se adapta a la pantalla: en un portátil, la comparación de soluciones pone la tabla encima de los gráficos; en una pantalla 4K, el zoom sube automáticamente (ajustable en Preferencias).",
"Die Ansichten sind in der oberen Leiste nach Studienschritten geordnet: Entwurf (Übersicht, Geometrie, 3D-Motor), Analyse (Feld, Rastmoment, EMK, Kräfte, Last, Dynamik, 3D-Karten), Optimierung (KI-Optimierung, Konsole, Lösungsvergleich, Parameterstudie), Validierung. Jeder Schritt merkt sich den zuletzt geöffneten Reiter.
Projekt:
• Rückgängig / Wiederholen: Schaltflächen ⟲ ⟳, Menü Bearbeiten, Strg+Z und Strg+Y (oder Strg+Umschalt+Z). Schnell aufeinanderfolgende Eingaben zählen als ein Schritt.
• „● geändert“ unter dem Titel (und im Fenstertitel): Das Projekt weicht von der gespeicherten Datei ab; ein Klick listet die Änderungen (gespeicherter Wert → aktueller Wert). Dieselbe Liste unter Bearbeiten > Änderungen seit dem Speichern.
• Datei > Zuletzt verwendete Projekte: die 8 zuletzt geöffneten oder gespeicherten Projekte.
• Lupe oben in der linken Leiste: Teil eines Parameternamens eingeben; gefundene Felder werden gelb umrahmt, die Ansicht springt zum ersten, die Abschnitte klappen auf.
Anzeige der Entwurfsqualität (Übersicht, neben dem Titel): erfüllte von bewerteten Kriterien; beim Überfahren werden die nicht erfüllten erläutert.
Beim ersten Start stellt die Begrüßung die Schritte vor und bietet die geführte Demo an.
Die Oberfläche passt sich dem Bildschirm an: auf einem Laptop steht beim Lösungsvergleich die Tabelle über den Diagrammen; auf einem 4K-Bildschirm steigt der Zoom automatisch (in den Einstellungen änderbar).",
        ],
    },
    Section {
        icon: "▶",
        title: ["Démo", "Demo", "Demo", "Demo"],
        body: [
"Menu Aide > Démo : une étude complète se déroule toute seule sur un moteur de démonstration.
Lancement :
• Attention : la démo remplace les données en cours par le moteur de démonstration. Une fenêtre le rappelle et demande confirmation ; « Enregistrer d'abord » garde votre projet dans un fichier.
• Votre projet est aussi mis de côté : à la fin, « Revenir à mon projet » le rétablit.
• Choisissez le moteur : drone 9N12P (rotor extérieur), servomoteur 12N8P (rotor intérieur, becs de dents), moteur-roue de vélo 36N42P, génératrice à flux axial 18N16P.
Déroulement :
• Le paramétrage défile dans l'ordre d'une étude : type de machine, encoches et pôles, stator, entrefer, aimants, bobinage, charge. Le champ en cours de saisie est encadré en rouge et sa valeur se tape chiffre par chiffre.
• Puis le calcul, et les onglets un à un : Synthèse, Géométrie, Champ, Cogging, FEM, Forces, Charge, le moteur 3D avec le champ magnétique, et un Ajustement I.A avec sa console et le graphe des possibilités explorées.
• Une légende en bas de l'écran explique chaque étape.
Arrêt : « ÉCHAP — sortir de la DÉMO » clignote en rouge en haut de l'écran ; la touche Échap arrête la démo à tout moment.",
"Help menu > Demo: a complete study runs by itself on a demonstration motor.
Start:
• Warning: the demo replaces the current data with the demonstration motor. A window says so and asks for confirmation; “Save first” keeps your project in a file.
• Your project is also set aside: at the end, “Back to my project” restores it.
• Pick the motor: drone 9N12P (outer rotor), servo motor 12N8P (inner rotor, tooth tips), bicycle hub motor 36N42P, axial-flux generator 18N16P.
Sequence:
• The settings scroll by in the order of a study: machine type, slots and poles, stator, air gap, magnets, winding, load. The field being entered is framed in red and its value is typed digit by digit.
• Then the computation, and the tabs one by one: Summary, Geometry, Field, Cogging, Back-EMF, Forces, Load, the 3D motor with the magnetic field, and an A.I. tuning with its console and the graph of explored possibilities.
• A caption at the bottom of the screen explains each step.
Stop: “ESC — exit the DEMO” blinks red at the top of the screen; the Esc key stops the demo at any time.",
"Menú Ayuda > Demo: un estudio completo se desarrolla solo con un motor de demostración.
Inicio:
• Atención: la demo sustituye los datos actuales por el motor de demostración. Una ventana lo recuerda y pide confirmación; «Guardar antes» conserva su proyecto en un archivo.
• Su proyecto también queda apartado: al final, «Volver a mi proyecto» lo restablece.
• Elija el motor: dron 9N12P (rotor exterior), servomotor 12N8P (rotor interior, picos de diente), motor de rueda de bicicleta 36N42P, generador de flujo axial 18N16P.
Desarrollo:
• Los ajustes desfilan en el orden de un estudio: tipo de máquina, ranuras y polos, estátor, entrehierro, imanes, devanado, carga. El campo en curso se enmarca en rojo y su valor se teclea cifra a cifra.
• Luego el cálculo y las pestañas una a una: Resumen, Geometría, Campo, Cogging, FCEM, Fuerzas, Carga, el motor 3D con el campo magnético y un Ajuste I.A con su consola y el grafo de posibilidades exploradas.
• Una leyenda en la parte inferior explica cada paso.
Parada: «ESC — salir de la DEMO» parpadea en rojo arriba de la pantalla; la tecla Esc detiene la demo en cualquier momento.",
"Menü Hilfe > Demo: Eine vollständige Studie läuft selbstständig mit einem Demo-Motor ab.
Start:
• Achtung: Die Demo ersetzt die aktuellen Daten durch den Demo-Motor. Ein Fenster weist darauf hin und fragt nach Bestätigung; „Zuerst speichern“ sichert Ihr Projekt in einer Datei.
• Ihr Projekt wird außerdem beiseitegelegt: Am Ende stellt „Zurück zu meinem Projekt“ es wieder her.
• Motor wählen: Drohne 9N12P (Außenläufer), Servomotor 12N8P (Innenläufer, Zahnköpfe), Fahrrad-Nabenmotor 36N42P, Axialfluss-Generator 18N16P.
Ablauf:
• Die Einstellungen laufen in der Reihenfolge einer Studie durch: Maschinentyp, Nuten und Pole, Stator, Luftspalt, Magnete, Wicklung, Last. Das gerade bearbeitete Feld ist rot umrahmt, sein Wert wird Ziffer für Ziffer eingetippt.
• Dann die Berechnung und die Reiter nacheinander: Übersicht, Geometrie, Feld, Rastmoment, EMK, Kräfte, Last, der 3D-Motor mit dem Magnetfeld und eine K.I.-Abstimmung mit Konsole und Graph der erkundeten Möglichkeiten.
• Eine Bildunterschrift unten erklärt jeden Schritt.
Stopp: „ESC — DEMO beenden“ blinkt rot oben am Bildschirm; die Esc-Taste beendet die Demo jederzeit.",
        ],
    },
    Section {
        icon: "☰",
        title: ["Les paramètres du moteur", "Motor parameters", "Parámetros del motor", "Motorparameter"],
        body: [
"Toutes les données se modifient dans le panneau de gauche (bouton ☰ pour le masquer sur un petit écran). Glissez une valeur avec la souris ou double-cliquez pour la saisir au clavier ; l'unité affichée suit vos préférences.
Projet et variante :
• Variante : répartition des aimants (régulière, ou positions personnalisées).
• Aimantation : radiale, parallèle ou Halbach (nombre de segments par pôle réglable).
Topologie : nombre d'encoches Q et de pôles 2p. Le bobinage concentré est déduit automatiquement de l'étoile des encoches ; il s'affiche en vert s'il est équilibré.
Stator, entrefer, aimants et rotor : rayons, épaisseurs, ouverture d'encoche, longueur active, arc d'aimant, induction rémanente Br (préréglages N35 à N52) et perméabilité de recul.
Répartition des aimants :
• Balourd, autocorrélation et écart minimal entre aimants, recalculés en direct.
• « Positions personnalisées » : éditez l'angle de chaque aimant ; « Rééquilibrer » annule le balourd par la méthode de Newton.
Précision du calcul : nombre d'harmoniques, points de balayage, préréglages Rapide / Précis.",
"All data are edited in the left panel (☰ button hides it on a small screen). Drag a value with the mouse or double-click to type it; the displayed unit follows your preferences.
Project and variant:
• Variant: magnet layout (regular, or custom positions).
• Magnetisation: radial, parallel or Halbach (adjustable number of segments per pole).
Topology: number of slots Q and poles 2p. The concentrated winding is derived automatically from the star of slots; it is shown in green when balanced.
Stator, air gap, magnets and rotor: radii, thicknesses, slot opening, active length, magnet arc, remanence Br (presets N35 to N52) and recoil permeability.
Magnet layout:
• Unbalance, autocorrelation and minimum gap between magnets, updated live.
• “Custom positions”: edit the angle of every magnet; “Rebalance” cancels the unbalance with Newton's method.
Computation accuracy: number of harmonics, sweep points, Fast / Accurate presets.",
"Todos los datos se modifican en el panel izquierdo (botón ☰ para ocultarlo en una pantalla pequeña). Arrastre un valor con el ratón o haga doble clic para escribirlo; la unidad mostrada sigue sus preferencias.
Proyecto y variante:
• Variante: distribución de los imanes (regular, o posiciones personalizadas).
• Magnetización: radial, paralela o Halbach (número de segmentos por polo ajustable).
Topología: número de ranuras Q y de polos 2p. El devanado concentrado se deduce automáticamente de la estrella de ranuras; aparece en verde si está equilibrado.
Estátor, entrehierro, imanes y rotor: radios, espesores, apertura de ranura, longitud activa, arco del imán, inducción remanente Br (preajustes N35 a N52) y permeabilidad de retroceso.
Distribución de los imanes:
• Desequilibrio, autocorrelación y separación mínima entre imanes, actualizados en directo.
• «Posiciones personalizadas»: edite el ángulo de cada imán; «Reequilibrar» anula el desequilibrio con el método de Newton.
Precisión del cálculo: número de armónicos, puntos de barrido, preajustes Rápido / Preciso.",
"Alle Daten werden in der linken Leiste bearbeitet (Schaltfläche ☰ blendet sie auf kleinen Bildschirmen aus). Ziehen Sie einen Wert mit der Maus oder doppelklicken Sie zur Eingabe; die angezeigte Einheit folgt Ihren Einstellungen.
Projekt und Variante:
• Variante: Magnetanordnung (regelmäßig oder eigene Positionen).
• Magnetisierung: radial, parallel oder Halbach (Segmente pro Pol einstellbar).
Topologie: Nutenzahl Q und Polzahl 2p. Die Zahnspulenwicklung wird automatisch aus dem Nutenstern bestimmt; grün angezeigt, wenn sie symmetrisch ist.
Stator, Luftspalt, Magnete und Rotor: Radien, Dicken, Nutöffnung, aktive Länge, Magnetbogen, Remanenz Br (Voreinstellungen N35 bis N52) und Permeabilität.
Magnetanordnung:
• Unwucht, Autokorrelation und minimaler Magnetabstand, live aktualisiert.
• „Eigene Positionen“: Winkel jedes Magneten bearbeiten; „Auswuchten“ hebt die Unwucht mit dem Newton-Verfahren auf.
Rechengenauigkeit: Anzahl der Harmonischen, Abtastpunkte, Voreinstellungen Schnell / Genau.",
        ],
    },
    Section {
        icon: "⚠",
        title: ["Faisabilité : chevauchement des aimants", "Feasibility: overlapping magnets", "Viabilidad: imanes solapados", "Machbarkeit: überlappende Magnete"],
        body: [
"Décaler les aimants réduit l'espace libre entre deux aimants voisins. Si un décalage dépasse cet espace, deux aimants occupent la même place : la géométrie est irréalisable et les résultats ne sont plus physiques dans la zone concernée.
FractLab le contrôle en permanence :
• « Écart min entre aimants » (panneau de gauche) et la carte « Faisabilité des aimants » (Synthèse) passent au rouge dès qu'un écart est négatif.
• Le bouton « Ajuster l'arc » réduit l'arc des aimants au maximum admissible, avec 0,1° de marge.
Exemple : un arc d'aimant trop grand combiné à des décalages de quelques degrés suffit à faire se toucher deux aimants ; réduire l'arc de quelques millièmes rend la géométrie réalisable.",
"Shifting magnets reduces the free space between neighbours. When an offset exceeds that space, two magnets occupy the same place: the geometry cannot be built and results are no longer physical in that area.
FractLab checks this continuously:
• “Min gap between magnets” (left panel) and the “Magnet feasibility” card (Summary) turn red as soon as a gap is negative.
• The “Fit the arc” button reduces the magnet arc to the largest admissible value, with a 0.1° margin.
Example: a large magnet arc combined with offsets of a few degrees is enough to make two magnets touch; reducing the arc by a few thousandths makes the geometry feasible.",
"Desplazar los imanes reduce el espacio libre entre vecinos. Si un desfase supera ese espacio, dos imanes ocupan el mismo lugar: la geometría es irrealizable y los resultados dejan de ser físicos en esa zona.
FractLab lo comprueba permanentemente:
• «Separación mín. entre imanes» (panel izquierdo) y la tarjeta «Viabilidad de los imanes» (Resumen) se ponen en rojo en cuanto una separación es negativa.
• El botón «Ajustar el arco» reduce el arco de los imanes al máximo admisible, con 0,1° de margen.
Ejemplo: un arco de imán grande combinado con desfases de pocos grados basta para que dos imanes se toquen; reducir el arco unas milésimas hace realizable la geometría.",
"Das Versetzen der Magnete verkleinert den freien Raum zwischen Nachbarn. Übersteigt ein Versatz diesen Raum, belegen zwei Magnete denselben Platz: Die Geometrie ist nicht baubar und die Ergebnisse sind dort nicht mehr physikalisch.
FractLab prüft dies laufend:
• „Min. Abstand zwischen Magneten“ (linke Leiste) und die Karte „Machbarkeit der Magnete“ (Übersicht) werden rot, sobald ein Abstand negativ ist.
• Die Schaltfläche „Bogen anpassen“ verkleinert den Magnetbogen auf den größten zulässigen Wert mit 0,1° Rand.
Beispiel: Ein großer Magnetbogen zusammen mit Versätzen von wenigen Grad genügt, damit sich zwei Magnete berühren; den Bogen um einige Tausendstel zu verkleinern macht die Geometrie baubar.",
        ],
    },
    Section {
        icon: "📋",
        title: ["Synthèse, vignettes et plein écran", "Summary, thumbnails and full screen", "Resumen, miniaturas y pantalla completa", "Übersicht, Miniaturen und Vollbild"],
        body: [
"La Synthèse est le tableau de bord du projet.
• Vignettes : un aperçu de chaque vue. Une vignette à cadre orange qui pulse, avec la mention « nouveau », signale un résultat neuf que vous n'avez pas encore ouvert ; une pastille orange apparaît aussi sur l'onglet correspondant.
• Un clic sur une vignette ou une carte ouvre la vue en plein écran ; le panneau des paramètres se masque pour laisser toute la place. Échap ou « Retour » ferme le plein écran.
• Sur un écran étroit, les cartes passent automatiquement en format compact ; le détail reste visible au survol.",
"The Summary is the project dashboard.
• Thumbnails: a preview of each view. A thumbnail with a pulsing orange frame and the word “new” marks a fresh result you have not opened yet; an orange dot also appears on the matching tab.
• Clicking a thumbnail or a card opens the view full screen; the parameter panel hides to give it all the room. Esc or “Back” closes full screen.
• On a narrow screen, cards switch to a compact format automatically; details remain available on hover.",
"El Resumen es el panel de control del proyecto.
• Miniaturas: una vista previa de cada vista. Una miniatura con marco naranja que late y la palabra «nuevo» indica un resultado reciente que aún no ha abierto; también aparece un punto naranja en la pestaña correspondiente.
• Un clic en una miniatura o tarjeta abre la vista en pantalla completa; el panel de parámetros se oculta para dejarle todo el espacio. Esc o «Volver» cierra la pantalla completa.
• En una pantalla estrecha, las tarjetas pasan automáticamente a formato compacto; el detalle sigue visible al pasar el ratón.",
"Die Übersicht ist das Dashboard des Projekts.
• Miniaturen: eine Vorschau jeder Ansicht. Eine Miniatur mit pulsierendem orangem Rahmen und dem Wort „neu“ kennzeichnet ein neues, noch nicht geöffnetes Ergebnis; auch am zugehörigen Reiter erscheint ein oranger Punkt.
• Ein Klick auf Miniatur oder Karte öffnet die Ansicht im Vollbild; die Parameterleiste wird ausgeblendet. Esc oder „Zurück“ schließt das Vollbild.
• Auf schmalen Bildschirmen wechseln die Karten automatisch ins Kompaktformat; Details bleiben beim Überfahren sichtbar.",
        ],
    },
    Section {
        icon: "〰",
        title: ["Courbes et mesures au curseur", "Curves and cursor measurements", "Curvas y medidas con cursor", "Kurven und Cursormessungen"],
        body: [
"Tous les graphes (champ, cogging, FEM, forces, comparaison, validation) partagent les mêmes outils.
• Survol : valeurs exactes x et y sous le pointeur.
• Clic gauche : pose le curseur A. Clic droit : pose le curseur B.
• Sous le graphe, un tableau donne pour chaque courbe la valeur en A, en B, leur écart Δ (et Δx), ainsi que le minimum, le maximum, la valeur crête-à-crête, la moyenne et la valeur efficace (RMS).
• Glisser : déplacer ; Ctrl + molette : zoomer ; double-clic : vue initiale.
• Les spectres peuvent s'afficher en amplitude ou en dB (case « spectre en dB »).",
"All charts (field, cogging, back-EMF, forces, comparison, validation) share the same tools.
• Hover: exact x and y values under the pointer.
• Left click: places cursor A. Right click: places cursor B.
• Below the chart, a table gives for each curve the value at A, at B, their difference Δ (and Δx), plus minimum, maximum, peak-to-peak, mean and RMS.
• Drag: pan; Ctrl + wheel: zoom; double-click: reset view.
• Spectra can be shown in amplitude or in dB (“spectrum in dB” box).",
"Todos los gráficos (campo, cogging, FCEM, fuerzas, comparación, validación) comparten las mismas herramientas.
• Al pasar el ratón: valores exactos x e y bajo el puntero.
• Clic izquierdo: coloca el cursor A. Clic derecho: coloca el cursor B.
• Bajo el gráfico, una tabla da para cada curva el valor en A, en B, su diferencia Δ (y Δx), así como mínimo, máximo, pico a pico, media y valor eficaz (RMS).
• Arrastrar: desplazar; Ctrl + rueda: zoom; doble clic: vista inicial.
• Los espectros pueden mostrarse en amplitud o en dB (casilla «espectro en dB»).",
"Alle Diagramme (Feld, Rastmoment, Gegen-EMK, Kräfte, Vergleich, Validierung) teilen dieselben Werkzeuge.
• Überfahren: exakte x- und y-Werte unter dem Zeiger.
• Linksklick: setzt Cursor A. Rechtsklick: setzt Cursor B.
• Unter dem Diagramm zeigt eine Tabelle je Kurve den Wert bei A, bei B, ihre Differenz Δ (und Δx) sowie Minimum, Maximum, Spitze-Spitze, Mittelwert und Effektivwert (RMS).
• Ziehen: verschieben; Strg + Mausrad: zoomen; Doppelklick: Ausgangsansicht.
• Spektren lassen sich als Amplitude oder in dB anzeigen (Kästchen „Spektrum in dB“).",
        ],
    },
    Section {
        icon: "◎",
        title: ["Géométrie, champ et moteur 3D", "Geometry, field and 3D motor", "Geometría, campo y motor 3D", "Geometrie, Feld und 3D-Motor"],
        body: [
"Le curseur « Position du rotor θr » fait tourner le rotor dans ces trois vues ; « Animer » le fait tourner en continu.
• Géométrie : coupe à l'échelle avec polarité (rouge N, bleu S) et sens d'aimantation (flèches). Le survol d'un aimant donne son numéro, son angle et son écart au pas régulier. À droite : les écarts de chaque aimant et les périodes de cogging.
• Champ : courbes d'induction radiale Br et tangentielle Bθ dans l'entrefer avec leur spectre, ou carte de champ (couleur = |B|, lignes = lignes de flux). Sur la carte, la molette zoome vers le pointeur : zoomez sur l'entrefer pour voir les effets d'encoche ; le survol donne r, θ, Br, Bθ, |B| et le potentiel vecteur A.
• Moteur 3D : vue en volume (cloche coupée pour voir l'intérieur), dents colorées par phase, ruban de l'induction d'entrefer. Glisser : orbite ; Maj ou clic droit + glisser : déplacer ; molette : zoom ; double-clic : vue initiale. Le survol identifie aimants, dents et bobines.",
"The “Rotor position θr” slider turns the rotor in these three views; “Animate” turns it continuously.
• Geometry: scaled cross-section with polarity (red N, blue S) and magnetisation direction (arrows). Hovering a magnet gives its number, angle and offset from the regular pitch. Right: per-magnet offsets and cogging periods.
• Field: radial Br and tangential Bθ air-gap flux density curves with their spectrum, or field map (colour = |B|, lines = flux lines). On the map, the wheel zooms towards the pointer: zoom into the air gap to see slotting effects; hovering gives r, θ, Br, Bθ, |B| and the vector potential A.
• 3D motor: solid view (bell cut open to show the inside), teeth coloured by phase, air-gap flux density ribbon. Drag: orbit; Shift or right-click + drag: pan; wheel: zoom; double-click: reset view. Hovering identifies magnets, teeth and coils.",
"El cursor «Posición del rotor θr» hace girar el rotor en estas tres vistas; «Animar» lo hace girar de forma continua.
• Geometría: sección a escala con polaridad (rojo N, azul S) y sentido de magnetización (flechas). Al pasar sobre un imán se ven su número, su ángulo y su desfase respecto al paso regular. A la derecha: desfases de cada imán y periodos de cogging.
• Campo: curvas de inducción radial Br y tangencial Bθ en el entrehierro con su espectro, o mapa de campo (color = |B|, líneas = líneas de flujo). En el mapa, la rueda hace zoom hacia el puntero: amplíe el entrehierro para ver los efectos de ranura; al pasar el ratón se obtienen r, θ, Br, Bθ, |B| y el potencial vector A.
• Motor 3D: vista en volumen (campana cortada para ver el interior), dientes coloreados por fase, cinta de inducción de entrehierro. Arrastrar: órbita; Mayús o clic derecho + arrastrar: desplazar; rueda: zoom; doble clic: vista inicial. Al pasar el ratón se identifican imanes, dientes y bobinas.",
"Der Schieberegler „Rotorposition θr“ dreht den Rotor in diesen drei Ansichten; „Animieren“ dreht ihn fortlaufend.
• Geometrie: maßstäblicher Schnitt mit Polarität (rot N, blau S) und Magnetisierungsrichtung (Pfeile). Beim Überfahren eines Magneten erscheinen Nummer, Winkel und Versatz zur regulären Teilung. Rechts: Versätze je Magnet und Rastperioden.
• Feld: Kurven der radialen Br- und tangentialen Bθ-Luftspaltinduktion mit Spektrum, oder Feldkarte (Farbe = |B|, Linien = Feldlinien). In der Karte zoomt das Mausrad zum Zeiger: in den Luftspalt zoomen, um Nuteffekte zu sehen; beim Überfahren erscheinen r, θ, Br, Bθ, |B| und das Vektorpotential A.
• 3D-Motor: Volumenansicht (Glocke aufgeschnitten), Zähne nach Phase eingefärbt, Band der Luftspaltinduktion. Ziehen: Orbit; Umschalt oder Rechtsklick + Ziehen: verschieben; Mausrad: Zoom; Doppelklick: Ausgangsansicht. Beim Überfahren werden Magnete, Zähne und Spulen erkannt.",
        ],
    },
    Section {
        icon: "💠",
        title: ["Moteur 3D animé : vitesse, pertes, chaleur, vibration et son", "Animated 3D motor: speed, losses, heat, vibration and sound", "Motor 3D animado: velocidad, pérdidas, calor, vibración y sonido", "Animierter 3D-Motor: Drehzahl, Verluste, Wärme, Schwingung und Geräusch"],
        body: [
"L'onglet Moteur 3D fait tourner le moteur à sa vitesse réelle, vue au ralenti.
Commandes :
• Lecture / Pause ; « Vitesse du moteur » : vitesse physique (10 à 60 000 tr/min) utilisée pour tous les calculs.
• « Ralenti » : facteur de ralenti de l'affichage (×1 à ×1/100 000), boutons ÷2 et ×2. La ligne voisine indique combien de millisecondes réelles s'écoulent par seconde d'écran.
• Dents colorées par leur induction instantanée, ruban de l'induction d'entrefer, vecteur de la force radiale (flèche orange).
Tableau de bord en direct :
• Instantané : position, temps réel simulé, fréquence électrique, couple de détente, FEM des trois phases, force radiale, induction maximale dans les dents.
• Pertes et frottements en régime établi : pertes fer (hystérésis et courants de Foucault, modèle de Bertotti appliqué à l'induction réelle de chaque dent et de la culasse), roulements (modèle de Palmgren, charge = poids + force magnétique + balourd centrifuge), ventilation, couple résistant.
• Chaleur : température en régime établi, constante de temps, convection ; baisse de Br avec la température, qui réduit la FEM et les couples quand la vitesse augmente.
• Son : raies d'excitation magnétique (Hz, origine couple ou force, niveau pondéré A), indice global ; « Écouter » fait entendre l'excitation synthétisée, « Exporter le son » l'enregistre en WAV.
Vibration : le rotor, supposé rigide sur des roulements élastiques (raideur et amortissement réglables), est excité par la force magnétique tournante et par le balourd. FractLab donne le déplacement crête (µm), la vitesse vibratoire efficace avec sa zone ISO 10816 (repère indicatif), l'accélération, la vitesse critique et les vitesses auxquelles chaque harmonique de force excite cette résonance ; en torsion, l'ondulation de vitesse due au couple de détente (inertie du rotor + charge). Dans la vue 3D, le rotor tremble selon son orbite réelle, amplifiée (curseur « Amplification de la vibration ») ; l'orbite s'affiche à droite de l'oscilloscope.
Oscilloscope : FEM et couple sur trois périodes électriques, en millisecondes réelles : les signaux se resserrent quand la vitesse augmente.
Les coefficients (tôles, roulements, lubrifiant, balourd résiduel, température ambiante, coefficient de Br, convection) se règlent dans le panneau « Frottements, fer et thermique ». Ce sont des modèles d'ingénierie : le son est un indicateur d'excitation, pas un niveau mesuré au micro.
Vue transparente et champ : le bouton violet « 👁 Champ 3D » (barre principale, haut de l'onglet Moteur 3D, bandeau de la Synthèse) active tout en un clic ; les icônes 👁 🌈 🌀 ➡ posées sur la vue règlent transparence, carte, lignes et vecteurs. Sinon, cochez « Vue transparente » (opacité réglable) puis, dans « Champ », la carte de l'induction, les lignes de flux et/ou les vecteurs B. En flux radial, le champ est dessiné dans le plan médian de la machine (sur la face avant sans transparence) ; en flux axial, l'induction axiale Bz est dessinée au milieu de chaque entrefer, avec des flèches le long de l'axe. Le champ suit le rotor pendant l'animation.",
"The 3D motor tab spins the motor at its real speed, shown in slow motion.
Controls:
• Play / Pause; “Motor speed”: physical speed (10 to 60,000 rpm) used for every computation.
• “Slow motion”: display slow-down factor (×1 to ×1/100,000), ÷2 and ×2 buttons. The next line shows how many real milliseconds elapse per screen second.
• Teeth coloured by their instantaneous flux density, air-gap flux density ribbon, radial force vector (orange arrow).
Live dashboard:
• Instantaneous: position, simulated real time, electrical frequency, cogging torque, back-EMF of the three phases, radial force, maximum tooth flux density.
• Steady-state losses and friction: iron losses (hysteresis and eddy currents, Bertotti model applied to the actual flux density of each tooth and yoke section), bearings (Palmgren model, load = weight + magnetic force + centrifugal unbalance), windage, drag torque.
• Heat: steady-state temperature, time constant, convection; Br drop with temperature, which reduces back-EMF and torques as speed rises.
• Sound: magnetic excitation lines (Hz, torque or force origin, A-weighted level), overall index; “Listen” plays the synthesised excitation, “Export sound” saves it as WAV.
Vibration: the rotor, assumed rigid on elastic bearings (adjustable stiffness and damping), is excited by the rotating magnetic force and the unbalance. FractLab gives the peak displacement (µm), RMS vibration velocity with its ISO 10816 zone (indicative), acceleration, critical speed and the speeds at which each force harmonic excites that resonance; torsionally, the speed ripple caused by cogging torque (rotor + load inertia). In the 3D view the rotor shakes along its real, magnified orbit (“Vibration magnification” slider); the orbit is plotted to the right of the oscilloscope.
Oscilloscope: back-EMF and torque over three electrical periods, in real milliseconds: signals tighten as speed rises.
Coefficients (laminations, bearings, lubricant, residual unbalance, ambient temperature, Br coefficient, convection) are set in the “Friction, iron and thermal” panel. These are engineering models: the sound is an excitation indicator, not a microphone level.
Transparent view and field: the purple “👁 3D field” button (main bar, top of the 3D motor tab, Summary banner) turns everything on in one click; the 👁 🌈 🌀 ➡ icons on the view toggle transparency, map, lines and vectors. Otherwise, tick “Transparent view” (adjustable opacity) then, under “Field”, the flux density map, the flux lines and/or the B vectors. For radial flux, the field is drawn in the mid plane of the machine (on the front face without transparency); for axial flux, the axial flux density Bz is drawn at mid air gap, with arrows along the axis. The field follows the rotor during the animation.",
"La pestaña Motor 3D hace girar el motor a su velocidad real, vista a cámara lenta.
Controles:
• Reproducir / Pausa; «Velocidad del motor»: velocidad física (10 a 60 000 rpm) usada en todos los cálculos.
• «Cámara lenta»: factor de ralentización de la visualización (×1 a ×1/100 000), botones ÷2 y ×2. La línea contigua indica cuántos milisegundos reales transcurren por segundo de pantalla.
• Dientes coloreados según su inducción instantánea, cinta de inducción de entrehierro, vector de la fuerza radial (flecha naranja).
Panel en directo:
• Instantáneo: posición, tiempo real simulado, frecuencia eléctrica, par de cogging, FCEM de las tres fases, fuerza radial, inducción máxima en los dientes.
• Pérdidas y rozamientos en régimen establecido: pérdidas en el hierro (histéresis y corrientes de Foucault, modelo de Bertotti aplicado a la inducción real de cada diente y de la culata), rodamientos (modelo de Palmgren, carga = peso + fuerza magnética + desequilibrio centrífugo), ventilación, par resistente.
• Calor: temperatura en régimen establecido, constante de tiempo, convección; descenso de Br con la temperatura, que reduce la FCEM y los pares al aumentar la velocidad.
• Sonido: rayas de excitación magnética (Hz, origen par o fuerza, nivel ponderado A), índice global; «Escuchar» reproduce la excitación sintetizada, «Exportar el sonido» la guarda en WAV.
Vibración: el rotor, supuesto rígido sobre rodamientos elásticos (rigidez y amortiguamiento ajustables), está excitado por la fuerza magnética giratoria y por el desequilibrio. FractLab da el desplazamiento pico (µm), la velocidad de vibración eficaz con su zona ISO 10816 (orientativa), la aceleración, la velocidad crítica y las velocidades a las que cada armónico de fuerza excita esa resonancia; en torsión, el rizado de velocidad debido al par de cogging (inercia del rotor + carga). En la vista 3D el rotor vibra según su órbita real amplificada (cursor «Amplificación de la vibración»); la órbita se muestra a la derecha del osciloscopio.
Osciloscopio: FCEM y par en tres periodos eléctricos, en milisegundos reales: las señales se estrechan al aumentar la velocidad.
Los coeficientes (chapas, rodamientos, lubricante, desequilibrio residual, temperatura ambiente, coeficiente de Br, convección) se ajustan en el panel «Rozamientos, hierro y térmica». Son modelos de ingeniería: el sonido es un indicador de excitación, no un nivel medido con micrófono.
Vista transparente y campo: el botón violeta «👁 Campo 3D» (barra principal, arriba de la pestaña Motor 3D, banner de la Síntesis) lo activa todo con un clic; los iconos 👁 🌈 🌀 ➡ sobre la vista ajustan transparencia, mapa, líneas y vectores. Si no, marque «Vista transparente» (opacidad ajustable) y luego, en «Campo», el mapa de inducción, las líneas de flujo y/o los vectores B. En flujo radial, el campo se dibuja en el plano medio de la máquina (en la cara frontal sin transparencia); en flujo axial, la inducción axial Bz se dibuja en el centro de cada entrehierro, con flechas a lo largo del eje. El campo sigue al rotor durante la animación.",
"Der Reiter 3D-Motor dreht den Motor mit seiner realen Drehzahl, in Zeitlupe dargestellt.
Bedienung:
• Abspielen / Pause; „Motordrehzahl“: physikalische Drehzahl (10 bis 60 000 U/min) für alle Berechnungen.
• „Zeitlupe“: Verlangsamungsfaktor der Anzeige (×1 bis ×1/100 000), Schaltflächen ÷2 und ×2. Die Zeile daneben zeigt, wie viele reale Millisekunden pro Bildschirmsekunde vergehen.
• Zähne nach ihrer momentanen Induktion eingefärbt, Band der Luftspaltinduktion, Radialkraftvektor (oranger Pfeil).
Live-Dashboard:
• Momentanwerte: Position, simulierte Echtzeit, elektrische Frequenz, Rastmoment, Gegen-EMK der drei Phasen, Radialkraft, maximale Zahninduktion.
• Stationäre Verluste und Reibung: Eisenverluste (Hysterese und Wirbelströme, Bertotti-Modell auf die tatsächliche Induktion jedes Zahns und Jochabschnitts angewandt), Lager (Palmgren-Modell, Last = Gewicht + Magnetkraft + Fliehkraft der Unwucht), Luftreibung, Widerstandsmoment.
• Wärme: Beharrungstemperatur, Zeitkonstante, Konvektion; Abnahme von Br mit der Temperatur, die EMK und Momente mit steigender Drehzahl verringert.
• Geräusch: magnetische Anregungslinien (Hz, Ursprung Moment oder Kraft, A-bewerteter Pegel), Gesamtindex; „Anhören“ spielt die synthetisierte Anregung ab, „Ton exportieren“ speichert sie als WAV.
Schwingung: Der Rotor, als starr auf elastischen Lagern angenommen (Steifigkeit und Dämpfung einstellbar), wird durch die umlaufende Magnetkraft und die Unwucht angeregt. FractLab liefert Spitzenauslenkung (µm), effektive Schwinggeschwindigkeit mit ISO-10816-Zone (Richtwert), Beschleunigung, kritische Drehzahl und die Drehzahlen, bei denen jede Kraftharmonische diese Resonanz anregt; torsional die Drehzahlwelligkeit durch das Rastmoment (Rotor- + Lastträgheit). In der 3D-Ansicht schwingt der Rotor auf seinem realen, verstärkten Orbit (Regler „Schwingungsverstärkung“); der Orbit erscheint rechts neben dem Oszilloskop.
Oszilloskop: Gegen-EMK und Moment über drei elektrische Perioden in realen Millisekunden: Die Signale werden mit steigender Drehzahl enger.
Die Koeffizienten (Bleche, Lager, Schmierstoff, Restunwucht, Umgebungstemperatur, Br-Koeffizient, Konvektion) werden in der Leiste „Reibung, Eisen und Thermik“ eingestellt. Es sind Ingenieurmodelle: Der Ton ist ein Anregungsindikator, kein Mikrofonpegel.
Transparente Ansicht und Feld: die violette Schaltfläche „👁 3D-Feld“ (Hauptleiste, oben im Reiter 3D-Motor, Banner der Übersicht) schaltet alles mit einem Klick ein; die Symbole 👁 🌈 🌀 ➡ auf der Ansicht steuern Transparenz, Karte, Linien und Vektoren. Sonst „Transparente Ansicht“ ankreuzen (Deckkraft einstellbar), dann unter „Feld“ die Induktionskarte, die Flusslinien und/oder die B-Vektoren. Bei Radialfluss wird das Feld in der Mittelebene der Maschine gezeichnet (ohne Transparenz auf der Vorderseite); bei Axialfluss wird die axiale Induktion Bz in jeder Luftspaltmitte gezeichnet, mit Pfeilen entlang der Achse. Das Feld folgt dem Rotor während der Animation.",
        ],
    },
    Section {
        icon: "⚡",
        title: ["Cogging, FEM et forces", "Cogging, back-EMF and forces", "Cogging, FCEM y fuerzas", "Rastmoment, Gegen-EMK und Kräfte"],
        body: [
"Cogging : couple de détente à vide en fonction de la position du rotor, avec son spectre par ordre mécanique (nombre de raies par tour).
• Fenêtre « commune 360°/Q » (par défaut) : la même fenêtre pour toutes les variantes, indispensable pour comparer les spectres. Un rotor régulier a une période plus courte (360°/ppcm(Q, 2p)) ; un rotor apériodique n'a plus que la symétrie du stator, 360°/Q.
• Part de la dominante : énergie portée par la seule raie la plus forte (100 % = son pur, plus bas = spectre étalé). Le « pic tonal » (dominante / plancher du spectre) est aussi donné, mais il dépend du plancher numérique quand le signal est sans bruit : préférez la part de la dominante et l'entropie.
FEM : tensions de phase et ligne-ligne sur un tour mécanique complet, flux totalisés, spectre, THD, interharmoniques (raies hors multiples de p, signature de l'apériodicité), déséquilibre, constante Ke et KV estimé (2D, sans effets d'extrémité).
Forces : force magnétique résultante sur le rotor sur un tour, et son orbite (comparée au seuil quand il y en a un).",
"Cogging: no-load cogging torque versus rotor position, with its spectrum by mechanical order (number of lines per revolution).
• “Common 360°/Q” window (default): the same window for all variants, essential to compare spectra. A regular rotor has a shorter period (360°/lcm(Q, 2p)); an aperiodic rotor keeps only the stator symmetry, 360°/Q.
• Dominant share: energy carried by the strongest line alone (100 % = pure tone, lower = spread spectrum). The “tonal peak” (dominant / spectrum floor) is also given, but it depends on the numerical floor when the signal has no noise: prefer the dominant share and the entropy.
Back-EMF: phase and line-to-line voltages over a full mechanical revolution, linked fluxes, spectrum, THD, interharmonics (lines off multiples of p, the signature of aperiodicity), unbalance, constant Ke and estimated KV (2D, no end effects).
Forces: resultant magnetic force on the rotor over one revolution, and its orbit (compared with the limit when there is one).",
"Cogging: par de cogging en vacío según la posición del rotor, con su espectro por orden mecánico (número de rayas por vuelta).
• Ventana «común 360°/Q» (por defecto): la misma ventana para todas las variantes, indispensable para comparar espectros. Un rotor regular tiene un periodo más corto (360°/mcm(Q, 2p)); un rotor aperiódico solo conserva la simetría del estátor, 360°/Q.
• Peso del dominante: energía de la raya más fuerte sola (100 % = tono puro, más bajo = espectro repartido). También se da el «pico tonal» (dominante / suelo del espectro), pero depende del suelo numérico cuando la señal no tiene ruido: prefiera el peso del dominante y la entropía.
FCEM: tensiones de fase y entre líneas en una vuelta mecánica completa, flujos concatenados, espectro, THD, interarmónicos (rayas fuera de los múltiplos de p, firma de la aperiodicidad), desequilibrio, constante Ke y KV estimado (2D, sin efectos de extremo).
Fuerzas: fuerza magnética resultante sobre el rotor en una vuelta, y su órbita (comparada con el umbral cuando lo hay).",
"Rastmoment: Leerlauf-Rastmoment über der Rotorposition mit Spektrum nach mechanischer Ordnung (Linien pro Umdrehung).
• Fenster „gemeinsam 360°/Q“ (Standard): dasselbe Fenster für alle Varianten, unerlässlich für den Spektrenvergleich. Ein regulärer Rotor hat eine kürzere Periode (360°/kgV(Q, 2p)); ein aperiodischer Rotor behält nur die Statorsymmetrie 360°/Q.
• Anteil der Dominanten: Energie der stärksten Linie allein (100 % = reiner Ton, niedriger = breiteres Spektrum). Die „Tonspitze“ (Dominante / Spektrumboden) wird ebenfalls angegeben, hängt aber ohne Rauschen vom numerischen Boden ab: besser Anteil der Dominanten und Entropie verwenden.
Gegen-EMK: Phasen- und Leiterspannungen über eine volle mechanische Umdrehung, verkettete Flüsse, Spektrum, THD, Zwischenharmonische (Linien außerhalb der Vielfachen von p, Signatur der Aperiodizität), Unsymmetrie, Konstante Ke und geschätzter KV (2D, ohne Endeffekte).
Kräfte: resultierende Magnetkraft auf den Rotor über eine Umdrehung und ihr Orbit (mit der Grenze verglichen, sofern vorhanden).",
        ],
    },
    Section {
        icon: "📊",
        title: ["Comparaison et cartes 3D", "Comparison and 3D maps", "Comparación y mapas 3D", "Vergleich und 3D-Karten"],
        body: [
"Comparaison : calcule toutes les répartitions proposées avec vos dimensions actuelles, seule la répartition des aimants changeant. Vous obtenez les courbes superposées, un histogramme et un tableau (cogging, écart à la répartition de référence, part de la dominante, entropie, force, THD, autocorrélation) coloré selon les seuils.
Cartes 3D :
• « Cogging × amplitude des décalages » : couple de détente en fonction de la position du rotor et de l'amplitude des décalages, de 0 (régulier) à la valeur choisie. Montre d'un coup d'œil comment le cogging évolue quand on « brouille » la répartition.
• « Br(θ, θr) » : induction d'entrefer selon la position sur le stator et la position du rotor.
Survolez la surface : le point le plus proche est marqué et ses trois coordonnées exactes s'affichent. Glisser : orbite ; molette : zoom ; « maillage » : affiche les arêtes.",
"Comparison: computes every proposed layout with your current dimensions, only the magnet layout changing. You get overlaid curves, a bar chart and a table (cogging, deviation from the reference layout, dominant share, entropy, force, THD, autocorrelation) coloured against the limits.
3D maps:
• “Cogging × offset amplitude”: cogging torque versus rotor position and offset amplitude, from 0 (regular) to the chosen value. Shows at a glance how cogging evolves when the layout is “scrambled”.
• “Br(θ, θr)”: air-gap flux density versus stator position and rotor position.
Hover the surface: the nearest point is marked and its three exact coordinates are shown. Drag: orbit; wheel: zoom; “wireframe”: shows the edges.",
"Comparación: calcula todas las distribuciones propuestas con sus dimensiones actuales; solo cambia la distribución de los imanes. Obtiene las curvas superpuestas, un histograma y una tabla (cogging, desviación respecto a la distribución de referencia, peso del dominante, entropía, fuerza, THD, autocorrelación) coloreada según los umbrales.
Mapas 3D:
• «Cogging × amplitud de los desfases»: par de cogging según la posición del rotor y la amplitud de los desfases, de 0 (regular) al valor elegido. Muestra de un vistazo cómo evoluciona el cogging al «desordenar» la distribución.
• «Br(θ, θr)»: inducción de entrehierro según la posición en el estátor y la posición del rotor.
Pase el ratón sobre la superficie: se marca el punto más cercano y se muestran sus tres coordenadas exactas. Arrastrar: órbita; rueda: zoom; «malla»: muestra las aristas.",
"Vergleich: berechnet alle vorgeschlagenen Anordnungen mit Ihren aktuellen Abmessungen; nur die Magnetverteilung ändert sich. Sie erhalten überlagerte Kurven, ein Balkendiagramm und eine Tabelle (Rastmoment, Abweichung zur Referenzanordnung, Anteil der Dominanten, Entropie, Kraft, THD, Autokorrelation), farbig nach Grenzwerten.
3D-Karten:
• „Rastmoment × Versatzamplitude“: Rastmoment über Rotorposition und Versatzamplitude, von 0 (regulär) bis zum gewählten Wert. Zeigt auf einen Blick, wie sich das Rastmoment beim „Verwürfeln“ der Verteilung entwickelt.
• „Br(θ, θr)“: Luftspaltinduktion über Statorposition und Rotorposition.
Fläche überfahren: Der nächste Punkt wird markiert und seine drei exakten Koordinaten angezeigt. Ziehen: Orbit; Mausrad: Zoom; „Gitter“: zeigt die Kanten.",
        ],
    },
    Section {
        icon: "✔",
        title: ["Validation", "Validation", "Validación", "Validierung"],
        body: [
"Le modèle analytique est contrôlé par un second solveur entièrement indépendant (volumes finis sur grille polaire, fer maillé de perméabilité 10⁵, couple d'Arkkio). « Lancer la validation » calcule le couple de détente aux mêmes positions avec les deux méthodes et affiche l'écart rapporté à l'amplitude. Un écart de quelques pour cent est normal (le solveur numérique a un fer de perméabilité finie et une grille discrète) ; un écart important signale une configuration hors des hypothèses du modèle.",
"The analytical model is checked by a second, fully independent solver (finite volumes on a polar grid, meshed iron of permeability 10⁵, Arkkio torque). “Run validation” computes the cogging torque at the same positions with both methods and shows the deviation relative to the amplitude. A few percent is normal (the numerical solver has finite-permeability iron and a discrete grid); a large deviation signals a configuration outside the model assumptions.",
"El modelo analítico se verifica con un segundo solver totalmente independiente (volúmenes finitos sobre malla polar, hierro mallado de permeabilidad 10⁵, par de Arkkio). «Iniciar la validación» calcula el par de cogging en las mismas posiciones con ambos métodos y muestra la desviación respecto a la amplitud. Unos pocos por ciento es normal (el solver numérico tiene hierro de permeabilidad finita y una malla discreta); una desviación grande indica una configuración fuera de las hipótesis del modelo.",
"Das analytische Modell wird durch einen zweiten, völlig unabhängigen Löser geprüft (Finite Volumen auf polarem Gitter, vernetztes Eisen mit Permeabilität 10⁵, Arkkio-Moment). „Validierung starten“ berechnet das Rastmoment an denselben Positionen mit beiden Methoden und zeigt die Abweichung bezogen auf die Amplitude. Einige Prozent sind normal (der numerische Löser hat Eisen endlicher Permeabilität und ein diskretes Gitter); eine große Abweichung weist auf eine Konfiguration außerhalb der Modellannahmen hin.",
        ],
    },
    Section {
        icon: "📂",
        title: ["Projets SIMUMSAP et figures MATLAB", "SIMUMSAP projects and MATLAB figures", "Proyectos SIMUMSAP y figuras MATLAB", "SIMUMSAP-Projekte und MATLAB-Abbildungen"],
        body: [
"FractLab ouvre les projets SIMUMSAP créés sous MATLAB, sans MATLAB.
• Fichier > Ouvrir un projet SIMUMSAP (MATLAB)… : la machine est reconstruite telle quelle (flux axial multi-tranches ou flux radial, 1 ou 2 entrefers, encoches semi-fermées, aimants, bobinage lu dans la matrice topologique et répété sur chaque période).
• La fenêtre « Projet SIMUMSAP » liste tous les champs du fichier et compare les inductances calculées par SIMUMSAP à celles de FractLab (ramenées à une période et un entrefer, comme dans SIMUMSAP).
• « Hypothèses SIMUMSAP (encoches ouvertes) » : le modèle axial de SIMUMSAP ignore les becs de dents ; ce bouton applique les mêmes hypothèses pour comparer à résultats égaux.
• « Charger les figures du dossier » lit toutes les figures .fig du projet ; Fichier > Importer > Figures MATLAB (.fig) / données (.mat) en ouvre d'autres. Chaque courbe peut être superposée aux vues Cogging, FEM, Champ, Forces ou Charge.",
"FractLab opens SIMUMSAP projects created in MATLAB, without MATLAB.
• File > Open a SIMUMSAP project (MATLAB)…: the machine is rebuilt as it is (multi-slice axial flux or radial flux, 1 or 2 air gaps, semi-closed slots, magnets, winding read from the topology matrix and repeated over each period).
• The “SIMUMSAP project” window lists every field of the file and compares the inductances computed by SIMUMSAP with FractLab's (per period and per air gap, as in SIMUMSAP).
• “SIMUMSAP assumptions (open slots)”: SIMUMSAP's axial model ignores tooth tips; this button applies the same assumptions for a like-for-like comparison.
• “Load the folder's figures” reads every .fig figure of the project; File > Import > MATLAB figures (.fig) / data (.mat) opens others. Each curve can be overlaid on the Cogging, Back-EMF, Field, Forces or Load views.",
"FractLab abre los proyectos SIMUMSAP creados con MATLAB, sin MATLAB.
• Archivo > Abrir un proyecto SIMUMSAP (MATLAB)…: la máquina se reconstruye tal cual (flujo axial multicapa o flujo radial, 1 o 2 entrehierros, ranuras semicerradas, imanes, devanado leído de la matriz topológica y repetido en cada periodo).
• La ventana «Proyecto SIMUMSAP» lista todos los campos del archivo y compara las inductancias calculadas por SIMUMSAP con las de FractLab (por periodo y por entrehierro, como en SIMUMSAP).
• «Hipótesis SIMUMSAP (ranuras abiertas)»: el modelo axial de SIMUMSAP ignora los picos de diente; este botón aplica las mismas hipótesis para comparar en igualdad de condiciones.
• «Cargar las figuras de la carpeta» lee todas las figuras .fig del proyecto; Archivo > Importar > Figuras MATLAB (.fig) / datos (.mat) abre otras. Cada curva puede superponerse a las vistas Cogging, FCEM, Campo, Fuerzas o Carga.",
"FractLab öffnet in MATLAB erstellte SIMUMSAP-Projekte, ohne MATLAB.
• Datei > SIMUMSAP-Projekt öffnen (MATLAB)…: Die Maschine wird unverändert nachgebildet (Axialfluss mit Mehrschichtmodell oder Radialfluss, 1 oder 2 Luftspalte, halbgeschlossene Nuten, Magnete, Wicklung aus der Topologiematrix gelesen und je Periode wiederholt).
• Das Fenster „SIMUMSAP-Projekt“ listet alle Felder der Datei und vergleicht die von SIMUMSAP berechneten Induktivitäten mit denen von FractLab (je Periode und Luftspalt, wie in SIMUMSAP).
• „SIMUMSAP-Annahmen (offene Nuten)“: Das Axialmodell von SIMUMSAP ignoriert Zahnköpfe; diese Schaltfläche übernimmt dieselben Annahmen für einen fairen Vergleich.
• „Abbildungen des Ordners laden“ liest alle .fig-Abbildungen des Projekts; Datei > Importieren > MATLAB-Abbildungen (.fig) / Daten (.mat) öffnet weitere. Jede Kurve lässt sich über die Ansichten Rastmoment, EMK, Feld, Kräfte oder Last legen.",
        ],
    },
    Section {
        icon: "◎",
        title: ["Topologies, becs de dents et bobinage général", "Topologies, tooth tips and general winding", "Topologías, picos de diente y devanado general", "Topologien, Zahnköpfe und allgemeine Wicklung"],
        body: [
"Type de machine (panneau Topologie) :
• Flux radial, rotor extérieur (cloche, moteurs de drone) ; flux radial, rotor intérieur ; flux axial à disques, calculé par la technique multi-tranches (nombre de tranches et 1 ou 2 entrefers réglables, arc des aimants aux rayons intérieur et extérieur).
• Becs de dents : une épaisseur non nulle crée des encoches semi-fermées (ouverture étroite, corps d'encoche plus large).
• Flux axial : l'onglet Géométrie montre la vue de face (stator et aimants du rotor superposés, dents trapézoïdales) et la coupe axiale (empilement des disques, épaisseurs) ; le moteur 3D est dessiné en disques. La force indiquée est la force transverse (dans le plan du disque) ; l'attraction axiale de chaque disque est donnée à part.
• Flux axial, onglet Champ : choix de la tranche ; courbes de l'induction axiale Bz et tangentielle ; carte du champ de la tranche développée (circonférence à l'échelle réelle, molette pour zoomer, glisser pour parcourir le tour) ; vue « Entrefer sur tout le disque » (Bz sur toute la face, une couronne par tranche).
• Flux axial, onglet Forces : défauts d'alignement du rotor — décalage axial et inclinaison — avec attraction d'un disque, force axiale nette, raideur axiale, moment de basculement et raideur angulaire (raideurs négatives : le rotor est attiré vers le côté le plus proche).
• Flux axial, onglet Validation : la tranche de référence est recalculée par les volumes finis en géométrie plane.
• Précision : le nombre d'harmoniques est relevé automatiquement si la géométrie l'exige (grandes machines à ouvertures étroites) ; le nombre utilisé s'affiche dans « Précision du calcul ».
Bobinage général et charge :
• Par défaut, le bobinage concentré est déduit automatiquement. « Éditer bobine par bobine » permet de définir chaque bobine : phase, encoche aller, encoche retour, spires.
• Courant crête, angle du courant par rapport à la FEM, voies en parallèle et coefficient de remplissage servent au fonctionnement en charge.",
"Machine type (Topology panel):
• Radial flux, outer rotor (bell, drone motors); radial flux, inner rotor; axial-flux disc machine, computed with the multi-slice technique (number of slices and 1 or 2 air gaps adjustable, magnet arc at inner and outer radius).
• Tooth tips: a non-zero thickness creates semi-closed slots (narrow opening, wider slot body).
• Axial flux: the Geometry tab shows the front view (stator and rotor magnets overlaid, trapezoidal teeth) and the axial section (stack of discs, thicknesses); the 3D motor is drawn as discs. The force shown is the transverse force (in the plane of the disc); the axial attraction of each disc is given separately.
• Axial flux, Field tab: slice selector; axial Bz and tangential flux density curves; field map of the unrolled slice (circumference at true scale, wheel to zoom, drag to travel around); “Air gap over the whole disc” view (Bz over the whole face, one ring per slice).
• Axial flux, Forces tab: rotor misalignment — axial offset and tilt — with attraction of one disc, net axial force, axial stiffness, tipping moment and angular stiffness (negative stiffnesses: the rotor is pulled towards the closer side).
• Axial flux, Validation tab: the reference slice is recomputed by finite volumes in planar geometry.
• Accuracy: the harmonic count is raised automatically when the geometry requires it (large machines with narrow openings); the count used is shown in “Computation accuracy”.
General winding and load:
• By default the concentrated winding is derived automatically. “Edit coil by coil” defines every coil: phase, go slot, return slot, turns.
• Peak current, current angle relative to the back-EMF, parallel paths and fill factor are used for on-load operation.",
"Tipo de máquina (panel Topología):
• Flujo radial, rotor exterior (campana, motores de dron); flujo radial, rotor interior; máquina de discos de flujo axial, calculada con la técnica multicapa (número de capas y 1 o 2 entrehierros ajustables, arco de imán en los radios interior y exterior).
• Picos de diente: un espesor no nulo crea ranuras semicerradas (apertura estrecha, cuerpo de ranura más ancho).
• Flujo axial: la pestaña Geometría muestra la vista frontal (estátor e imanes del rotor superpuestos, dientes trapezoidales) y el corte axial (apilamiento de discos, espesores); el motor 3D se dibuja en discos. La fuerza indicada es la fuerza transversal (en el plano del disco); la atracción axial de cada disco se da aparte.
• Flujo axial, pestaña Campo: elección de la capa; curvas de la inducción axial Bz y tangencial; mapa del campo de la capa desarrollada (circunferencia a escala real, rueda para zoom, arrastrar para recorrer la vuelta); vista «Entrehierro en todo el disco» (Bz en toda la cara, una corona por capa).
• Flujo axial, pestaña Fuerzas: defectos de alineación del rotor — desplazamiento axial e inclinación — con atracción de un disco, fuerza axial neta, rigidez axial, momento de vuelco y rigidez angular (rigideces negativas: el rotor es atraído hacia el lado más cercano).
• Flujo axial, pestaña Validación: la capa de referencia se recalcula por volúmenes finitos en geometría plana.
• Precisión: el número de armónicos se aumenta automáticamente si la geometría lo exige (máquinas grandes con aperturas estrechas); el número usado se muestra en «Precisión del cálculo».
Devanado general y carga:
• Por defecto el devanado concentrado se deduce automáticamente. «Editar bobina a bobina» define cada bobina: fase, ranura de ida, ranura de retorno, espiras.
• Corriente pico, ángulo de la corriente respecto a la FCEM, ramas en paralelo y factor de llenado sirven para el funcionamiento en carga.",
"Maschinentyp (Leiste Topologie):
• Radialfluss, Außenläufer (Glocke, Drohnenmotoren); Radialfluss, Innenläufer; Axialfluss-Scheibenmaschine, mit Mehrschichttechnik berechnet (Anzahl Scheiben und 1 oder 2 Luftspalte einstellbar, Magnetbogen am Innen- und Außenradius).
• Zahnköpfe: eine Dicke ungleich null erzeugt halbgeschlossene Nuten (schmale Öffnung, breiterer Nutkörper).
• Axialfluss: der Reiter Geometrie zeigt die Vorderansicht (Stator und Rotormagnete überlagert, trapezförmige Zähne) und den Axialschnitt (Scheibenstapel, Dicken); der 3D-Motor wird als Scheiben dargestellt. Die angezeigte Kraft ist die Querkraft (in der Scheibenebene); die axiale Anziehung jeder Scheibe wird getrennt angegeben.
• Axialfluss, Reiter Feld: Wahl der Scheibe; Kurven der axialen Induktion Bz und der tangentialen; Feldkarte der abgewickelten Scheibe (Umfang maßstabsgetreu, Mausrad zum Zoomen, Ziehen zum Abfahren); Ansicht „Luftspalt über die ganze Scheibe“ (Bz über die ganze Fläche, ein Ring je Scheibe).
• Axialfluss, Reiter Kräfte: Fehlausrichtung des Rotors — axialer Versatz und Neigung — mit Anziehung einer Scheibe, axialer Nettokraft, axialer Steifigkeit, Kippmoment und Winkelsteifigkeit (negative Steifigkeiten: der Rotor wird zur näheren Seite gezogen).
• Axialfluss, Reiter Validierung: die Referenzscheibe wird mit Finiten Volumen in ebener Geometrie nachgerechnet.
• Genauigkeit: die Zahl der Harmonischen wird automatisch erhöht, wenn die Geometrie es erfordert (große Maschinen mit schmalen Öffnungen); die verwendete Zahl wird unter „Rechengenauigkeit“ angezeigt.
Allgemeine Wicklung und Last:
• Standardmäßig wird die Zahnspulenwicklung automatisch bestimmt. „Spule für Spule bearbeiten“ definiert jede Spule: Phase, Hin-Nut, Rück-Nut, Windungen.
• Spitzenstrom, Stromwinkel zur EMK, parallele Zweige und Füllfaktor dienen dem Lastbetrieb.",
        ],
    },
    Section {
        icon: "⚙",
        title: ["Fonctionnement en charge", "On-load operation", "Funcionamiento en carga", "Lastbetrieb"],
        body: [
"L'onglet Charge alimente le bobinage par des courants sinusoïdaux équilibrés, alignés sur la FEM (angle réglable).
• Couple en charge (tenseur de Maxwell) et contrôle énergétique Σ i·dλ/dθ + couple de détente : les deux courbes doivent se superposer.
• Couple moyen, ondulation, constante de couple Kt, puissance mécanique, couple moyen en fonction de l'angle du courant.
• Matrice d'inductances (hors têtes de bobines), inductance synchrone, résistance de phase estimée, pertes Joule et rendement estimé.
• Têtes de bobines : leur inductance de fuite est ajoutée à l'inductance synchrone (coefficient de perméance λ réglable dans « Bobinage général et charge » ; λ = 0 les ignore, comme SIMUMSAP).
• Saturation magnétique : inductions dans les dents, la culasse du stator et la culasse du rotor (vert sous le coude, orange au-dessus, rouge en saturation forte), FEM et couple corrigés, courbe couple / courant linéaire et saturée jusqu'à 3 fois le courant nominal.",
"The Load tab feeds the winding with balanced sinusoidal currents aligned with the back-EMF (adjustable angle).
• On-load torque (Maxwell stress tensor) and energy check Σ i·dλ/dθ + cogging torque: both curves must coincide.
• Mean torque, ripple, torque constant Kt, mechanical power, mean torque versus current angle.
• Inductance matrix (end windings excluded), synchronous inductance, estimated phase resistance, copper losses and estimated efficiency.
• End windings: their leakage inductance is added to the synchronous inductance (permeance coefficient λ adjustable in “General winding and load”; λ = 0 ignores them, as SIMUMSAP does).
• Magnetic saturation: flux densities in the teeth, stator yoke and rotor yoke (green below the knee, orange above, red when strongly saturated), corrected back-EMF and torque, linear and saturated torque / current curve up to 3 times the rated current.",
"La pestaña Carga alimenta el devanado con corrientes sinusoidales equilibradas, alineadas con la FCEM (ángulo ajustable).
• Par en carga (tensor de Maxwell) y control energético Σ i·dλ/dθ + par de cogging: ambas curvas deben coincidir.
• Par medio, rizado, constante de par Kt, potencia mecánica, par medio en función del ángulo de la corriente.
• Matriz de inductancias (sin cabezas de bobina), inductancia síncrona, resistencia de fase estimada, pérdidas Joule y rendimiento estimado.
• Cabezas de bobina: su inductancia de fuga se suma a la inductancia síncrona (coeficiente de permeancia λ ajustable en «Devanado general y carga»; λ = 0 las ignora, como SIMUMSAP).
• Saturación magnética: inducciones en los dientes, el yugo del estátor y el yugo del rotor (verde bajo el codo, naranja por encima, rojo en saturación fuerte), FCEM y par corregidos, curva par / corriente lineal y saturada hasta 3 veces la corriente nominal.",
"Der Reiter Last speist die Wicklung mit symmetrischen Sinusströmen, an der EMK ausgerichtet (Winkel einstellbar).
• Lastmoment (Maxwell-Spannungstensor) und Energieprüfung Σ i·dλ/dθ + Rastmoment: beide Kurven müssen übereinanderliegen.
• Mittleres Moment, Welligkeit, Momentkonstante Kt, mechanische Leistung, mittleres Moment über dem Stromwinkel.
• Induktivitätsmatrix (ohne Wickelköpfe), Synchroninduktivität, geschätzter Phasenwiderstand, Kupferverluste und geschätzter Wirkungsgrad.
• Wickelköpfe: ihre Streuinduktivität wird zur Synchroninduktivität addiert (Permeanzbeiwert λ einstellbar unter „Allgemeine Wicklung und Last“; λ = 0 ignoriert sie, wie SIMUMSAP).
• Magnetische Sättigung: Induktionen in Zähnen, Statorjoch und Rotorjoch (grün unter dem Knick, orange darüber, rot bei starker Sättigung), korrigierte EMK und korrigiertes Moment, lineare und gesättigte Moment-/Strom-Kurve bis zum 3-fachen Nennstrom.",
        ],
    },
    Section {
        icon: "⏱",
        title: ["Simulation dynamique, défauts et excentricité", "Dynamic simulation, faults and eccentricity", "Simulación dinámica, defectos y excentricidad", "Dynamische Simulation, Fehler und Exzentrizität"],
        body: [
"L'onglet Dynamique simule la machine dans le temps.
• Alimentation : réseau sinusoïdal, onduleur MLI autopiloté, commutation 120° (contrôleur de drone, ESC), ou redresseur à diodes en génératrice (bus continu, capacité, résistance de charge).
• Charge mécanique : aucune, couple constant, hélice / ventilateur / pompe (k·ω²), convoyeur, éolienne entraînant la génératrice.
• Défauts : phase ouverte, court-circuit entre spires de la phase A (fraction de spires, résistance de défaut), à partir d'un instant choisi.
• Résultats : vitesse, couple, courants, tension, bus continu et courant de défaut en fonction du temps.
Excentricité (onglet Forces) : réglez l'excentricité ε (statique ou dynamique) puis « Calculer l'attraction magnétique déséquilibrée » ; la force et la raideur magnétique s'affichent (méthode de perméance, valable pour une petite excentricité).",
"The Dynamics tab simulates the machine over time.
• Supply: sinusoidal grid, self-synchronised PWM inverter, 120° commutation (drone controller, ESC), or diode rectifier in generator mode (DC bus, capacitance, load resistance).
• Mechanical load: none, constant torque, propeller / fan / pump (k·ω²), conveyor, wind turbine driving the generator.
• Faults: open phase, inter-turn short circuit on phase A (fraction of turns, fault resistance), from a chosen instant.
• Results: speed, torque, currents, voltage, DC bus and fault current versus time.
Eccentricity (Forces tab): set the eccentricity ε (static or dynamic) then “Compute unbalanced magnetic pull”; the force and the magnetic stiffness are shown (permeance method, valid for small eccentricity).",
"La pestaña Dinámica simula la máquina en el tiempo.
• Alimentación: red sinusoidal, inversor MLI autopilotado, conmutación 120° (controlador de dron, ESC) o rectificador de diodos en generador (bus continuo, capacidad, resistencia de carga).
• Carga mecánica: ninguna, par constante, hélice / ventilador / bomba (k·ω²), transportador, aerogenerador que acciona el generador.
• Defectos: fase abierta, cortocircuito entre espiras de la fase A (fracción de espiras, resistencia de defecto), a partir de un instante elegido.
• Resultados: velocidad, par, corrientes, tensión, bus continuo y corriente de defecto en función del tiempo.
Excentricidad (pestaña Fuerzas): ajuste la excentricidad ε (estática o dinámica) y luego «Calcular la atracción magnética desequilibrada»; se muestran la fuerza y la rigidez magnética (método de permeancia, válido para pequeña excentricidad).",
"Der Reiter Dynamik simuliert die Maschine über der Zeit.
• Speisung: Sinusnetz, selbstgeführter PWM-Wechselrichter, 120°-Kommutierung (Drohnenregler, ESC) oder Diodengleichrichter im Generatorbetrieb (Zwischenkreis, Kapazität, Lastwiderstand).
• Mechanische Last: keine, konstantes Moment, Propeller / Lüfter / Pumpe (k·ω²), Förderband, Windturbine als Antrieb des Generators.
• Fehler: offene Phase, Windungsschluss in Phase A (Windungsanteil, Fehlerwiderstand), ab einem gewählten Zeitpunkt.
• Ergebnisse: Drehzahl, Moment, Ströme, Spannung, Zwischenkreis und Fehlerstrom über der Zeit.
Exzentrizität (Reiter Kräfte): Exzentrizität ε (statisch oder dynamisch) einstellen, dann „Einseitigen magnetischen Zug berechnen“; Kraft und magnetische Steifigkeit werden angezeigt (Permeanzmethode, gültig für kleine Exzentrizität).",
        ],
    },
    Section {
        icon: "✨",
        title: ["Ajustement I.A, console et comparaison des solutions", "A.I. tuning, console and solution comparison", "Ajuste I.A, consola y comparación de soluciones", "KI-Optimierung, Konsole und Lösungsvergleich"],
        body: [
"Étape Optimisation : trois onglets qui s'enchaînent.
Onglet « Ajustement I.A » :
• Objectif : couple de détente minimal, force radiale minimale, FEM maximale, compromis, ondulation de couple minimale, spectre le plus étalé, entropie spectrale maximale, ou compromis spectre étalé / cogging faible.
• Réglages libres (arc des aimants, ouverture d'encoche, épaisseur des aimants, amplitude des décalages, becs de dents, entrefer), nombre d'essais, part de FEM conservée.
• Contraintes facultatives : culasse rotor non saturée (épaisseur dimensionnée pour chaque essai), pertes Joule au plus « original + x % », couple moyen au moins « x % de l'original », amplitude minimale des décalages.
• Chaque essai est un calcul du moteur ; la recherche (Nelder–Mead multi-départs) respecte les contraintes. Les 5 meilleures solutions s'affichent en direct.
Onglet « Console » (il s'ouvre au lancement) :
• Moteur I.A : bilan départ → meilleure solution, graphe 3D des possibilités explorées (il tourne ; glisser pour tourner, molette pour zoomer, clic sur un nœud pour le détail et « Appliquer cet essai »), journal des essais en bas (déplier : au moins 5 lignes).
• Script : commandes enchaînées (taper « aide »).
Onglet « Comparaison des solutions » (il s'ouvre à la fin de l'ajustement) :
• L'original et les propositions sont recalculés en calcul complet ; tableau de 18 métriques avec l'écart à l'original et la meilleure valeur de chaque ligne (★), six graphiques superposés, profil normalisé.
• « Appliquer » sous la solution choisie, ou « Garder l'original ». Historique des comparaisons, copie du tableau en CSV.",
"Optimisation step: three tabs in sequence.
“A.I. tuning” tab:
• Goal: minimum cogging torque, minimum radial force, maximum back-EMF, trade-off, minimum torque ripple, most spread spectrum, maximum spectral entropy, or spread spectrum / low cogging trade-off.
• Free settings (magnet arc, slot opening, magnet thickness, offset amplitude, tooth tips, air gap), number of trials, share of back-EMF kept.
• Optional constraints: rotor yoke not saturated (thickness sized for each trial), copper losses at most “original + x %”, mean torque at least “x % of original”, minimum offset amplitude.
• Each trial is a motor computation; the search (multi-start Nelder–Mead) respects the constraints. The 5 best solutions are shown live.
“Console” tab (opens when the tuning starts):
• A.I. engine: start → best summary, 3D graph of the explored possibilities (it rotates; drag to rotate, wheel to zoom, click a node for details and “Apply this trial”), trial log at the bottom (expand: at least 5 lines).
• Script: chained commands (type “help”).
“Solution comparison” tab (opens at the end of the tuning):
• The original and the proposals are fully recomputed; table of 18 metrics with the deviation from the original and the best value of each row (★), six overlaid charts, normalised profile.
• “Apply” under the chosen solution, or “Keep the original”. Comparison history, table copied as CSV.",
"Etapa Optimización: tres pestañas encadenadas.
Pestaña «Ajuste I.A»:
• Objetivo: par de cogging mínimo, fuerza radial mínima, FCEM máxima, compromiso, rizado de par mínimo, espectro más repartido, entropía espectral máxima o compromiso espectro repartido / cogging bajo.
• Ajustes libres (arco de imanes, apertura de ranura, espesor de imanes, amplitud de desfases, picos de diente, entrehierro), número de ensayos, parte de FCEM conservada.
• Restricciones opcionales: culata del rotor no saturada (espesor dimensionado en cada ensayo), pérdidas Joule como máximo «original + x %», par medio al menos «x % del original», amplitud mínima de desfases.
• Cada ensayo es un cálculo del motor; la búsqueda (Nelder–Mead multiarranque) respeta las restricciones. Las 5 mejores soluciones se muestran en directo.
Pestaña «Consola» (se abre al lanzar):
• Motor I.A: resumen inicio → mejor solución, grafo 3D de las posibilidades exploradas (gira; arrastrar para girar, rueda para el zoom, clic en un nodo para el detalle y «Aplicar este ensayo»), registro de ensayos abajo (desplegar: al menos 5 líneas).
• Script: comandos encadenados (escriba «aide»).
Pestaña «Comparación de las soluciones» (se abre al terminar el ajuste):
• El original y las propuestas se recalculan con el cálculo completo; tabla de 18 métricas con la desviación respecto al original y el mejor valor de cada fila (★), seis gráficos superpuestos, perfil normalizado.
• «Aplicar» bajo la solución elegida, o «Conservar el original». Historial de comparaciones, copia de la tabla en CSV.",
"Schritt Optimierung: drei aufeinanderfolgende Reiter.
Reiter „KI-Optimierung“:
• Ziel: minimales Rastmoment, minimale Radialkraft, maximale EMK, Kompromiss, minimale Drehmomentwelligkeit, breitestes Spektrum, maximale spektrale Entropie oder Kompromiss breites Spektrum / geringes Rastmoment.
• Freie Parameter (Magnetbogen, Nutöffnung, Magnetdicke, Versatzamplitude, Zahnköpfe, Luftspalt), Anzahl der Versuche, erhaltener EMK-Anteil.
• Optionale Nebenbedingungen: Rotorjoch nicht gesättigt (Dicke je Versuch bemessen), Kupferverluste höchstens „Original + x %“, mittleres Moment mindestens „x % des Originals“, minimale Versatzamplitude.
• Jeder Versuch ist eine Motorberechnung; die Suche (Mehrstart-Nelder–Mead) beachtet die Nebenbedingungen. Die 5 besten Lösungen erscheinen live.
Reiter „Konsole“ (öffnet sich beim Start):
• K.I.-Engine: Übersicht Start → beste Lösung, 3D-Graph der erkundeten Möglichkeiten (er dreht sich; ziehen zum Drehen, Mausrad zum Zoomen, Klick auf einen Knoten für Details und „Diesen Versuch übernehmen“), Versuchsprotokoll unten (aufklappen: mindestens 5 Zeilen).
• Skript: verkettete Befehle („aide“ eingeben).
Reiter „Lösungsvergleich“ (öffnet sich am Ende der Abstimmung):
• Original und Vorschläge werden vollständig neu berechnet; Tabelle mit 18 Kennwerten, Abweichung zum Original und bestem Wert je Zeile (★), sechs überlagerte Diagramme, normiertes Profil.
• „Übernehmen“ unter der gewählten Lösung oder „Original behalten“. Vergleichsverlauf, Tabelle als CSV kopieren.",
        ],
    },
    Section {
        icon: "📝",
        title: ["Rédaction, impression et sauvegarde générale", "Writing, printing and full save", "Redacción, impresión y guardado general", "Redaktion, Drucken und Gesamtsicherung"],
        body: [
"Rédaction (bouton 📝) :
• Remplissez la fiche du projet, entièrement facultative : titre, sous-titre, auteurs, organisation, INPI, autres dépôts, licence, arXiv, DOI, GitHub, site web, adresses mail, LinkedIn, mots-clés, résumé, remarques. Elle est enregistrée avec le projet.
• Choisissez la langue du document puis « Générer le document » : les textes fixes (titres, tableaux, méthode) sont traduits et les nombres prennent le séparateur décimal de la langue. Titre, sous-titre, mots-clés, résumé et remarques se saisissent pour chaque langue : un champ laissé vide reprend le texte français.
• Basculez entre la source LaTeX (modifiable) et l'aperçu PDF. Le PDF est compilé par LaTeX s'il est installé, sinon produit par le générateur intégré. Chaque page porte le logo FractLab et sa version.
• Ouvrir le PDF, enregistrer .tex / .pdf, et ⎙ Imprimer : choix de l'imprimante et du nombre de copies.
Sauvegarde générale (bouton 💾) : un dossier daté reçoit le document LaTeX et PDF, le logo, les données du projet (JSON), une sauvegarde MATLAB (.mat) relisible par MATLAB, Octave ou Python, et les formats cochés (DXF, STL, SVG, FEMM, Gmsh, CSV, HTML, scripts, rapport JSON, son WAV).",
"Writing (📝 button):
• Fill in the project sheet, entirely optional: title, subtitle, authors, organisation, INPI, other filings, licence, arXiv, DOI, GitHub, website, e-mail addresses, LinkedIn, keywords, abstract, notes. It is saved with the project.
• Choose the document language then “Generate the document”: fixed texts (headings, tables, method) are translated and numbers use the language's decimal separator. Title, subtitle, keywords, abstract and notes are entered per language: a field left empty takes the French text.
• Switch between the LaTeX source (editable) and the PDF preview. The PDF is compiled by LaTeX when installed, otherwise produced by the built-in generator. Every page carries the FractLab logo and version.
• Open the PDF, save .tex / .pdf, and ⎙ Print: choice of printer and number of copies.
Full save (💾 button): a dated folder receives the LaTeX and PDF document, the logo, the project data (JSON), a MATLAB save (.mat) readable by MATLAB, Octave or Python, and the ticked formats (DXF, STL, SVG, FEMM, Gmsh, CSV, HTML, scripts, JSON report, WAV sound).",
"Redacción (botón 📝):
• Rellene la ficha del proyecto, totalmente opcional: título, subtítulo, autores, organización, INPI, otros depósitos, licencia, arXiv, DOI, GitHub, sitio web, correos, LinkedIn, palabras clave, resumen, observaciones. Se guarda con el proyecto.
• Elija el idioma del documento y luego «Generar el documento»: los textos fijos (títulos, tablas, método) se traducen y los números usan el separador decimal del idioma. Título, subtítulo, palabras clave, resumen y observaciones se escriben para cada idioma: un campo vacío toma el texto francés.
• Alterne entre la fuente LaTeX (editable) y la vista previa PDF. El PDF se compila con LaTeX si está instalado; si no, lo produce el generador integrado. Cada página lleva el logotipo de FractLab y su versión.
• Abrir el PDF, guardar .tex / .pdf y ⎙ Imprimir: elección de impresora y número de copias.
Guardado general (botón 💾): una carpeta fechada recibe el documento LaTeX y PDF, el logotipo, los datos del proyecto (JSON), una copia MATLAB (.mat) legible por MATLAB, Octave o Python, y los formatos marcados (DXF, STL, SVG, FEMM, Gmsh, CSV, HTML, scripts, informe JSON, sonido WAV).",
"Redaktion (Schaltfläche 📝):
• Projektblatt ausfüllen, vollständig optional: Titel, Untertitel, Autoren, Organisation, INPI, weitere Hinterlegungen, Lizenz, arXiv, DOI, GitHub, Website, E-Mail-Adressen, LinkedIn, Schlüsselwörter, Zusammenfassung, Bemerkungen. Es wird mit dem Projekt gespeichert.
• Dokumentsprache wählen, dann „Dokument erzeugen“: feste Texte (Überschriften, Tabellen, Methode) werden übersetzt, Zahlen erhalten das Dezimaltrennzeichen der Sprache. Titel, Untertitel, Schlüsselwörter, Zusammenfassung und Bemerkungen werden je Sprache eingegeben: ein leeres Feld übernimmt den französischen Text.
• Zwischen LaTeX-Quelle (bearbeitbar) und PDF-Vorschau wechseln. Das PDF wird mit LaTeX kompiliert, falls installiert, sonst vom integrierten Generator erzeugt. Jede Seite trägt das FractLab-Logo und die Version.
• PDF öffnen, .tex / .pdf speichern und ⎙ Drucken: Wahl des Druckers und der Kopienzahl.
Gesamtsicherung (Schaltfläche 💾): Ein datierter Ordner erhält das LaTeX- und PDF-Dokument, das Logo, die Projektdaten (JSON), eine MATLAB-Sicherung (.mat), lesbar mit MATLAB, Octave oder Python, und die angekreuzten Formate (DXF, STL, SVG, FEMM, Gmsh, CSV, HTML, Skripte, JSON-Bericht, WAV-Ton).",
        ],
    },
    Section {
        icon: "🔄",
        title: ["Calculs dynamiques : tout se met à jour", "Dynamic computing: everything updates", "Cálculos dinámicos: todo se actualiza", "Dynamische Berechnung: alles aktualisiert sich"],
        body: [
"Chaque modification (paramètre, bouton « Calcul », Ajustement I.A, recalage, étude paramétrique) relance le calcul complet, réparti sur tous les cœurs du processeur (moins d'une seconde pour un moteur courant).
• Les vues directes (synthèse, géométrie, champ, cogging, FEM, forces, charge, saturation, moteur 3D) se mettent à jour aussitôt.
• Les vues plus lourdes se relancent seules dès que le projet est stable, si elles ont déjà été calculées une fois : comparaison des variantes, cartes 3D, validation, étude paramétrique. La simulation dynamique et l'excentricité suivent aussi, de même qu'un changement des réglages de la simulation.
• Le tableau de bord place en tête les critères non tenus, avec un bandeau d'alerte rouge. Un clic sur le bandeau ouvre un tableau qui donne, pour chaque critère, la valeur, le seuil, la cause et que faire, avec un bouton de correction quand elle est directe (ajuster l'arc, épaissir une culasse…). Le même bandeau apparaît dans chaque onglet, limité aux critères de cet onglet ; il disparaît quand tout est tenu. Une valeur sans sens physique (pic tonal au-dessus du bruit numérique) est signalée « non mesurable » au lieu d'un chiffre trompeur.",
"Every change (parameter, “Compute” button, A.I. tuning, calibration, parametric study) reruns the full computation, spread over all processor cores (under a second for a typical motor).
• Direct views (summary, geometry, field, cogging, back-EMF, forces, load, saturation, 3D motor) update at once.
• Heavier views rerun by themselves once the project is stable, if they were computed before: variant comparison, 3D maps, validation, parametric study. The dynamic simulation and eccentricity follow too, as does any change of the simulation settings.
• The dashboard lists the criteria that are not met first, under a red alert banner. Clicking the banner opens a table giving, for each criterion, the value, the threshold, the cause and what to do, with a fix button when a direct fix exists (adjust the arc, thicken a yoke…). The same banner appears in each tab, limited to that tab's criteria; it disappears when everything is met. A value with no physical meaning (tonal peak above numerical noise) is shown as “not measurable” rather than a misleading number.",
"Cada cambio (parámetro, botón «Cálculo», Ajuste I.A, calibración, estudio paramétrico) relanza el cálculo completo, repartido en todos los núcleos del procesador (menos de un segundo para un motor habitual).
• Las vistas directas (síntesis, geometría, campo, cogging, FCEM, fuerzas, carga, saturación, motor 3D) se actualizan al instante.
• Las vistas más pesadas se relanzan solas cuando el proyecto está estable, si ya se calcularon una vez: comparación de variantes, mapas 3D, validación, estudio paramétrico. La simulación dinámica y la excentricidad también siguen, al igual que un cambio de ajustes de la simulación.
• El tablero muestra primero los criterios no cumplidos, bajo un banner rojo de alerta. Un clic en el banner abre una tabla con, para cada criterio, el valor, el umbral, la causa y qué hacer, con un botón de corrección cuando es directa (ajustar el arco, engrosar un yugo…). El mismo banner aparece en cada pestaña, limitado a sus criterios; desaparece cuando todo se cumple. Un valor sin sentido físico (pico tonal por encima del ruido numérico) se indica «no medible» en lugar de una cifra engañosa.",
"Jede Änderung (Parameter, Schaltfläche „Berechnung“, KI-Optimierung, Kalibrierung, Parameterstudie) startet die vollständige Berechnung neu, verteilt auf alle Prozessorkerne (unter einer Sekunde für einen üblichen Motor).
• Direkte Ansichten (Übersicht, Geometrie, Feld, Rastmoment, EMK, Kräfte, Last, Sättigung, 3D-Motor) aktualisieren sich sofort.
• Aufwendigere Ansichten starten von selbst neu, sobald das Projekt stabil ist, falls sie schon einmal berechnet wurden: Variantenvergleich, 3D-Karten, Validierung, Parameterstudie. Die dynamische Simulation und die Exzentrizität folgen ebenfalls, ebenso bei geänderten Simulationseinstellungen.
• Das Dashboard zeigt nicht erfüllte Kriterien zuerst, unter einem roten Warnbanner. Ein Klick auf das Banner öffnet eine Tabelle mit Wert, Schwelle, Ursache und Abhilfe je Kriterium, mit einer Korrekturschaltfläche, wenn eine direkte Korrektur möglich ist (Bogen anpassen, Joch verstärken…). Dasselbe Banner erscheint in jedem Reiter, beschränkt auf dessen Kriterien; es verschwindet, wenn alles erfüllt ist. Ein Wert ohne physikalischen Sinn (Tonspitze über dem numerischen Rauschen) wird als „nicht messbar“ statt mit einer irreführenden Zahl angezeigt.",
        ],
    },
    Section {
        icon: "📈",
        title: ["Étude paramétrique (calcul en lot)", "Parametric study (batch computing)", "Estudio paramétrico (cálculo por lotes)", "Parameterstudie (Stapelberechnung)"],
        body: [
"L'onglet Paramétrique fait varier un paramètre entre deux bornes et recalcule le moteur complet à chaque valeur.
• Paramètres : arc des aimants, ouverture d'encoche, épaisseur des aimants, amplitude des décalages, becs de dents, entrefer, Br, longueur active, courant, angle du courant, spires par bobine, vitesse.
• Grandeurs : cogging crête-à-crête et efficace, force radiale max, FEM crête, THD, couple moyen, ondulation, inductance synchrone, pertes Joule, écart minimal entre aimants, KV. Cochez celles à tracer ; une courbe par grandeur, la valeur actuelle du projet est marquée d'un trait orange.
• Les points s'affichent dès qu'ils sont calculés. Le tableau donne toutes les valeurs ; « Appliquer » sur une ligne reporte cette valeur dans le projet. « Exporter CSV » enregistre le tableau (lisible par un tableur, MATLAB, Octave ou Python).
• L'étude se relance seule quand le projet change.",
"The Parametric tab varies one parameter between two bounds and recomputes the full motor at each value.
• Parameters: magnet arc, slot opening, magnet thickness, offset amplitude, tooth tips, air gap, Br, active length, current, current angle, turns per coil, speed.
• Quantities: peak-to-peak and RMS cogging, max radial force, peak back-EMF, THD, mean torque, ripple, synchronous inductance, copper losses, minimum magnet gap, KV. Tick the ones to plot; one curve per quantity, the current project value is marked by an orange line.
• Points appear as soon as they are computed. The table gives every value; “Apply” on a row copies that value into the project. “Export CSV” saves the table (readable by a spreadsheet, MATLAB, Octave or Python).
• The study reruns by itself when the project changes.",
"La pestaña Paramétrico hace variar un parámetro entre dos límites y recalcula el motor completo en cada valor.
• Parámetros: arco de imanes, apertura de ranura, espesor de imanes, amplitud de desfases, picos de diente, entrehierro, Br, longitud activa, corriente, ángulo de la corriente, espiras por bobina, velocidad.
• Magnitudes: cogging pico a pico y eficaz, fuerza radial máx., FCEM pico, THD, par medio, rizado, inductancia síncrona, pérdidas Joule, separación mínima entre imanes, KV. Marque las que desea trazar; una curva por magnitud, el valor actual del proyecto se marca con una línea naranja.
• Los puntos aparecen en cuanto se calculan. La tabla da todos los valores; «Aplicar» en una fila lleva ese valor al proyecto. «Exportar CSV» guarda la tabla (legible por una hoja de cálculo, MATLAB, Octave o Python).
• El estudio se relanza solo cuando cambia el proyecto.",
"Der Reiter Parametrisch variiert einen Parameter zwischen zwei Grenzen und berechnet für jeden Wert den gesamten Motor neu.
• Parameter: Magnetbogen, Nutöffnung, Magnetdicke, Versatzamplitude, Zahnköpfe, Luftspalt, Br, aktive Länge, Strom, Stromwinkel, Windungen je Spule, Drehzahl.
• Größen: Rastmoment Spitze-Spitze und effektiv, max. Radialkraft, Spitzen-EMK, THD, mittleres Moment, Welligkeit, Synchroninduktivität, Kupferverluste, minimaler Magnetabstand, KV. Die darzustellenden ankreuzen; eine Kurve je Größe, der aktuelle Projektwert ist mit einer orangen Linie markiert.
• Die Punkte erscheinen, sobald sie berechnet sind. Die Tabelle zeigt alle Werte; „Übernehmen“ in einer Zeile überträgt diesen Wert ins Projekt. „CSV exportieren“ speichert die Tabelle (lesbar mit Tabellenkalkulation, MATLAB, Octave oder Python).
• Die Studie startet bei Projektänderungen von selbst neu.",
        ],
    },
    Section {
        icon: "🔩",
        title: ["Saturation et matériaux du fer", "Saturation and iron materials", "Saturación y materiales del hierro", "Sättigung und Eisenwerkstoffe"],
        body: [
"Panneau « Matériaux du fer et saturation » : matériau du stator (tôles M270-35A, M400-50A, ou SMC pour les machines axiales), matériau de la culasse du rotor (acier massif 1010 par défaut), foisonnement des tôles.
Méthode du facteur de saturation : le modèle en sous-domaines (fer parfait) donne les inductions dans les dents et les culasses ; avec la courbe B(H) réelle du matériau, FractLab calcule la force magnétomotrice consommée par le fer et cherche le point où elle s'équilibre avec celle de l'entrefer. On obtient le facteur de saturation k_sat, la baisse de flux, la FEM et le couple corrigés, et la courbe couple / courant.
• Résultats dans l'onglet Charge et dans la carte « Saturation du fer » du tableau de bord (rouge si une partie dépasse nettement le coude de saturation).
• Les courbes B(H) sont typiques des catalogues : pour un matériau précis, les valeurs réelles peuvent différer.
• C'est une estimation d'ingénieur, non validée par éléments finis non linéaires.",
"“Iron materials and saturation” panel: stator material (M270-35A, M400-50A laminations, or SMC for axial machines), rotor yoke material (solid 1010 steel by default), lamination stacking factor.
Saturation-factor method: the subdomain model (perfect iron) gives the flux densities in the teeth and yokes; with the material's real B(H) curve, FractLab computes the magnetomotive force used by the iron and finds the point where it balances the air-gap one. This gives the saturation factor k_sat, the flux reduction, the corrected back-EMF and torque, and the torque / current curve.
• Results in the Load tab and in the “Iron saturation” card of the dashboard (red when a part is well beyond the saturation knee).
• The B(H) curves are typical catalogue curves: for a specific material the real values may differ.
• It is an engineering estimate, not validated by nonlinear finite elements.",
"Panel «Materiales del hierro y saturación»: material del estátor (chapas M270-35A, M400-50A, o SMC para máquinas axiales), material del yugo del rotor (acero macizo 1010 por defecto), factor de apilamiento.
Método del factor de saturación: el modelo por subdominios (hierro perfecto) da las inducciones en los dientes y los yugos; con la curva B(H) real del material, FractLab calcula la fuerza magnetomotriz consumida por el hierro y busca el punto en que se equilibra con la del entrehierro. Se obtiene el factor de saturación k_sat, la caída de flujo, la FCEM y el par corregidos, y la curva par / corriente.
• Resultados en la pestaña Carga y en la tarjeta «Saturación del hierro» del tablero (rojo si una parte supera claramente el codo de saturación).
• Las curvas B(H) son típicas de catálogo: para un material concreto los valores reales pueden diferir.
• Es una estimación de ingeniería, no validada por elementos finitos no lineales.",
"Leiste „Eisenwerkstoffe und Sättigung“: Statorwerkstoff (Bleche M270-35A, M400-50A oder SMC für Axialmaschinen), Rotorjoch-Werkstoff (standardmäßig massiver Stahl 1010), Stapelfaktor.
Sättigungsfaktor-Methode: das Teilgebietsmodell (ideales Eisen) liefert die Induktionen in Zähnen und Jochen; mit der realen B(H)-Kurve des Werkstoffs berechnet FractLab die vom Eisen verbrauchte magnetische Spannung und sucht den Punkt, an dem sie mit der des Luftspalts im Gleichgewicht ist. Man erhält den Sättigungsfaktor k_sat, die Flussminderung, korrigierte EMK und korrigiertes Moment sowie die Moment-/Strom-Kurve.
• Ergebnisse im Reiter Last und in der Karte „Eisensättigung“ des Dashboards (rot, wenn ein Teil deutlich über dem Sättigungsknick liegt).
• Die B(H)-Kurven sind typische Katalogkurven: für einen bestimmten Werkstoff können die realen Werte abweichen.
• Es handelt sich um eine Ingenieurabschätzung, nicht durch nichtlineare Finite Elemente validiert.",
        ],
    },
    Section {
        icon: "🎯",
        title: ["Recalage sur mesures du prototype", "Calibration on prototype measurements", "Calibración con medidas del prototipo", "Kalibrierung an Prototyp-Messungen"],
        body: [
"Onglet Validation, section « Recalage sur mesures du prototype » : le modèle est ajusté sur une mesure réelle.
• Importez une FEM de phase à vide (oscilloscope, moteur entraîné) ou un couple de détente mesuré : fichier CSV à deux colonnes (angle du rotor en degrés, ou temps en secondes avec la vitesse de la mesure ; valeur).
• Choisissez les paramètres à ajuster : Br réelle des aimants, entrefer réel, longueur effective (effets 3D). Br et longueur ne font que changer l'amplitude : un seul des deux est identifiable, FractLab interdit de les cocher ensemble. L'entrefer change aussi la forme d'onde et se distingue de Br.
• L'origine angulaire de la mesure est recalée automatiquement. L'écart (valeur efficace de la différence / valeur efficace de la mesure) s'affiche avant et après ; les courbes mesurée, initiale et recalée sont superposées.
• « Appliquer le recalage au projet » reporte les valeurs trouvées : tous les calculs suivants s'appuient alors sur le moteur réel.",
"Validation tab, “Calibration on prototype measurements” section: the model is fitted to a real measurement.
• Import a no-load phase back-EMF (oscilloscope, driven motor) or a measured cogging torque: two-column CSV file (rotor angle in degrees, or time in seconds with the measurement speed; value).
• Choose the parameters to adjust: actual magnet Br, actual air gap, effective length (3D effects). Br and length only change the amplitude: only one of them can be identified, so FractLab does not allow ticking both. The air gap also changes the waveform and can be told apart from Br.
• The angular origin of the measurement is aligned automatically. The deviation (RMS of the difference / RMS of the measurement) is shown before and after; measured, initial and calibrated curves are overlaid.
• “Apply the calibration to the project” copies the values found: every following computation then relies on the real motor.",
"Pestaña Validación, sección «Calibración con medidas del prototipo»: el modelo se ajusta a una medida real.
• Importe una FCEM de fase en vacío (osciloscopio, motor arrastrado) o un par de cogging medido: archivo CSV de dos columnas (ángulo del rotor en grados, o tiempo en segundos con la velocidad de la medida; valor).
• Elija los parámetros a ajustar: Br real de los imanes, entrehierro real, longitud efectiva (efectos 3D). Br y longitud solo cambian la amplitud: solo uno de los dos es identificable, FractLab impide marcarlos juntos. El entrehierro también cambia la forma de onda y se distingue de Br.
• El origen angular de la medida se alinea automáticamente. La desviación (valor eficaz de la diferencia / valor eficaz de la medida) se muestra antes y después; las curvas medida, inicial y calibrada se superponen.
• «Aplicar la calibración al proyecto» lleva los valores encontrados: todos los cálculos siguientes se basan entonces en el motor real.",
"Reiter Validierung, Abschnitt „Kalibrierung an Prototyp-Messungen“: das Modell wird an eine reale Messung angepasst.
• Eine Leerlauf-Phasen-EMK (Oszilloskop, geschleppter Motor) oder ein gemessenes Rastmoment importieren: CSV-Datei mit zwei Spalten (Rotorwinkel in Grad oder Zeit in Sekunden mit der Drehzahl der Messung; Wert).
• Die anzupassenden Parameter wählen: tatsächliches Br der Magnete, tatsächlicher Luftspalt, wirksame Länge (3D-Effekte). Br und Länge ändern nur die Amplitude: nur eines von beiden ist identifizierbar, FractLab erlaubt nicht, beide anzukreuzen. Der Luftspalt ändert auch die Kurvenform und ist von Br unterscheidbar.
• Der Winkelnullpunkt der Messung wird automatisch abgeglichen. Die Abweichung (Effektivwert der Differenz / Effektivwert der Messung) wird vorher und nachher angezeigt; gemessene, Ausgangs- und kalibrierte Kurve werden überlagert.
• „Kalibrierung ins Projekt übernehmen“ überträgt die gefundenen Werte: alle folgenden Berechnungen beruhen dann auf dem realen Motor.",
        ],
    },
    Section {
        icon: "⬇",
        title: ["Importer", "Import", "Importar", "Importieren"],
        body: [
"Menu Fichier > Importer :
• Angles des aimants (CSV) : un angle par ligne en degrés, ou « numéro ; angle ». Le nombre d'aimants doit être pair ; les positions deviennent personnalisées.
• Résultats de simulation externe (JSON) : reprend les angles d'aimants et superpose la courbe de couple au graphe de cogging.
• Mesures de banc (CSV, deux colonnes : angle en degrés ; couple en N·m, ou tension en V) : superposées aux courbes calculées de cogging ou de FEM pour comparer calcul et essai.
• Projet SIMUMSAP (.txt) : reprend les dimensions, les aimants et le bobinage d'un projet SIMUMSAP. Les machines à flux axial et à rotor intérieur sont transposées à la topologie de FractLab, avec un avertissement : les ordres de grandeur sont comparables, pas les valeurs exactes.
« Retirer les courbes importées » efface les superpositions.",
"File menu > Import:
• Magnet angles (CSV): one angle per line in degrees, or “number ; angle”. The number of magnets must be even; positions become custom.
• External simulation results (JSON): takes the magnet angles and overlays the torque curve on the cogging chart.
• Test-bench measurements (CSV, two columns: angle in degrees ; torque in N·m, or voltage in V): overlaid on the computed cogging or back-EMF curves to compare computation and test.
• SIMUMSAP project (.txt): takes the dimensions, magnets and winding of a SIMUMSAP project. Axial-flux and inner-rotor machines are transposed to the FractLab topology, with a warning: orders of magnitude are comparable, exact values are not.
“Remove imported curves” clears the overlays.",
"Menú Archivo > Importar:
• Ángulos de los imanes (CSV): un ángulo por línea en grados, o «número ; ángulo». El número de imanes debe ser par; las posiciones pasan a ser personalizadas.
• Resultados de simulación externa (JSON): toma los ángulos de los imanes y superpone la curva de par al gráfico de cogging.
• Medidas de banco (CSV, dos columnas: ángulo en grados ; par en N·m, o tensión en V): superpuestas a las curvas calculadas de cogging o FCEM para comparar cálculo y ensayo.
• Proyecto SIMUMSAP (.txt): toma las dimensiones, imanes y devanado de un proyecto SIMUMSAP. Las máquinas de flujo axial y de rotor interior se transponen a la topología de FractLab, con un aviso: los órdenes de magnitud son comparables, no los valores exactos.
«Quitar las curvas importadas» borra las superposiciones.",
"Menü Datei > Importieren:
• Magnetwinkel (CSV): ein Winkel pro Zeile in Grad oder „Nummer ; Winkel“. Die Magnetzahl muss gerade sein; die Positionen werden benutzerdefiniert.
• Externe Simulationsergebnisse (JSON): übernimmt die Magnetwinkel und legt die Momentkurve über das Rastmoment-Diagramm.
• Prüfstandsmessungen (CSV, zwei Spalten: Winkel in Grad ; Moment in N·m oder Spannung in V): über die berechneten Rastmoment- oder EMK-Kurven gelegt, um Rechnung und Versuch zu vergleichen.
• SIMUMSAP-Projekt (.txt): übernimmt Abmessungen, Magnete und Wicklung eines SIMUMSAP-Projekts. Axialfluss- und Innenläufermaschinen werden mit einem Hinweis auf die FractLab-Topologie übertragen: Größenordnungen sind vergleichbar, exakte Werte nicht.
„Importierte Kurven entfernen“ löscht die Überlagerungen.",
        ],
    },
    Section {
        icon: "⬆",
        title: ["Exporter", "Export", "Exportar", "Exportieren"],
        body: [
"Menu Fichier > Exporter :
Simulation (logiciels libres) :
• Modèle FEMM (.lua) : géométrie, matériaux et balayage du cogging, à lancer dans FEMM 4.2 (File > Open Lua Script) ; résultats dans fractlab_cogging.csv.
• Géométrie Gmsh (.geo) : régions nommées (fer stator, encoches, aimants N et S, cloche), maillable pour Elmer ou GetDP.
Fabrication :
• Coupe 2D (DXF) : calques STATOR, AIMANTS, ROTOR, pour la CAO ou la découpe.
• Gabarit de pose des aimants 1:1 (SVG) : imprimer à 100 % et vérifier la règle de 50 mm ; chaque aimant porte son numéro, sa polarité, son angle et son écart au pas régulier.
• Modèle 3D (STL) : stator, aimants et cloche, solides fermés en millimètres, pour l'impression 3D ou la CAO.
Rapports et données : rapport HTML autonome (graphes inclus), tableau LaTeX pour vos dossiers, scripts Octave/MATLAB et Python contenant les données et les tracés, courbes CSV, rapport JSON.",
"File menu > Export:
Simulation (free software):
• FEMM model (.lua): geometry, materials and cogging sweep, to run in FEMM 4.2 (File > Open Lua Script); results in fractlab_cogging.csv.
• Gmsh geometry (.geo): named regions (stator iron, slots, N and S magnets, bell), meshable for Elmer or GetDP.
Manufacturing:
• 2D cross-section (DXF): layers STATOR, AIMANTS, ROTOR, for CAD or cutting.
• 1:1 magnet placement template (SVG): print at 100 % and check the 50 mm ruler; each magnet carries its number, polarity, angle and offset from the regular pitch.
• 3D model (STL): stator, magnets and bell, closed solids in millimetres, for 3D printing or CAD.
Reports and data: standalone HTML report (charts included), LaTeX table for your documents, Octave/MATLAB and Python scripts containing data and plots, CSV curves, JSON report.",
"Menú Archivo > Exportar:
Simulación (software libre):
• Modelo FEMM (.lua): geometría, materiales y barrido de cogging, para ejecutar en FEMM 4.2 (File > Open Lua Script); resultados en fractlab_cogging.csv.
• Geometría Gmsh (.geo): regiones con nombre (hierro del estátor, ranuras, imanes N y S, campana), mallable para Elmer o GetDP.
Fabricación:
• Sección 2D (DXF): capas STATOR, AIMANTS, ROTOR, para CAD o corte.
• Plantilla de colocación de imanes 1:1 (SVG): imprimir al 100 % y comprobar la regla de 50 mm; cada imán lleva su número, polaridad, ángulo y desfase respecto al paso regular.
• Modelo 3D (STL): estátor, imanes y campana, sólidos cerrados en milímetros, para impresión 3D o CAD.
Informes y datos: informe HTML autónomo (gráficos incluidos), tabla LaTeX para sus documentos, scripts Octave/MATLAB y Python con datos y gráficos, curvas CSV, informe JSON.",
"Menü Datei > Exportieren:
Simulation (freie Software):
• FEMM-Modell (.lua): Geometrie, Materialien und Rastmoment-Sweep zum Start in FEMM 4.2 (File > Open Lua Script); Ergebnisse in fractlab_cogging.csv.
• Gmsh-Geometrie (.geo): benannte Bereiche (Statoreisen, Nuten, N- und S-Magnete, Glocke), vernetzbar für Elmer oder GetDP.
Fertigung:
• 2D-Schnitt (DXF): Ebenen STATOR, AIMANTS, ROTOR für CAD oder Zuschnitt.
• Magnet-Positionierschablone 1:1 (SVG): mit 100 % drucken und das 50-mm-Lineal prüfen; jeder Magnet trägt Nummer, Polarität, Winkel und Versatz zur regulären Teilung.
• 3D-Modell (STL): Stator, Magnete und Glocke als geschlossene Körper in Millimetern, für 3D-Druck oder CAD.
Berichte und Daten: eigenständiger HTML-Bericht (mit Diagrammen), LaTeX-Tabelle für Ihre Dokumente, Octave-/MATLAB- und Python-Skripte mit Daten und Plots, CSV-Kurven, JSON-Bericht.",
        ],
    },
    Section {
        icon: "⚙",
        title: ["Préférences, unités et affichage", "Preferences, units and display", "Preferencias, unidades y visualización", "Einstellungen, Einheiten und Anzeige"],
        body: [
"Menu Paramètres > Préférences et unités :
• Langue : français, anglais, espagnol, allemand (aussi en haut à droite).
• Thème sombre ou clair (aussi par le bouton ☀/🌙 en haut à droite).
• Taille de l'interface : Auto adapte la taille à la résolution de l'écran (pratique sur un portable), ou choisissez 80 % à 150 %.
• Unités : longueurs (mm, µm, cm, pouce, mil), couple (mN·m, N·m, N·cm, oz·in, lbf·in), force (N, mN, kgf, lbf), induction (T, mT, gauss), angles (°, rad, arcmin), vitesse (tr/min, rad/s, Hz), flux, tension, balourd. Les saisies, les cartes et les graphes suivent ce choix ; les calculs restent en unités SI.
Toutes les préférences sont mémorisées d'une session à l'autre.",
"Settings menu > Preferences and units:
• Language: French, English, Spanish, German (also at the top right).
• Dark or light theme (also with the ☀/🌙 button at the top right).
• Interface size: Auto adapts to the screen resolution (handy on a laptop), or choose 80 % to 150 %.
• Units: lengths (mm, µm, cm, inch, mil), torque (mN·m, N·m, N·cm, oz·in, lbf·in), force (N, mN, kgf, lbf), flux density (T, mT, gauss), angles (°, rad, arcmin), speed (rpm, rad/s, Hz), flux, voltage, unbalance. Inputs, cards and charts follow this choice; computations stay in SI units.
All preferences are remembered between sessions.",
"Menú Parámetros > Preferencias y unidades:
• Idioma: francés, inglés, español, alemán (también arriba a la derecha).
• Tema oscuro o claro (también con el botón ☀/🌙 arriba a la derecha).
• Tamaño de la interfaz: Auto se adapta a la resolución de la pantalla (práctico en un portátil), o elija de 80 % a 150 %.
• Unidades: longitudes (mm, µm, cm, pulgada, mil), par (mN·m, N·m, N·cm, oz·in, lbf·in), fuerza (N, mN, kgf, lbf), inducción (T, mT, gauss), ángulos (°, rad, arcmin), velocidad (tr/min, rad/s, Hz), flujo, tensión, desequilibrio. Las entradas, tarjetas y gráficos siguen esta elección; los cálculos permanecen en unidades SI.
Todas las preferencias se conservan de una sesión a otra.",
"Menü Einstellungen > Einstellungen und Einheiten:
• Sprache: Französisch, Englisch, Spanisch, Deutsch (auch oben rechts).
• Dunkles oder helles Design (auch über ☀/🌙 oben rechts).
• Oberflächengröße: Auto passt sich der Bildschirmauflösung an (praktisch am Laptop), oder 80 % bis 150 % wählen.
• Einheiten: Längen (mm, µm, cm, Zoll, mil), Drehmoment (mN·m, N·m, N·cm, oz·in, lbf·in), Kraft (N, mN, kgf, lbf), Induktion (T, mT, Gauß), Winkel (°, rad, arcmin), Drehzahl (U/min, rad/s, Hz), Fluss, Spannung, Unwucht. Eingaben, Karten und Diagramme folgen dieser Wahl; gerechnet wird in SI-Einheiten.
Alle Einstellungen bleiben zwischen Sitzungen erhalten.",
        ],
    },
    Section {
        icon: "ℹ",
        title: ["Hypothèses et limites du modèle", "Model assumptions and limits", "Hipótesis y límites del modelo", "Modellannahmen und Grenzen"],
        body: [
"Méthode : solution formelle des équations de Maxwell en magnétostatique 2D, par sous-domaines en coordonnées polaires exactes (encoches, entrefer, aimants), avec des séries de Fourier sur 360° complets. Toute répartition d'aimants est donc admise, y compris apériodique.
Hypothèses de cette version :
• fer linéaire de perméabilité infinie (pas de saturation) ;
• aimants de perméabilité uniforme, induction d'entrefer calculée en 2D par tronçon (multi-tranches pour le flux axial) ;
• inductances hors têtes de bobines ; simulation dynamique à inductance constante ;
• pertes, frottements, thermique, vibration et son : modèles d'ingénierie aux coefficients réglables.
Validé contre un solveur volumes finis indépendant (encoches semi-fermées, rotor intérieur, courants : écarts inférieurs à 1,1 %) et comparé aux résultats de SIMUMSAP.",
"Method: formal solution of Maxwell's equations in 2D magnetostatics, by subdomains in exact polar coordinates (slots, air gap, magnets), with Fourier series over the full 360°. Any magnet layout is therefore allowed, including aperiodic ones.
Assumptions of this version:
• linear iron of infinite permeability (no saturation);
• magnets of uniform permeability, air-gap field computed in 2D per section (multi-slice for axial flux);
• inductances without end windings; dynamic simulation with constant inductance;
• losses, friction, thermal, vibration and sound: engineering models with adjustable coefficients.
Validated against an independent finite-volume solver (semi-closed slots, inner rotor, currents: deviations below 1.1 %) and compared with SIMUMSAP results.",
"Método: solución formal de las ecuaciones de Maxwell en magnetostática 2D, por subdominios en coordenadas polares exactas (ranuras, entrehierro, imanes), con series de Fourier sobre 360° completos. Se admite por tanto cualquier distribución de imanes, incluso aperiódica.
Hipótesis de esta versión:
• hierro lineal de permeabilidad infinita (sin saturación);
• imanes de permeabilidad uniforme, campo de entrehierro calculado en 2D por tramo (multicapa para el flujo axial);
• inductancias sin cabezas de bobina; simulación dinámica con inductancia constante;
• pérdidas, rozamientos, térmica, vibración y sonido: modelos de ingeniería con coeficientes ajustables.
Validado con un solver de volúmenes finitos independiente (ranuras semicerradas, rotor interior, corrientes: desviaciones inferiores al 1,1 %) y comparado con los resultados de SIMUMSAP.",
"Methode: formale Lösung der Maxwell-Gleichungen in 2D-Magnetostatik mit Teilgebieten in exakten Polarkoordinaten (Nuten, Luftspalt, Magnete) und Fourier-Reihen über volle 360°. Damit ist jede Magnetverteilung zulässig, auch aperiodisch.
Annahmen dieser Version:
• lineares Eisen mit unendlicher Permeabilität (keine Sättigung);
• Magnete mit einheitlicher Permeabilität, Luftspaltfeld in 2D je Abschnitt berechnet (Mehrschicht für Axialfluss);
• Induktivitäten ohne Wickelköpfe; dynamische Simulation mit konstanter Induktivität;
• Verluste, Reibung, Thermik, Schwingung und Geräusch: Ingenieurmodelle mit einstellbaren Koeffizienten.
Validiert mit einem unabhängigen Finite-Volumen-Löser (halbgeschlossene Nuten, Innenläufer, Ströme: Abweichungen unter 1,1 %) und mit SIMUMSAP-Ergebnissen verglichen.",
        ],
    },
    Section {
        icon: "📖",
        title: ["Glossaire", "Glossary", "Glosario", "Glossar"],
        body: [
"• Cogging (couple de détente) : couple dû à l'attraction entre aimants et dents, présent moteur éteint.
• FEM : force électromotrice induite dans les bobinages quand le rotor tourne.
• THD : taux de distorsion harmonique de la FEM (harmoniques multiples de la fondamentale).
• Interharmoniques : raies hors multiples de la fondamentale, créées par une répartition apériodique.
• Part de la dominante : part de l'énergie spectrale portée par la raie la plus forte.
• Entropie spectrale : mesure de l'étalement du spectre (plus haute = énergie plus répartie).
• Balourd : déséquilibre massique du rotor (mg·cm).
• Autocorrélation : ressemblance de la suite des écarts avec elle-même décalée (plus basse = plus apériodique).
• Étoile des encoches : méthode qui attribue chaque bobine à une phase selon son angle électrique.",
"• Cogging torque: torque due to the attraction between magnets and teeth, present with the motor switched off.
• Back-EMF: electromotive force induced in the windings when the rotor turns.
• THD: total harmonic distortion of the back-EMF (harmonics that are multiples of the fundamental).
• Interharmonics: lines off multiples of the fundamental, created by an aperiodic layout.
• Dominant share: fraction of the spectral energy carried by the strongest line.
• Spectral entropy: measure of how spread the spectrum is (higher = energy more distributed).
• Unbalance: mass imbalance of the rotor (mg·cm).
• Autocorrelation: similarity of the offset sequence with a shifted copy of itself (lower = more aperiodic).
• Star of slots: method assigning each coil to a phase according to its electrical angle.",
"• Cogging (par de retención): par debido a la atracción entre imanes y dientes, presente con el motor apagado.
• FCEM: fuerza contraelectromotriz inducida en los devanados cuando gira el rotor.
• THD: tasa de distorsión armónica de la FCEM (armónicos múltiplos de la fundamental).
• Interarmónicos: rayas fuera de los múltiplos de la fundamental, creadas por una distribución aperiódica.
• Peso del dominante: fracción de la energía espectral de la raya más fuerte.
• Entropía espectral: medida de la dispersión del espectro (más alta = energía más repartida).
• Desequilibrio: desequilibrio de masa del rotor (mg·cm).
• Autocorrelación: parecido de la secuencia de desfases con ella misma desplazada (más baja = más aperiódica).
• Estrella de ranuras: método que asigna cada bobina a una fase según su ángulo eléctrico.",
"• Rastmoment: Moment durch die Anziehung zwischen Magneten und Zähnen, auch bei abgeschaltetem Motor vorhanden.
• Gegen-EMK: in den Wicklungen induzierte Spannung bei drehendem Rotor.
• THD: Klirrfaktor der Gegen-EMK (Harmonische als Vielfache der Grundschwingung).
• Zwischenharmonische: Linien außerhalb der Vielfachen der Grundschwingung, entstehen durch aperiodische Verteilung.
• Anteil der Dominanten: Anteil der Spektralenergie in der stärksten Linie.
• Spektrale Entropie: Maß für die Breite des Spektrums (höher = Energie stärker verteilt).
• Unwucht: Massenunwucht des Rotors (mg·cm).
• Autokorrelation: Ähnlichkeit der Versatzfolge mit sich selbst verschoben (niedriger = aperiodischer).
• Nutenstern: Verfahren, das jede Spule nach ihrem elektrischen Winkel einer Phase zuordnet.",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toutes_les_sections_traduites() {
        for s in SECTIONS {
            for l in 0..4 {
                assert!(!s.title[l].is_empty() && s.body[l].len() > 80, "{}", s.title[0]);
            }
        }
        assert!(SECTIONS.len() >= 12);
    }
}
