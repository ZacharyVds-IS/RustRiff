use atomic_float::AtomicF32;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// Number of most-recent samples retained for analyzer FFT snapshots.
///
/// Smaller values lower latency and CPU cost, but reduce low-frequency detail.
pub const SPECTRUM_WINDOW_SIZE: usize = 2048;

/// Lock-free sample tap used by the analyzer view.
///
/// The audio worker writes samples here without taking locks. UI-side commands can
/// snapshot the latest window and compute FFT data off the audio path.
pub struct SpectrumTap {
    sample_rate_hz: AtomicU32,
    write_index: AtomicUsize,
    samples: Vec<AtomicF32>,
}

impl SpectrumTap {
    /// Creates a new lock-free tap with preallocated sample storage.
    pub fn new(sample_rate_hz: u32) -> Self {
        Self {
            sample_rate_hz: AtomicU32::new(sample_rate_hz.max(1)),
            write_index: AtomicUsize::new(0),
            samples: (0..SPECTRUM_WINDOW_SIZE)
                .map(|_| AtomicF32::new(0.0))
                .collect(),
        }
    }

    /// Updates sample rate metadata read by analyzer commands.
    pub fn set_sample_rate_hz(&self, sample_rate_hz: u32) {
        self.sample_rate_hz
            .store(sample_rate_hz.max(1), Ordering::Relaxed);
    }

    /// Returns the latest sample rate metadata.
    pub fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz.load(Ordering::Relaxed)
    }

    /// Writes one processed sample into the circular tap buffer.
    pub fn push_sample(&self, sample: f32) {
        let index = self.write_index.fetch_add(1, Ordering::Relaxed) % self.samples.len();
        self.samples[index].store(sample, Ordering::Relaxed);
    }

    /// Returns a chronologically ordered snapshot of the current ring buffer.
    pub fn snapshot_window(&self) -> Vec<f32> {
        let len = self.samples.len();
        let newest_index = self.write_index.load(Ordering::Relaxed) % len;

        (0..len)
            .map(|i| {
                let idx = (newest_index + i) % len;
                self.samples[idx].load(Ordering::Relaxed)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn new_initializes_zeroed_window_and_sample_rate() {
            let tap = SpectrumTap::new(48_000);

            assert_eq!(tap.sample_rate_hz(), 48_000);
            let snapshot = tap.snapshot_window();
            assert_eq!(snapshot.len(), SPECTRUM_WINDOW_SIZE);
            assert!(snapshot.iter().all(|sample| *sample == 0.0));
        }

        #[test]
        fn set_sample_rate_hz_updates_metadata() {
            let tap = SpectrumTap::new(44_100);

            tap.set_sample_rate_hz(96_000);

            assert_eq!(tap.sample_rate_hz(), 96_000);
        }

        #[test]
        fn snapshot_window_preserves_chronological_order_after_wraparound() {
            let tap = SpectrumTap::new(48_000);

            // Push more samples than the buffer length to force at least one full wrap.
            for i in 0..(SPECTRUM_WINDOW_SIZE + 16) {
                tap.push_sample(i as f32);
            }

            let snapshot = tap.snapshot_window();
            assert_eq!(snapshot.len(), SPECTRUM_WINDOW_SIZE);

            let expected_start = 16_f32;
            let expected_end = (SPECTRUM_WINDOW_SIZE + 15) as f32;
            assert_eq!(snapshot.first().copied(), Some(expected_start));
            assert_eq!(snapshot.last().copied(), Some(expected_end));

            for pair in snapshot.windows(2) {
                assert!(pair[1] >= pair[0]);
            }
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn new_with_zero_sample_rate_clamps_to_one() {
            let tap = SpectrumTap::new(0);
            assert_eq!(tap.sample_rate_hz(), 1);
        }

        #[test]
        fn set_sample_rate_hz_with_zero_clamps_to_one() {
            let tap = SpectrumTap::new(48_000);
            tap.set_sample_rate_hz(0);
            assert_eq!(tap.sample_rate_hz(), 1);
        }
    }
}
