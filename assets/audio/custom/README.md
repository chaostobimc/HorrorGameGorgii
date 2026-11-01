# Eigene Klänge — Regen & Gewitter

Dieser Ordner ist der Austauschplatz für die Wetterklänge. Solange er leer ist,
benutzt das Spiel die generierten Dateien aus `assets/audio/`. Legst du hier
eine Datei mit passendem Namen ab, gewinnt sie — ohne eine Zeile Code zu ändern.

## Aufnahme aus einem Video holen

Wenn `yt-dlp` (und am besten `ffmpeg`) installiert sind:

```bash
yt-dlp -x --audio-format mp3 -o "gewitter.%(ext)s" "https://www.youtube.com/watch?v=y4h_4NIOxuY"
```

Ohne `ffmpeg` liefert yt-dlp `.webm` oder `.m4a` — **die kann Bevy nicht lesen**.
Dann einmal umwandeln, z. B. mit VLC (Medien → Konvertieren/Speichern → Audio –
MP3) oder mit Audacity (Datei → Exportieren → Als MP3/WAV exportieren).

## Welche Datei wohin

| Datei | Im Spiel |
|---|---|
| `rain.mp3` | Dauerregen, läuft in Schleife im ganzen Haus |
| `wind.mp3` | Wind, läuft in Schleife |
| `thunder_far_1.mp3` … `thunder_far_3.mp3` | fernes Grollen, zufällig |
| `thunder_near_1.mp3`, `thunder_near_2.mp3` | naher Schlag, zufällig |

Endungen, die funktionieren: `.wav`, `.mp3`, `.ogg`, `.oga`, `.flac`.
Liegen mehrere Varianten da, gewinnt in dieser Reihenfolge: wav, mp3, ogg,
oga, flac.

Fehlt eine Datei, wird für genau diesen Klang weiter die generierte benutzt —
du kannst also auch nur den Regen tauschen und die Donner behalten.

## Zwei Dinge, die du wissen solltest

* **Ausschnitte statt Komplettaufnahme.** Bevy lädt die Datei vollständig in den
  Speicher: 2 Minuten MP3 sind ≈ 2 MB, 2 Minuten WAV ≈ 21 MB, drei Stunden WAV
  schon ≈ 1,9 GB. Für den Regen reichen 60–120 s völlig, weil er ohnehin
  schleift.
* **Saubere Schleife.** Nimm für `rain` einen Abschnitt mit *durchgehendem*
  Regen (kein Donnerschlag, keine Pause am Rand), dann hört man den Übergang
  nicht. Für die Donner sind Einzelaufnahmen mit ein paar Sekunden Abstand
  davor/danach ideal — sie werden als Einmal-Klang abgespielt.

## Prüfen, ob es greift

Beim Start schreibt das Spiel in die Konsole:

```
[audio] eigener Klang aktiv: assets/audio/custom/rain.mp3
```

Erscheint die Zeile nicht, stimmt der Dateiname oder die Endung nicht (oder das
Programm läuft nicht im Projektordner — die Datei wird relativ zum
Arbeitsverzeichnis gesucht).
