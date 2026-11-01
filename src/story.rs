//! Story state, objectives, subtitles and everything Gorgii can read.

use bevy::prelude::*;

use crate::audio::PlaySfx;
use crate::core::{AppState, GameAssets};
use crate::interact::Focus;
use crate::player::Player;
use crate::world::{EntryDoor, Swing};

// ---------------------------------------------------------------------------
// documents
// ---------------------------------------------------------------------------

pub struct Document {
    pub source: &'static str,
    pub title: &'static str,
    pub body: &'static str,
}

pub const DOCUMENTS: [Document; 5] = [
    Document {
        source: "Manila Daily Bulletin · 14. August 1988",
        title: "Gäste flohen in der Sturmnacht",
        body: "BAGUIO. In der Nacht des 12. August verließen mehr als vierzig Gäste das \
Berghotel „The Diplomat Peak“ innerhalb weniger Stunden. Augenzeugen berichten von \
Schreien im dritten Stock und von einem Sturm, der die Telefonleitungen kappte. Der \
Direktor des Hauses, Eduardo Ramirez, geboren 1949, wird seit dieser Nacht vermisst. \
Sein Zimmer, die 104, war leer; im Bett lag noch die nasse Uniform eines Pagen. Die \
Polizei fand den Safe des Direktors ungeöffnet vor. Ramirez, so heißt es im Personal, \
habe ihn niemals mit einem Schlüssel geöffnet, sondern stets mit seiner Zimmernummer \
und den letzten beiden Ziffern seines Geburtsjahres. Die Ermittlungen wurden im \
Oktober desselben Jahres eingestellt.",
    },
    Document {
        source: "Baguio Herald · 03. November 1991",
        title: "Das Licht kam nie zurück",
        body: "Die technische Prüfung des Hotels „The Diplomat Peak“ hat ergeben, dass die \
Schalttafel im Maschinenraum unsachgemäß verkabelt wurde. Der rote Hauptschalter muss \
vor dem blauen Sicherungshebel gedrückt werden – andernfalls fällt das gesamte Netz. \
Der grüne Hebel des Hilfsaggregats darf niemals zuerst gelegt werden, sondern steht in \
der Kette immer an dritter Stelle. Den Abschluss bildet der gelbe Erdungsschalter, der \
keinerlei Strom führt, aber die Anlage erst freigibt, wenn alles andere geschlossen ist. \
Der Aufzug ins Penthouse lief zuletzt 1988 – mit einem Ruck, der im ganzen Haus zu \
hören war.",
    },
    Document {
        source: "Tagesspiegel Baguio · 20. September 1988",
        title: "Die Uhr, die vorausläuft",
        body: "Aus dem Nachlass des Küchenchefs Mateo Villanueva. Der Koch notierte über \
Jahre die Merkwürdigkeiten des Hauses. „Jede Nacht, wenn die Kirchturmuhr im Tal genau \
03:15 schlägt, bleiben alle Uhren im Penthouse stehen. Sie zeigen dann 07:35 – vier \
Stunden und zwanzig Minuten voraus. Niemand hat sie je zurückgestellt. Und die \
Zahnräder der großen Standuhr drehen sich nur, wenn man ihre Zeiger auf jene Stunde \
stellt, die der 03:15 um genau diese vier Stunden und zwanzig Minuten vorausgeht. \
Dann öffnet sich die Wand hinter ihr.“",
    },
    Document {
        source: "Tagebuch · Mateo Villanueva, Küchenchef",
        title: "Der Dienstgang",
        body: "12. August. Der Sturm hat die Terrassentür aus den Angeln gedrückt. Ich habe \
sie mit Draht befestigt; der Weg zum Notausgang führt ohnehin nur über den Dienstgang \
hinter der Uhr. Ramirez hat den Schlüssel zum Torgenerator an der Brüstung befestigt, \
damit ihn im Dunkeln jeder findet. Er sagt, das Haus sei sicher. Er sagt das jeden \
Abend, während er die Treppe zur 104 hinaufgeht und die Tür hinter sich abschließt.\n\n\
13. August. Um 03:15 hat die Uhr im Tal geschlagen. Die Standuhr im Ballsaal steht \
seither still, wie immer. Ich habe die Zeiger nicht angefasst. Ich schwöre, ich habe \
sie nicht angefasst.\n\n\
Die Küche ist geschlossen. Das Hotel ist geschlossen. Ich schreibe das hier, während \
unten im Foyer jemand gegen die Flügeltür klopft – dreimal, dann Pause, dann dreimal. \
Es ist niemand da.",
    },
    Document {
        source: "Gästebuch · The Diplomat Peak Resort",
        title: "Eintragungen 1988",
        body: "4. August – „Ein Haus wie aus einem Traum. Der Blick vom Penthouse ist \
unbezahlbar.“ Familie Krüger, Hamburg.\n\n\
9. August – „Der Portier hat uns gewarnt, nachts nicht auf die Terrasse zu gehen. Wir \
sind trotzdem gegangen. Es war wunderschön und vollkommen still.“ A. Moreau, Lyon.\n\n\
11. August – „Das Klavier im Ballsaal spielt von selbst. Nicht schön, aber es spielt.“ \
Ohne Namen.\n\n\
12. August – „Wir reisen ab. Sofort.“ Ohne Namen.\n\n\
Die nächste Seite ist leer. Darunter, in anderer Handschrift, sehr klein: „Es hat \
gewartet, bis alle fort waren. Ich zähle die Stunden.“",
    },
];

// ---------------------------------------------------------------------------
// state
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct Progress {
    pub notes: Vec<usize>,
    pub safe_open: bool,
    pub has_key: bool,
    pub corridor_open: bool,
    pub generator: bool,
    pub reached_penthouse: bool,
    pub clock_hour: u8,
    pub clock_minute: u8,
    pub clock_solved: bool,
    pub gate_open: bool,
    pub ended: bool,
    pub levers: Vec<u8>,
    /// Which narrative beats already played.
    pub scene: u8,
    pub started: f32,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            notes: Vec::new(),
            safe_open: false,
            has_key: false,
            corridor_open: false,
            generator: false,
            reached_penthouse: false,
            // the clock is frozen at 03:15, exactly as the article says
            clock_hour: 3,
            clock_minute: 15,
            clock_solved: false,
            gate_open: false,
            ended: false,
            levers: Vec::new(),
            scene: 0,
            started: 0.0,
        }
    }
}

impl Progress {
    pub fn note(&mut self, index: usize) {
        if !self.notes.contains(&index) {
            self.notes.push(index);
            self.notes.sort_unstable();
        }
    }

    pub fn objective(&self) -> &'static str {
        if !self.corridor_open {
            if self.safe_open {
                "Nimm den Schlüssel aus dem Tresen."
            } else {
                "Durchsuche die Rezeption nach einem Weg in den Korridor."
            }
        } else if !self.generator {
            "Starte im Maschinenraum den Strom für den Aufzug."
        } else if !self.reached_penthouse {
            "Rufe den Aufzug und fahre ins Penthouse."
        } else if !self.clock_solved {
            "Stelle die Standuhr im Ballsaal auf die Schattenzeit."
        } else if !self.gate_open {
            "Öffne auf der Dachterrasse das Tor."
        } else {
            "Verlasse das Hotel."
        }
    }

    /// The hands as they would read on the dial.
    pub fn clock_angle_hour(&self) -> f32 {
        let h = (self.clock_hour % 12) as f32;
        let m = self.clock_minute as f32;
        (h + m / 60.0) * 30.0
    }

    pub fn clock_angle_minute(&self) -> f32 {
        self.clock_minute as f32 * 6.0
    }

    pub fn clock_is_solution(&self) -> bool {
        (self.clock_hour == 22 || self.clock_hour == 10) && self.clock_minute == 55
    }
}

#[derive(Resource, Default)]
pub struct Subtitle {
    pub text: String,
    pub timer: f32,
    /// Length of the current line, used to fade it in.
    pub total: f32,
}

impl Subtitle {
    pub fn say(&mut self, text: &str, secs: f32) {
        self.text = text.to_string();
        self.timer = secs;
        self.total = secs.max(0.5);
    }
}

// ---------------------------------------------------------------------------
// plugin
// ---------------------------------------------------------------------------

pub struct StoryPlugin;

impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Progress>()
            .init_resource::<Subtitle>()
            .add_systems(OnEnter(AppState::Playing), reset_story)
            .add_systems(
                Update,
                (tick_subtitle, intro_sequence, area_beats, whisper_hint)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn reset_story(
    mut progress: ResMut<Progress>,
    mut sub: ResMut<Subtitle>,
    time: Res<Time>,
    mut ui: ResMut<crate::core::UiState>,
    mut virt: ResMut<Time<bevy::time::Virtual>>,
) {
    // Entering play always starts from a clean input state: no modal page, no
    // cinematic lock and a running clock. A stale flag here would silently
    // kill mouse look and movement.
    ui.overlay = crate::core::Overlay::None;
    ui.locked = false;
    ui.open_doc = 0;
    if virt.is_paused() {
        virt.unpause();
    }
    *progress = Progress::default();
    progress.started = time.elapsed_secs();
    sub.say(
        "23:45. Der Sturm hat die Zufahrt verschluckt. Nur dieses Haus steht noch.",
        6.0,
    );
}

fn tick_subtitle(time: Res<Time>, mut sub: ResMut<Subtitle>) {
    if sub.timer > 0.0 {
        sub.timer = (sub.timer - time.delta_secs()).max(0.0);
    }
}

/// The front doors fall shut a few seconds after the player is inside.
fn intro_sequence(
    time: Res<Time>,
    progress: Res<Progress>,
    assets: Res<GameAssets>,
    player: Query<&Transform, With<Player>>,
    mut doors: Query<&mut Swing, With<EntryDoor>>,
    mut commands: Commands,
    mut sub: ResMut<Subtitle>,
    mut sfx: EventWriter<PlaySfx>,
    blockers: Query<Entity, With<EntryDoorBlock>>,
) {
    let elapsed = time.elapsed_secs() - progress.started;
    if elapsed < 6.5 {
        return;
    }
    // once they are shut they stay shut
    if doors.iter().any(|s| s.active) {
        return;
    }
    let Ok(t) = player.get_single() else {
        return;
    };
    if t.translation.z > 8.5 {
        return;
    }
    for mut swing in doors.iter_mut() {
        if !swing.active {
            swing.active = true;
        }
    }
    sfx.send(PlaySfx::flat(assets.snd.door_slam.clone(), 0.85));
    sub.say(
        "Die Flügeltür fällt zu. Das Schloss rastet ein, bevor ich mich umdrehen kann.",
        6.0,
    );
    if blockers.iter().next().is_none() {
        commands.spawn((
            Transform::from_translation(Vec3::new(0.0, 1.6, 10.0)),
            crate::collision::SolidBox {
                half: Vec3::new(2.3, 1.7, 0.2),
            },
            EntryDoorBlock,
            Name::new("entry_locked"),
        ));
    }
}

#[derive(Component)]
pub struct EntryDoorBlock;

/// Small narrative beats fired by walking into a new area.
fn area_beats(
    player: Query<&Transform, With<Player>>,
    assets: Res<GameAssets>,
    mut progress: ResMut<Progress>,
    mut sub: ResMut<Subtitle>,
    mut sfx: EventWriter<PlaySfx>,
    mut flicker_q: Query<&mut crate::world::Flicker>,
) {
    let Ok(t) = player.get_single() else {
        return;
    };
    let p = t.translation;
    let mut beat = |progress: &mut Progress, n: u8, text: &str, secs: f32| {
        if progress.scene < n {
            progress.scene = n;
            sub.say(text, secs);
        }
    };

    if p.y < 1.2 && p.z < 6.0 && p.z > -11.0 && p.x.abs() < 12.0 {
        beat(
            &mut progress,
            1,
            "Der Regen bleibt draußen. Es riecht nach feuchtem Holz, Terpentin und \
etwas Süßlichem, das hier nicht hingehört.",
            7.0,
        );
    }
    if p.y > 4.0 && p.y < 8.4 && p.z < -17.0 && p.x < 15.5 {
        beat(
            &mut progress,
            2,
            "Der Korridor. Sechs Türen, alle verschlossen. Der Wind schiebt Papier über \
den Teppich.",
            7.0,
        );
    }
    if p.y > 4.0 && p.x > 16.0 {
        beat(
            &mut progress,
            3,
            "Der Maschinenraum. Irgendwo unter dem Boden läuft noch ein Strom, der nicht \
existieren dürfte.",
            7.0,
        );
        let _ = assets;
    }
    if p.y > 8.6 && p.x < 6.0 {
        beat(
            &mut progress,
            4,
            "Der Ballsaal. Jeder Schritt kommt doppelt zurück, und der zweite Schritt ist \
immer ein wenig zu spät.",
            8.0,
        );
    }
    if p.y > 8.6 && p.x > 6.6 {
        beat(
            &mut progress,
            5,
            "Die Terrasse liegt unter der Wolkendecke. Der Sturm hat die Bäume bis an die \
Brüstung gedrückt.",
            7.0,
        );
    }
    if p.x > 23.0 {
        beat(
            &mut progress,
            6,
            "Der Dienstgang. Die Luft steht, und die Ziegel sind warm.",
            6.0,
        );
        // the bulbs die one by one behind the player
        for mut fl in flicker_q.iter_mut() {
            if fl.base < 900.0 {
                fl.dying = true;
                fl.amp = 1.0;
            }
        }
        let _ = &mut sfx;
    }
}

/// After a long silence, something whispers behind the player in the corridor.
fn whisper_hint(
    time: Res<Time>,
    progress: Res<Progress>,
    player: Query<&Transform, With<Player>>,
    focus: Res<Focus>,
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
    mut last: Local<f32>,
) {
    let Ok(t) = player.get_single() else {
        return;
    };
    // only in the first-floor corridor, only when nothing is being read
    let in_corridor = t.translation.y > 4.0
        && t.translation.y < 8.4
        && t.translation.z < -17.0
        && t.translation.x < 15.5;
    if !in_corridor || focus.entity.is_some() {
        return;
    }
    let now = time.elapsed_secs();
    if now - *last < 24.0 + progress.scene as f32 * 2.0 {
        return;
    }
    *last = now;
    if progress.scene >= 2 {
        sfx.send(PlaySfx::flat(assets.snd.whisper.clone(), 0.30));
    }
}
