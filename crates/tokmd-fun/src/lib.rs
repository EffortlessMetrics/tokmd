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

    // ── sanitize_name ─────────────────────────────────────────────────
    #[test]
    fn sanitize_name_replaces_non_alphanumeric() {
        assert_eq!(sanitize_name("hello world"), "hello_world");
        assert_eq!(sanitize_name("src/main.rs"), "src_main_rs");
        assert_eq!(sanitize_name("foo-bar_baz"), "foo_bar_baz");
    }

    #[test]
    fn sanitize_name_preserves_alphanumeric() {
        assert_eq!(sanitize_name("abc123"), "abc123");
    }

    #[test]
    fn sanitize_name_empty() {
        assert_eq!(sanitize_name(""), "");
    }

    // ── render_obj ────────────────────────────────────────────────────
    #[test]
    fn render_obj_empty_input() {
        let result = render_obj(&[]);
        assert_eq!(result, "# tokmd code city\n");
    }

    #[test]
    fn render_obj_single_building() {
        let buildings = vec![ObjBuilding {
            name: "main".into(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 2.0,
        }];
        let result = render_obj(&buildings);
        assert!(result.starts_with("# tokmd code city\n"));
        assert!(result.contains("o main\n"));
        // 8 vertices per building
        assert_eq!(result.matches("\nv ").count(), 8);
        // 6 faces per building
        assert_eq!(result.matches("\nf ").count(), 6);
    }

    #[test]
    fn render_obj_multiple_buildings() {
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
                h: 3.0,
            },
        ];
        let result = render_obj(&buildings);
        assert!(result.contains("o a\n"));
        assert!(result.contains("o b\n"));
        // 16 vertices total (2 × 8)
        assert_eq!(result.matches("\nv ").count(), 16);
        // 12 faces total (2 × 6)
        assert_eq!(result.matches("\nf ").count(), 12);
    }

    #[test]
    fn render_obj_sanitizes_names() {
        let buildings = vec![ObjBuilding {
            name: "src/main.rs".into(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        }];
        let result = render_obj(&buildings);
        assert!(result.contains("o src_main_rs\n"));
        assert!(!result.contains("o src/main.rs\n"));
    }

    // ── render_midi ───────────────────────────────────────────────────
    #[test]
    fn render_midi_empty_notes() {
        let result = render_midi(&[], 120).unwrap();
        // Should produce valid MIDI even with no notes (header + tempo + end-of-track)
        assert!(!result.is_empty());
        // MIDI files start with "MThd"
        assert_eq!(&result[..4], b"MThd");
    }

    #[test]
    fn render_midi_single_note() {
        let notes = vec![MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        }];
        let result = render_midi(&notes, 120).unwrap();
        assert_eq!(&result[..4], b"MThd");
        // Should contain track data
        assert!(result.len() > 14); // Header is 14 bytes minimum
    }

    #[test]
    fn render_midi_multiple_notes() {
        let notes = vec![
            MidiNote {
                key: 60,
                velocity: 100,
                start: 0,
                duration: 480,
                channel: 0,
            },
            MidiNote {
                key: 64,
                velocity: 80,
                start: 480,
                duration: 480,
                channel: 0,
            },
            MidiNote {
                key: 67,
                velocity: 60,
                start: 960,
                duration: 480,
                channel: 1,
            },
        ];
        let result = render_midi(&notes, 120).unwrap();
        assert_eq!(&result[..4], b"MThd");
    }

    #[test]
    fn render_midi_channel_clamped_to_15() {
        let notes = vec![MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 255, // Should be clamped to 15
        }];
        // Should not panic or error
        let result = render_midi(&notes, 120).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn render_midi_tempo_min_clamped() {
        let notes = vec![MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        }];
        // tempo_bpm = 0 should be clamped via max(1)
        let result = render_midi(&notes, 0).unwrap();
        assert!(!result.is_empty());
    }
}
