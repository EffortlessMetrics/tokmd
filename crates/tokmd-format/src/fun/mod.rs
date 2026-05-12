//! Fun renderers for analysis outputs.
//!
//! This module provides creative visualizations like 3D code cities and audio
//! representations for `tokmd-format`'s analysis rendering surface.
//!
//! ## What belongs here
//! * 3D code city visualization (OBJ format)
//! * Audio representation (MIDI format)
//! * Other novelty outputs
//!
//! ## What does NOT belong here
//! * Serious analysis features
//! * Analysis computation
//! * Core receipt formatting outside the fun formats

mod midi;
mod obj;

pub use midi::{MidiNote, render_midi};
pub use obj::{ObjBuilding, render_obj};
