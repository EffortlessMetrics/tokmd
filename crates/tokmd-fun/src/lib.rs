//! # tokmd-fun
//!
//! Fun renderers for tokmd analysis outputs.

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
        header: Header::new(Format::SingleTrack, Timing::Metrical(ticks_per_quarter.into())),
        tracks: vec![track],
    };

    let mut out = Vec::new();
    smf.write_std(&mut out)?;
    Ok(out)
}
