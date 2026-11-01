//! The level file: what the editor writes and the game reads back.
//!
//! One object per line, plain text, no dependencies:
//!
//! ```text
//! kind  x y z  yaw  key=value key=value
//! rug   3.500 0.010 2.000  0.0000  w=3.5 d=2.2
//! ```

use bevy::prelude::*;

/// Where the editor writes and the game reads.
pub const LEVEL_PATH: &str = "assets/level/extras.lvl";

/// One placed object: a library id, a transform and per kind settings.
#[derive(Clone, Debug)]
pub struct LevelObject {
    pub kind: String,
    pub pos: Vec3,
    pub yaw: f32,
    pub props: Vec<(String, String)>,
}

impl LevelObject {
    pub fn new(kind: &str, pos: Vec3) -> Self {
        Self {
            kind: kind.to_string(),
            pos,
            yaw: 0.0,
            props: Vec::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        let value = value.into();
        if let Some(slot) = self.props.iter_mut().find(|(k, _)| k == key) {
            slot.1 = value;
        } else {
            self.props.push((key.to_string(), value));
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.props
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn number(&self, key: &str, fallback: f32) -> f32 {
        self.get(key)
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(fallback)
    }

    pub fn integer(&self, key: &str, fallback: usize) -> usize {
        self.get(key)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(fallback)
    }

    pub fn flag(&self, key: &str, fallback: bool) -> bool {
        self.get(key).map(|v| v != "0").unwrap_or(fallback)
    }

    pub fn to_line(&self) -> String {
        let mut out = format!(
            "{} {:.3} {:.3} {:.3} {:.4}",
            self.kind, self.pos.x, self.pos.y, self.pos.z, self.yaw
        );
        for (key, value) in self.props.iter() {
            out.push(' ');
            out.push_str(key);
            out.push('=');
            out.push_str(value);
        }
        out
    }

    pub fn from_line(line: &str) -> Option<Self> {
        let mut parts = line.split_whitespace();
        let kind = parts.next()?.to_string();
        if kind.starts_with('#') {
            return None;
        }
        let x: f32 = parts.next()?.parse().ok()?;
        let y: f32 = parts.next()?.parse().ok()?;
        let z: f32 = parts.next()?.parse().ok()?;
        let yaw: f32 = parts.next()?.parse().ok()?;
        let mut props = Vec::new();
        for part in parts {
            if let Some((key, value)) = part.split_once('=') {
                props.push((key.to_string(), value.to_string()));
            }
        }
        Some(Self {
            kind,
            pos: Vec3::new(x, y, z),
            yaw,
            props,
        })
    }
}

/// Every object of the custom layer.
#[derive(Resource, Default, Clone)]
pub struct LevelFile {
    pub objects: Vec<LevelObject>,
}

impl LevelFile {
    pub fn load() -> Self {
        Self::load_from(LEVEL_PATH)
    }

    pub fn load_from(path: &str) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let mut objects = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(object) = LevelObject::from_line(line) {
                objects.push(object);
            }
        }
        Self { objects }
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(LEVEL_PATH)
    }

    pub fn save_to(&self, path: &str) -> std::io::Result<()> {
        if let Some(dir) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let mut text = String::from(
            "# The Diplomat Peak - custom level layer\n# kind x y z yaw key=value ...\n",
        );
        for object in self.objects.iter() {
            text.push_str(&object.to_line());
            text.push('\n');
        }
        std::fs::write(path, text)
    }
}
