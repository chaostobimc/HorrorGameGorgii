# Eigene 3D-Modelle (Blender)

Alles, was hier als `.glb` oder `.gltf` liegt, erscheint im Level-Editor in der
Liste **MODELLE** und lässt sich mit einem Klick in die Map setzen — mit
automatischer Kollision.

`room_example.glb` liegt schon hier: ein hohler Raum (6 × 3 m) mit Türdurchgang
und einer Wandstärke von 20 cm. Setz ihn in der Lobby ab und lauf hinein — innen
ist es leer, nur die Wände sind fest. Genau dafür ist die Kollision da.

## Aus Blender exportieren

1. **Vorbereiten** — Modell auf den Ursprung setzen, Ursprung an die Stelle, wo
   es im Spiel stehen soll (bei Möbeln: Fußbodenmitte).
   `Objekt → Anwenden → Alle Transformationen` (Object → Apply → All Transforms).
   Das ist wichtig: die Kollision wird aus den Meshdaten gebaut, nicht aus den
   Node-Transformationen.
2. **Einheiten** — 1 Blender-Einheit = 1 Meter, `+Y` oben (Blender-Vorgabe,
   glTF macht das automatisch richtig).
3. **Exportieren** — `Datei → Exportieren → glTF 2.0 (.glb)`:
   * Format: **glTF Binary (.glb)**
   * Include: Selected Objects (oder die ganze Szene)
   * Transform: `+Y Up` ✔
   * Data → Modifiers: anwenden
4. Datei hierher kopieren, Editor neu starten — fertig.

## Wie die Kollision entsteht

Der Editor tastet die Dreiecke des Modells in einem Raster ab (Voxel). Jede
Zelle, in der Geometrie liegt, wird ein Kasten-Collider; benachbarte Zellen
werden zu möglichst wenigen Kästen zusammengefasst.

Folgen daraus:

* **Hohl ist hohl.** Ein Raum mit Wänden ist innen begehbar, weil innen keine
  Dreiecke liegen. Es gibt keinen unsichtbaren Würfel um das Modell.
* **Wände brauchen Dicke** (oder eine Fläche — eine einzelne Ebene ergibt eine
  Lage Kollision, das reicht zum Anlehnen, aber nicht zum Durchqueren).
* **Dünne Deko** (Blätter, Kabel, Kabelbinder) wird mit-kollidiert. Wenn etwas
  nicht blockieren soll: im Editor den Wert **Kollision (m)** hochsetzen (grob
  = weniger Kästen) oder die Datei ohne Kollision platzieren
  (`solid=0` in der Level-Datei).
* Die Rastergröße ist der Wert **Kollision (m)** in den Eigenschaften: 0.2 für
  Möbel, 0.3–0.35 für große Räume (schneller, weniger Collider).

## Grenzen

* Keine `.blend`, kein `.fbx`, kein `.obj` — nur glTF (das kann Blender,
  und Bevy liest es ohne Zusatzmodul).
* Keine Animationen: das Spiel bewegt Türen und Requisiten selbst. Ein Modell,
  das eine Tür ersetzen soll, gehört deshalb auf das **Türblatt** (Teil), nicht
  auf die ganze Tür — dann schwingt es mit.
* Materialien: PBR aus glTF (Farbe, Rauheit, Metall, Texturen) wird übernommen.
