//! # tokmd-fun
//!
//! **Tier 3 (Novelty)**
//!
//! Fun renderers for tokmd analysis outputs. Provides creative visualizations
//! like 3D code cities and audio representations.
//!
//! ## What belongs here
//! * 3D code city visualization (OBJ format)
//! * Audio representation (MIDI format)
//! * Eco-label generation
//! * Other novelty outputs
//!
//! ## What does NOT belong here
//! * Serious analysis features
//! * Analysis computation
//! * Core receipt formatting

use anyhow::Result;
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

#[derive(Debug, Clone)]
pub struct ObjBuilding {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub d: f32,
    pub h: f32,
}

pub fn render_obj(buildings: &[ObjBuilding]) -> String {
    let mut out = String::new();
    out.push_str("# tokmd code city\n");
    let mut vertex_index = 1usize;

    for b in buildings {
        out.push_str(&format!("o {}\n", sanitize_name(&b.name)));
        let (x, y, z) = (b.x, b.y, 0.0f32);
        let (w, d, h) = (b.w, b.d, b.h);

        let v = [
            (x, y, z),
            (x + w, y, z),
            (x + w, y + d, z),
            (x, y + d, z),
            (x, y, z + h),
            (x + w, y, z + h),
            (x + w, y + d, z + h),
            (x, y + d, z + h),
        ];
        for (vx, vy, vz) in v {
            out.push_str(&format!("v {} {} {}\n", vx, vy, vz));
        }

        let faces = [
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [1, 2, 6, 5],
            [2, 3, 7, 6],
            [3, 4, 8, 7],
            [4, 1, 5, 8],
        ];
        for face in faces {
            out.push_str(&format!(
                "f {} {} {} {}\n",
                vertex_index + face[0] - 1,
                vertex_index + face[1] - 1,
                vertex_index + face[2] - 1,
                vertex_index + face[3] - 1,
            ));
        }

        vertex_index += 8;
    }

    out
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub key: u8,
    pub velocity: u8,
    pub start: u32,
    pub duration: u32,
    pub channel: u8,
}

pub fn render_midi(notes: &[MidiNote], tempo_bpm: u16) -> Result<Vec<u8>> {
    let ticks_per_quarter = 480u16;
    let mut events: Vec<(u32, TrackEventKind<'static>)> = Vec::new();

    let tempo = 60_000_000u32 / tempo_bpm.max(1) as u32;
    events.push((0, TrackEventKind::Meta(MetaMessage::Tempo(tempo.into()))));

    for note in notes {
        let ch = note.channel.min(15).into();
        events.push((
            note.start,
            TrackEventKind::Midi {
                channel: ch,
                message: MidiMessage::NoteOn {
                    key: note.key.into(),
                    vel: note.velocity.into(),
                },
            },
        ));
        events.push((
            note.start + note.duration,
            TrackEventKind::Midi {
                channel: ch,
                message: MidiMessage::NoteOff {
                    key: note.key.into(),
                    vel: 0.into(),
                },
            },
        ));
    }

    events.sort_by_key(|e| e.0);

    let mut track: Vec<TrackEvent> = Vec::new();
    let mut last_time = 0u32;
    for (time, kind) in events {
        let delta = time.saturating_sub(last_time);
        last_time = time;
        track.push(TrackEvent {
            delta: delta.into(),
            kind,
        });
    }

    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let smf = Smf {
        header: Header::new(
            Format::SingleTrack,
            Timing::Metrical(ticks_per_quarter.into()),
        ),
        tracks: vec![track],
    };

    let mut out = Vec::new();
    smf.write_std(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_obj_empty_input() {
        let out = render_obj(&[]);
        assert_eq!(out, "# tokmd code city\n");
    }

    #[test]
    fn render_obj_single_building() {
        let b = ObjBuilding {
            name: "src/main.rs".into(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 2.0,
        };
        let out = render_obj(&[b]);
        assert!(out.starts_with("# tokmd code city\n"));
        // Non-alphanumeric chars in name are replaced with underscores
        assert!(out.contains("o src_main_rs\n"));
        // 8 vertices per building
        assert_eq!(out.matches("\nv ").count(), 8);
        // 6 faces per building
        assert_eq!(out.matches("\nf ").count(), 6);
    }

    #[test]
    fn render_obj_deterministic() {
        let buildings = vec![
            ObjBuilding {
                name: "a".into(),
                x: 0.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                h: 3.0,
            },
            ObjBuilding {
                name: "b".into(),
                x: 2.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                h: 5.0,
            },
        ];
        let a = render_obj(&buildings);
        let b = render_obj(&buildings);
        assert_eq!(a, b, "render_obj must be deterministic");
    }

    #[test]
    fn render_obj_vertex_indices_advance() {
        let buildings = vec![
            ObjBuilding {
                name: "a".into(),
                x: 0.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                h: 1.0,
            },
            ObjBuilding {
                name: "b".into(),
                x: 2.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                h: 1.0,
            },
        ];
        let out = render_obj(&buildings);
        // Second building faces should reference vertices 9-16
        assert!(out.contains("f 9 10 11 12\n"));
    }

    #[test]
    fn render_midi_empty_notes() {
        let bytes = render_midi(&[], 120).unwrap();
        // Valid MIDI starts with "MThd"
        assert!(bytes.len() >= 4);
        assert_eq!(&bytes[..4], b"MThd");
    }

    #[test]
    fn render_midi_single_note() {
        let note = MidiNote {
            key: 60,
            velocity: 80,
            start: 0,
            duration: 480,
            channel: 0,
        };
        let bytes = render_midi(&[note], 120).unwrap();
        assert_eq!(&bytes[..4], b"MThd");
        // Must contain track chunk "MTrk"
        let has_mtrk = bytes.windows(4).any(|w| w == b"MTrk");
        assert!(has_mtrk, "MIDI output must contain MTrk chunk");
    }

    #[test]
    fn render_midi_deterministic() {
        let notes = vec![
            MidiNote {
                key: 60,
                velocity: 80,
                start: 0,
                duration: 480,
                channel: 0,
            },
            MidiNote {
                key: 64,
                velocity: 70,
                start: 480,
                duration: 480,
                channel: 0,
            },
        ];
        let a = render_midi(&notes, 120).unwrap();
        let b = render_midi(&notes, 120).unwrap();
        assert_eq!(a, b, "render_midi must be deterministic");
    }
}
