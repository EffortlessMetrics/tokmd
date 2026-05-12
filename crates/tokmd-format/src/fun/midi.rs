//! MIDI rendering for fun analysis outputs.

use anyhow::Result;
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

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

    events.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| {
            let rank = |k: &TrackEventKind| -> (u8, u8, u8) {
                match k {
                    TrackEventKind::Meta(_) => (0, 0, 0),
                    TrackEventKind::Midi {
                        channel,
                        message: MidiMessage::NoteOff { key, .. },
                    } => (1, (*channel).into(), (*key).into()),
                    TrackEventKind::Midi {
                        channel,
                        message: MidiMessage::NoteOn { key, .. },
                    } => (2, (*channel).into(), (*key).into()),
                    _ => (3, 0, 0),
                }
            };
            rank(&a.1).cmp(&rank(&b.1))
        })
    });

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
    fn render_midi_deterministic_overlap() {
        let notes1 = vec![
            MidiNote {
                key: 60,
                velocity: 100,
                start: 0,
                duration: 480,
                channel: 0,
            },
            MidiNote {
                key: 64,
                velocity: 100,
                start: 0,
                duration: 480,
                channel: 1,
            },
        ];
        let notes2 = vec![
            MidiNote {
                key: 64,
                velocity: 100,
                start: 0,
                duration: 480,
                channel: 1,
            },
            MidiNote {
                key: 60,
                velocity: 100,
                start: 0,
                duration: 480,
                channel: 0,
            },
        ];

        let result1 = render_midi(&notes1, 120).unwrap();
        let result2 = render_midi(&notes2, 120).unwrap();

        assert_eq!(
            result1, result2,
            "Output must be deterministic regardless of input note order"
        );
    }

    #[test]
    fn render_midi_empty_notes() {
        let result = render_midi(&[], 120).unwrap();
        assert!(!result.is_empty());
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
        assert!(result.len() > 14);
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
            channel: 255,
        }];
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
        let result = render_midi(&notes, 0).unwrap();
        assert!(!result.is_empty());
    }
}
