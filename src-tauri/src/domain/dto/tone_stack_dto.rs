use std::sync::atomic::Ordering;
use serde::{Deserialize, Serialize};
use crate::domain::tone_stack::ToneStack;

/// Data transfer object for tone stack parameters.
///
/// Used for serializing and deserializing tone stack data, typically for communication between
/// the UI and backend. The bass, middle, and treble values are expected to be in the range 0.0 to 1.0.
#[derive(Serialize,Deserialize, Clone, Debug)]
pub struct ToneStackDto{
    /// The bass level (0.0 to 1.0).
    pub bass: f32,
    /// The middle level (0.0 to 1.0).
    pub middle: f32,
    /// The treble level (0.0 to 1.0).
    pub treble: f32
}


impl From<&ToneStack> for ToneStackDto {
    fn from(tone_stack: &ToneStack) -> Self {
        Self {
            bass: tone_stack.bass().load(Ordering::Relaxed)*100.0,
            middle: tone_stack.middle().load(Ordering::Relaxed)*100.0,
            treble: tone_stack.treble().load(Ordering::Relaxed)*100.0,
        }
    }
}
