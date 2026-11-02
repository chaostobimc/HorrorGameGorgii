//! The level file: what the editor writes and the game reads back.
//!
//! Two kinds of lines, both plain text and both hand editable:
//!
//! ```text
//! # an object that does not exist in the hand built hotel
//! rug   3.500 0.010 2.000  0.0000  w=3.5 d=2.2
//! model 1.000 0.000 2.000  0.0000  file=models/door.glb voxel=0.25
//!
//! # a change to something the hotel builder made
//! part  12 swap models/chair.glb 1.0
//! group 7  remove
//! group 9  move 0.500 0.000 0.000
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

/// What a change targets: one piece (a door leaf) or a whole prop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditTarget {
    Part(u32),
    Group(u32),
}

/// What happens to it.
#[derive(Clone, Debug)]
pub enum EditKind {
    /// Take it out of the hotel.
    Remove,
    /// Make it invisible, keep its behaviour (a hidden door still opens).
    Hide,
    /// Draw a custom model in its place; keeps transform and behaviour.
    Swap { file: String, scale: f32 },
    /// Shift it.
    Move(Vec3),
}

#[derive(Clone, Debug)]
pub struct PartEdit {
    pub target: EditTarget,
    pub kind: EditKind,
    /// Kept so the editor can show what it was.
    pub label: String,
}

impl PartEdit {
    pub fn to_line(&self) -> String {
        match (&self.target, &self.kind) {
            (EditTarget::Part(id), EditKind::Remove) => format!("part {} remove", id),
            (EditTarget::Part(id), EditKind::Hide) => format!("part {} hide", id),
            (EditTarget::Part(id), EditKind::Swap { file, scale }) => {
                format!("part {} swap {} {:.3}", id, file, scale)
            }
            (EditTarget::Part(id), EditKind::Move(delta)) => format!(
                "part {} move {:.3} {:.3} {:.3}",
                id, delta.x, delta.y, delta.z
            ),
            (EditTarget::Group(id), EditKind::Remove) => format!("group {} remove", id),
            (EditTarget::Group(id), EditKind::Hide) => format!("group {} hide", id),
            (EditTarget::Group(id), EditKind::Swap { file, scale }) => {
                format!("group {} swap {} {:.3}", id, file, scale)
            }
            (EditTarget::Group(id), EditKind::Move(delta)) => format!(
                "group {} move {:.3} {:.3} {:.3}",
                id, delta.x, delta.y, delta.z
            ),
        }
    }

    pub fn from_line(line: &str) -> Option<Self> {
        let mut words = line.split_whitespace();
        let target = match words.next()? {
            "part" => EditTarget::Part(words.next()?.parse().ok()?),
            "group" => EditTarget::Group(words.next()?.parse().ok()?),
            _ => return None,
        };
        let kind = match words.next()? {
            "remove" => EditKind::Remove,
            "hide" => EditKind::Hide,
            "swap" => {
                let file = words.next()?.to_string();
                let scale: f32 = words
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1.0);
                EditKind::Swap { file, scale }
            }
            "move" => {
                let x: f32 = words.next()?.parse().ok()?;
                let y: f32 = words.next()?.parse().ok()?;
                let z: f32 = words.next()?.parse().ok()?;
                EditKind::Move(Vec3::new(x, y, z))
            }
            _ => return None,
        };
        Some(Self {
            target,
            kind,
            label: String::new(),
        })
    }
}

/// Every object and every change of the custom layer.
#[derive(Resource, Default, Clone)]
pub struct LevelFile {
    pub objects: Vec<LevelObject>,
    pub edits: Vec<PartEdit>,
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
        let mut edits = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with("part ") || line.starts_with("group ") {
                if let Some(edit) = PartEdit::from_line(line) {
                    edits.push(edit);
                }
            } else if let Some(object) = LevelObject::from_line(line) {
                objects.push(object);
            }
        }
        Self { objects, edits }
    }

    /// Every model file the level needs, so they can be loaded up front.
    pub fn model_files(&self) -> Vec<String> {
        let mut files: Vec<String> = Vec::new();
        for object in self.objects.iter() {
            if let Some(file) = object.get("file") {
                files.push(file.to_string());
            }
        }
        for edit in self.edits.iter() {
            if let EditKind::Swap { file, .. } = &edit.kind {
                files.push(file.clone());
            }
        }
        files.sort();
        files.dedup();
        files
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(LEVEL_PATH)
    }

    pub fn save_to(&self, path: &str) -> std::io::Result<()> {
        if let Some(dir) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let mut text = String::from(
            "# The Diplomat Peak - custom level layer\n\
             # object: kind x y z yaw key=value ...\n\
             # change: part <id> swap <file> [scale] | part <id> hide | group <id> remove|move dx dy dz\n",
        );
        for object in self.objects.iter() {
            text.push_str(&object.to_line());
            text.push('\n');
        }
        if !self.edits.is_empty() {
            text.push_str("# changes to the hand built hotel\n");
        }
        for edit in self.edits.iter() {
            text.push_str(&edit.to_line());
            text.push('\n');
        }
        std::fs::write(path, text)
    }
}
