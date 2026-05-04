use atomic_float::AtomicF32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::error;

/// Represents the tone stack with atomic bass, middle, and treble parameters for audio equalization.
///
/// The tone stack provides controls for adjusting low (bass), mid-range (middle), and high (treble) frequencies.
/// It uses [`AtomicF32`] for lock-free updates, enabling low-latency parameter changes from the UI thread
/// while the audio processing thread reads them without interruption.
///
/// All parameters are validated to be between 0.0 and 1.0 (inclusive); attempting to set a value outside this range will panic.
pub struct ToneStack {
    bass: Arc<AtomicF32>,
    middle: Arc<AtomicF32>,
    treble: Arc<AtomicF32>,
}

impl ToneStack {
    /// Creates a new `ToneStack` with default parameter values.
    ///
    /// Bass, middle, and treble are initialized to `1.0`.
    pub fn new() -> Self {
        Self {
            bass: Arc::new(AtomicF32::new(1.0)),
            middle: Arc::new(AtomicF32::new(1.0)),
            treble: Arc::new(AtomicF32::new(1.0)),
        }
    }

    /// Returns a cloned [`Arc`] to the atomic bass value.
    ///
    /// Allows independent threads to share and read/write the bass parameter without contention.
    pub fn bass(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.bass)
    }

    /// Returns a cloned [`Arc`] to the atomic middle value.
    ///
    /// Allows independent threads to share and read/write the middle parameter without contention.
    pub fn middle(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.middle)
    }

    /// Returns a cloned [`Arc`] to the atomic treble value.
    ///
    /// Allows independent threads to share and read/write the treble parameter without contention.
    pub fn treble(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.treble)
    }

    /// Sets the bass level for the tone stack.
    ///
    /// The bass value is atomically updated and will be read by the audio processing thread on the next sample cycle.
    ///
    /// # Arguments
    ///
    /// * `bass` - The new bass value. Must be between 0.0 and 1.0 (inclusive).
    ///
    /// # Panics
    ///
    /// Panics if `bass` is negative or greater than 1.0.
    pub fn set_bass(&self, bass: f32) {
        if bass.is_sign_positive() && bass <= 1.0 {
            self.bass.store(bass, Ordering::Relaxed);
        } else {
            error!("Bass must be a positive number between 0 and 1");
            panic!("Bass must be positive and between 0 and 1");
        }
    }

    /// Sets the middle level for the tone stack.
    ///
    /// The middle value is atomically updated and will be read by the audio processing thread on the next sample cycle.
    ///
    /// # Arguments
    ///
    /// * `middle` - The new middle value. Must be between 0.0 and 1.0 (inclusive).
    ///
    /// # Panics
    ///
    /// Panics if `middle` is negative or greater than 1.0.
    pub fn set_middle(&self, middle: f32) {
        if middle.is_sign_positive() && middle <= 1.0 {
            self.middle.store(middle, Ordering::Relaxed);
        }else {
            error!("Middle must be a positive number between 0 and 1");
            panic!("Middle must be positive and between 0 and 1");
        }
    }

    /// Sets the treble level for the tone stack.
    ///
    /// The treble value is atomically updated and will be read by the audio processing thread on the next sample cycle.
    ///
    /// # Arguments
    ///
    /// * `treble` - The new treble value. Must be between 0.0 and 1.0 (inclusive).
    ///
    /// # Panics
    ///
    /// Panics if `treble` is negative or greater than 1.0.
    pub fn set_treble(&self, treble: f32) {
        if treble.is_sign_positive() && treble <= 1.0{
            self.treble.store(treble, Ordering::Relaxed);
        } else {
            error!("Treble must be a positive number between 0 and 1");
            panic!("Treble must be positive and between 0 and 1");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn bass_set_to_positive_value_within_range_should_succeed() {
            let tone_stack = ToneStack::new();
            tone_stack.set_bass(0.5);
            assert_eq!(tone_stack.bass().load(Ordering::Relaxed), 0.5);
        }
        #[test]
        fn middle_set_to_positive_value_within_range_should_succeed() {
            let tone_stack = ToneStack::new();
            tone_stack.set_middle(0.5);
            assert_eq!(tone_stack.middle().load(Ordering::Relaxed), 0.5);
        }
        #[test]
        fn treble_set_to_positive_value_within_range_should_succeed() {
            let tone_stack = ToneStack::new();
            tone_stack.set_treble(0.5);
            assert_eq!(tone_stack.treble().load(Ordering::Relaxed), 0.5);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        #[should_panic(expected = "Bass must be positive and between 0 and 1")]
        fn bass_set_to_negative_value_should_panic() {
            let tone_stack = ToneStack::new();
            tone_stack.set_bass(-0.5);
        }
        #[test]
        #[should_panic(expected = "Bass must be positive and between 0 and 1")]
        fn bass_set_to_value_greater_than_one_should_panic() {
            let tone_stack = ToneStack::new();
            tone_stack.set_bass(1.5);
        }
        #[test]
        #[should_panic(expected = "Middle must be positive and between 0 and 1")]
        fn middle_set_to_negative_value_should_panic() {
            let tone_stack = ToneStack::new();
            tone_stack.set_middle(-0.5);
        }
        #[test]
        #[should_panic(expected = "Middle must be positive and between 0 and 1")]
        fn middle_set_to_value_greater_than_one_should_panic() {
            let tone_stack = ToneStack::new();
            tone_stack.set_middle(1.5);
        }
        #[test]
        #[should_panic(expected = "Treble must be positive and between 0 and 1")]
        fn treble_set_to_negative_value_should_panic() {
            let tone_stack = ToneStack::new();
            tone_stack.set_treble(-0.5);
        }
        #[test]
        #[should_panic(expected = "Treble must be positive and between 0 and 1")]
        fn treble_set_to_value_greater_than_one_should_panic() {
            let tone_stack = ToneStack::new();
            tone_stack.set_treble(1.5);
        }
    }
}