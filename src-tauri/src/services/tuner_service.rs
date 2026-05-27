use crate::services::analyzers::spectrum_tap::{SpectrumTap, SPECTRUM_WINDOW_SIZE};
use log::info;
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use std::cell::RefCell;


pub struct PitchSnapshot {
    pub frequency_hz: f32,
    pub note_name: String,
    pub cents_deviation: f32,
    pub clarity: f32,
}

pub struct TunerService;

thread_local! {
    static MCLEOD_DETECTOR: RefCell<McLeodDetector<f64>> =
        RefCell::new(McLeodDetector::new(SPECTRUM_WINDOW_SIZE, SPECTRUM_WINDOW_SIZE / 2));
}

impl TunerService {
    pub fn detect_pitch(tap: &SpectrumTap) -> Option<PitchSnapshot> {
        let sample_rate = tap.sample_rate_hz() as usize;
        let samples = tap.snapshot_window();

        if samples.is_empty() {
            return None;
        }

        let signal: Vec<f64> = samples.iter().map(|&s| s as f64).collect();

        let power_threshold = 0.01;
        let clarity_threshold = 0.65;

        MCLEOD_DETECTOR.with(|cell| {
            let mut detector = cell.borrow_mut();

            let pitch =
                detector.get_pitch(&signal, sample_rate, power_threshold, clarity_threshold)?;

            if pitch.frequency > 4000.0 {
                return None;
            }

            Some(Self::hz_to_pitch_snapshot(
                pitch.frequency as f32,
                pitch.clarity as f32,
            ))
        })
    }

    fn hz_to_pitch_snapshot(mut frequency: f32, clarity: f32) -> PitchSnapshot {
        if frequency <= 20.0 {
            return PitchSnapshot {
                frequency_hz: frequency,
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                clarity,
            };
        }

        //Double the frequency because McLeodDetector returns the fundamental resonance frequency,
        //which is an octave below the actual note for guitar pickups.
        frequency *= 2.0;

        let n = 12.0 * (frequency / 440.0).log2();
        let midi_note = n.round() + 69.0;
        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];

        let midi_int = midi_note as i32;
        let note_index = ((midi_int % 12 + 12) % 12) as usize;
        let octave = ((midi_note / 12.0).floor() - 1.0) as i32;
        let note_name = format!("{}{}", note_names[note_index], octave);

        let cents_deviation = (n - n.round()) * 100.0;

        info!(
            "Detected pitch: {:.2} Hz, note: {}, cents deviation: {:.2}, clarity: {:.2}",
            frequency, note_name, cents_deviation, clarity
        );

        PitchSnapshot {
            frequency_hz: frequency,
            note_name,
            cents_deviation,
            clarity,
        }
    }
}
