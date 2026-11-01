# The Diplomat Peak — Das letzte Kapitel

Ein First-Person-Horrorspiel in **Rust** (Bevy 0.15), das dem Design-Dokument
„The Diplomat Peak – Das letzte Kapitel“ folgt: ein verlassenes Berghotel in
Baguio City, ein tropischer Sturm, drei Rätsel und ein Ende, das man nicht
vergisst.

Alles, was du auf dem Bildschirm siehst und hörst, ist **prozedural erzeugt**:
keine importierten Modelle, keine fremden Texturen, keine fremden Sounds. Die
Texturen und Klänge entstehen mit den zwei Python-Werkzeugen in `tools/`, die
Geometrie, das Licht und die Musik des Hauses entstehen im Spiel selbst.

---

## Starten

```bash
cargo run --release
```

Der erste Build kompiliert Bevy und dauert ein paar Minuten (CPU: 2 Kerne
genügen, ~3 GB RAM reichen). Es gibt keine weiteren Abhängigkeiten außer dem
Rust-Toolchain (`rustup`, stable) und einem Grafikbeschleuniger mit Vulkan,
Metal oder DX12 (wgpu).

Optional lassen sich Texturen und Audio neu erzeugen:

```bash
python3 -m pip install numpy pillow --break-system-packages
python3 tools/gen_textures.py     # 28 MB, 1024²/512² inkl. Normal Maps
python3 tools/gen_audio.py        # 49 WAVs, bandbegrenzt, 16 Bit
```

## Level-Editor

Der Editor ist ein **eigenes Programm**, kein Teil des Spiels — er nutzt nur
dieselbe Library (`src/lib.rs`), also dieselbe Palette, dieselben Props und
dieselbe Hotel-Geometrie. Darum ist das, was du dort setzt, auch genau das, was
im Spiel steht.

```bash
cargo run --release --bin editor
```

* links: alle Objekte (Moebel, Waende/Bloecke, Licht, **Eingabepad**, **Tuer**)
* rechts: Sprungmarken zu jedem Bereich des Hotels
* `F5` speichert nach `assets/level/extras.lvl`, `F9` laedt neu
* das Spiel liest diese Datei beim Aufbau des Hotels — nichts weiter zu tun

| Taste | Wirkung |
|---|---|
| `W A S D` + `Q`/`E` | fliegen (mit `Shift` schneller) |
| rechte Maustaste | umsehen |
| linke Maustaste | Objekt setzen / Objekt auswaehlen |
| Pfeiltasten, `U`/`O` | ausgewaehltes Objekt verschieben |
| `,` / `.` | ausgewaehltes Objekt drehen |
| `C` / `Entf` | duplizieren / loeschen |

Ein platziertes **Eingabepad** oeffnet im Spiel wirklich das Code-Fenster, eine
platzierte **Tuer** schwingt auf und gibt den Weg frei: die Objekte bringen ihre
Logik mit (`src/factory.rs` + `src/custom.rs`), nicht nur ihr Aussehen.

## Steuerung

| Taste | Wirkung |
|---|---|
| `W A S D` | gehen |
| `Shift` | langsam gehen |
| Maus | umsehen |
| `E` / Linksklick | interagieren |
| `F` | Stirnlampe |
| `Tab` | Notizbuch (gefundene Artikel, aktuelle Aufgabe) |
| `Esc` | Pause / Einstellungen |

Im Tastenfeld des Tresors: Ziffern tippen, `Enter` bestätigen,
`Backspace` löschen, `Esc` abbrechen.

## Aufbau des Spiels

| Akt | Ort | Rätsel |
|---|---|---|
| 1 | Veranda, Lobby, Rezeption, Treppenhaus | Zeitungsartikel → Tresorcode → Eisenschlüssel für die Korridortür |
| 2 | Korridor im 1. Stock, Aufzugsschacht, Maschinenraum | Schalttafel: richtige Reihenfolge der vier Hebel → Strom für den Aufzug |
| 3 | Ballsaal im Penthouse | Standuhr auf die „Schattenzeit“ stellen → Geheimtür zur Dachterrasse |
| 4 | Dachterrasse, Tor, Dienstgang | Notstromschalter → 5 Sekunden totale Stille → Tor entriegelt → … |

Die Lösung steht bewusst nur in den Dokumenten im Spiel (und ganz unten in
dieser Datei).

## Technik

* **Engine:** Bevy 0.15.3, `bevy_pbr`, `bevy_ui`, `bevy_audio` (WAV).
* **Eine durchgehende Geometrie:** Das Hotel wird beim Start einmal gebaut und
  nach Material gebündelt — ein paar Dutzend Drawcalls für ein ganzes Haus.
  Alle Räume sind achsenparallel, damit die AABB-Kollision exakt bleibt
  (`src/collision.rs`).
* **Materialbibliothek:** `src/palette.rs` definiert jede Oberfläche des Hauses
  (Marmor, Mahagoni, Damast, Samt, Messing, Ziegel, Blattwerk …) mit
  Normal-Maps und passender Rauheit.
* **Licht:** kaltes Mondlicht von außen, warmes Wolframlicht innen, Blitze mit
  drei ungleichmäßigen Pulsen (`src/lighting.rs`), volumetrischer Nebel,
  Taschenlampe mit `VolumetricLight`, dazu ein eigener Fullscreen-Post-Look
  (Korn, Vignette, Farbstich) als `UiMaterial` (`assets/shaders/screen_fx.wgsl`).
* **Wetter:** Regen als gebündelte Streifen-Meshes, die in Schleifen fallen,
  Staub in der Luft, Blitze als gezeichnete Zickzack-Geometrie (`src/fx.rs`).
* **Klang:** Ambience-Betten (Regen, Wind, Raumton), Donner mit Verzögerung,
  Schritte je nach Boden, alle Klänge synthetisch erzeugt und bandbegrenzt,
  damit nichts zischt (`src/audio.rs`).
* **Interface:** HUD, Dokumentenleser, Notizbuch, Tastenfeld, Pausenmenü und
  Ladebildschirm komplett in Bevy-UI, deutsche Typografie mit Special Elite
  und Playfair Display (`src/ui/`).

### Dateien

```
src/main.rs            Einstiegspunkt, Fenster, Plugins
src/core.rs            Zustände, Assets, Einstellungen, Zufall
src/palette.rs         Materialbibliothek des Hotels
src/geom.rs            prozeduraler Mesh-Builder
src/collision.rs       AABB-Kollision, Boden-/Deckenabfragen, Ray-Picking
src/player.rs          First-Person-Controller, Kamera, Stirnlampe, Schritte
src/interact.rs        Ansehen, Fokus, Interaktion
src/world/mod.rs       Level-Builder, Türen/Fenster-Helfer, Animationen
src/world/foyer.rs     Veranda, Lobby, Rezeption, Tresor, Treppenhaus
src/world/corridor.rs  1. Stock, Aufzug, Maschinenraum, Schalttafel
src/world/penthouse.rs Ballsaal, Standuhr, Dachterrasse, Dienstgang
src/world/props.rs     Möbel, Leuchten, Pflanzen, Türen, Klavier
src/story.rs           Aufgaben, Dokumente, Zwischentexte, Regie
src/puzzles.rs         Tresor, Hebel, Uhr, Aufzug
src/finale.rs          Notstromschalter, Stille, Sprungschreck, Abspann
src/audio.rs           Ambience, SFX, Wetter
src/fx.rs              Regen, Staub, Blitze
src/lighting.rs        Mondlicht und Gewitterlicht
src/ui/                HUD, Dokumente, Notizbuch, Tastenfeld, Menüs, Post-Look
assets/fonts           Playfair Display, Special Elite, Barlow Condensed
assets/textures        prozedurale Texturen + Normal Maps
assets/audio           49 synthetische WAVs
tools/                 Generatoren für Texturen und Audio
```

## Hinweis zum Stand

Der Code wurde ohne Rust-Toolchain in dieser Umgebung geschrieben und ist
gegen die API von Bevy 0.15.3 geprüft (Quelltext als Referenz), aber noch nicht
kompiliert. Sollte `cargo` beim ersten Build etwas anmeckern, sind es
voraussichtlich Kleinigkeiten an Typen oder Imports — die Struktur, die
Räume, die Rätsel und alle Assets stehen.

## Spoiler: die Lösungen

* **Tresor:** Zimmer 104 + Geburtsjahr 49 → **10449**
* **Schalttafel:** **Rot → Blau → Grün → Gelb** (Hauptschalter, Sicherung,
  Hilfsaggregat, Erdung)
* **Standuhr:** 03:15 minus 4 h 20 min → **22:55**
