use crate::domain::dto::spectrum_snapshot_dto::SpectrumSnapshotDto;
use crate::services::analyzers::spectrum_tap::{SpectrumTap, SPECTRUM_WINDOW_SIZE};
use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::cell::RefCell;
use std::sync::{Arc, OnceLock};

/// Lower bound for analyzer frequencies in Hz.
const MIN_ANALYZER_FREQ_HZ: f32 = 20.0;
/// Lower clamp for displayed magnitudes (dBFS).
const MIN_DB: f32 = -90.0;
/// Upper clamp for displayed magnitudes (dBFS).
const MAX_DB: f32 = 6.0;
/// Number of points emitted to the frontend per frame.
const ANALYZER_BINS: usize = 96;
/// Upper frequency visible in the analyzer. Normal sample rates (≥ 44 100 Hz) always
/// reach Nyquist above this value, so the log-spaced bin frequencies are session-stable.
const MAX_ANALYZER_FREQ_HZ: f32 = 20_000.0;

/// Immutable per-session caches: FFT plan, Hann coefficients, and log-spaced frequencies.
///
/// All three only depend on the fixed window / bin constants and never change at runtime,
/// so a single `OnceLock` initialization is sufficient.
struct AnalyzerCaches {
    fft: Arc<dyn Fft<f32>>,
    hann: Vec<f32>,
    /// Log-spaced center frequencies shared across every frame in the DTO.
    frequencies_hz: Arc<[f32]>,
}

static CACHES: OnceLock<AnalyzerCaches> = OnceLock::new();

fn caches() -> &'static AnalyzerCaches {
    CACHES.get_or_init(|| {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(SPECTRUM_WINDOW_SIZE);
        let hann = (0..SPECTRUM_WINDOW_SIZE)
            .map(|i| hann_window(i, SPECTRUM_WINDOW_SIZE))
            .collect();
        let frequencies_hz: Arc<[f32]> = (0..ANALYZER_BINS)
            .map(|i| {
                frequency_for_bin(i, ANALYZER_BINS, MIN_ANALYZER_FREQ_HZ, MAX_ANALYZER_FREQ_HZ)
            })
            .collect::<Vec<_>>()
            .into();

        AnalyzerCaches {
            fft,
            hann,
            frequencies_hz,
        }
    })
}

// Per-thread scratch buffer so the hot path never allocates a Vec<Complex<f32>> per frame.
thread_local! {
    static FFT_BUF: RefCell<Vec<Complex<f32>>> =
        RefCell::new(vec![Complex::new(0.0, 0.0); SPECTRUM_WINDOW_SIZE]);
}

/// Stateless service that converts time-domain tap samples into log-spaced dB spectrum data.
pub struct SpectrumAnalyzerService;

impl SpectrumAnalyzerService {
    /// Builds a spectrum snapshot from the most recent samples in the tap.
    pub fn analyze_tap(tap: &SpectrumTap) -> SpectrumSnapshotDto {
        let sample_rate_hz = tap.sample_rate_hz();
        let samples = tap.snapshot_window();
        Self::analyze_samples(&samples, sample_rate_hz)
    }

    /// Computes FFT magnitudes at log-spaced frequencies and returns a frontend DTO.
    fn analyze_samples(samples: &[f32], sample_rate_hz: u32) -> SpectrumSnapshotDto {
        if samples.is_empty() {
            return SpectrumSnapshotDto {
                sample_rate_hz: sample_rate_hz.max(1),
                frequencies_hz: caches().frequencies_hz.to_vec(),
                magnitudes: vec![MIN_DB; ANALYZER_BINS],
                level_db: MIN_DB,
            };
        }

        let sample_rate = sample_rate_hz.max(1) as f32;

        if samples.len() == SPECTRUM_WINDOW_SIZE {
            // Hot path: reuse cached plan, Hann coefficients, and per-thread scratch buffer —
            // no heap allocation occurs here beyond the output magnitudes Vec.
            let c = caches();
            FFT_BUF.with(|cell| {
                let mut buf = cell.borrow_mut();
                for (i, (dst, &sample)) in buf.iter_mut().zip(samples.iter()).enumerate() {
                    *dst = Complex::new(sample * c.hann[i], 0.0);
                }
                c.fft.process(&mut buf);

                let magnitudes = c
                    .frequencies_hz
                    .iter()
                    .map(|&f| magnitude_db_at_frequency(&buf, sample_rate, f))
                    .collect();

                SpectrumSnapshotDto {
                    sample_rate_hz: sample_rate_hz.max(1),
                    frequencies_hz: c.frequencies_hz.to_vec(),
                    magnitudes,
                    level_db: rms_db(samples),
                }
            })
        } else {
            // Fallback path for non-standard window sizes (used in unit tests).
            let max_frequency_hz =
                (sample_rate * 0.5).clamp(MIN_ANALYZER_FREQ_HZ + 1.0, MAX_ANALYZER_FREQ_HZ);

            let mut fft_input: Vec<Complex<f32>> = samples
                .iter()
                .enumerate()
                .map(|(i, sample)| Complex::new(*sample * hann_window(i, samples.len()), 0.0))
                .collect();

            let mut planner = FftPlanner::<f32>::new();
            planner
                .plan_fft_forward(samples.len())
                .process(&mut fft_input);

            let frequencies_hz: Vec<f32> = (0..ANALYZER_BINS)
                .map(|i| {
                    frequency_for_bin(i, ANALYZER_BINS, MIN_ANALYZER_FREQ_HZ, max_frequency_hz)
                })
                .collect();

            let magnitudes: Vec<f32> = frequencies_hz
                .iter()
                .map(|&f| magnitude_db_at_frequency(&fft_input, sample_rate, f))
                .collect();

            SpectrumSnapshotDto {
                sample_rate_hz: sample_rate_hz.max(1),
                frequencies_hz,
                magnitudes,
                level_db: rms_db(samples),
            }
        }
    }
}

/// Reads one FFT bin nearest to the target frequency and returns clamped dBFS.
fn magnitude_db_at_frequency(
    spectrum: &[Complex<f32>],
    sample_rate: f32,
    frequency_hz: f32,
) -> f32 {
    let n = spectrum.len().max(2);
    let half = n / 2;
    if half <= 1 {
        return MIN_DB;
    }

    let exact_bin = (frequency_hz / sample_rate) * n as f32;


    let max_bound = (half - 1) as f32;
    let exact_bin = exact_bin.clamp(0.0, max_bound);

    let bin_low = exact_bin.floor() as usize;
    let bin_high = exact_bin.ceil().min((half - 1) as f32) as usize;
    let t = exact_bin - bin_low as f32;

    let norm_low = spectrum[bin_low].norm();
    let norm_high = spectrum[bin_high].norm();

    let interpolated_norm = (1.0 - t) * norm_low + t * norm_high;

    let normalized = (2.0 * interpolated_norm) / n as f32;
    (20.0 * normalized.max(1e-7).log10()).clamp(MIN_DB, MAX_DB)
}

/// Returns the geometric center frequency for a log-spaced analyzer bin.
fn frequency_for_bin(index: usize, bin_count: usize, min_hz: f32, max_hz: f32) -> f32 {
    let ratio = max_hz / min_hz;
    let center = (index as f32 + 0.5) / bin_count as f32;
    min_hz * ratio.powf(center)
}

/// Hann window coefficient for sample `index` in a window of length `len`.
fn hann_window(index: usize, len: usize) -> f32 {
    if len <= 1 {
        return 1.0;
    }

    let phase = (2.0 * std::f32::consts::PI * index as f32) / (len as f32 - 1.0);
    0.5 * (1.0 - phase.cos())
}

/// Computes whole-window RMS in dBFS for level metering.
fn rms_db(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return MIN_DB;
    }

    let mean_square =
        samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32;
    let rms = mean_square.sqrt();
    (20.0 * rms.max(1e-7).log10()).max(MIN_DB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn analyze_samples_returns_expected_bin_shape() {
            let samples = vec![0.0_f32; 2048];
            let snapshot = SpectrumAnalyzerService::analyze_samples(&samples, 48_000);

            assert_eq!(snapshot.sample_rate_hz, 48_000);
            assert_eq!(snapshot.magnitudes.len(), ANALYZER_BINS);
            assert_eq!(snapshot.frequencies_hz.len(), ANALYZER_BINS);
            assert!(snapshot.level_db <= 0.0);
            assert!(snapshot.level_db >= MIN_DB);
        }

        #[test]
        fn analyze_samples_detects_a_tone_peak() {
            let sample_rate = 48_000.0;
            let target_freq = 1_000.0;
            let samples = (0..2048)
                .map(|n| {
                    (2.0 * std::f32::consts::PI * target_freq * (n as f32 / sample_rate)).sin()
                        * 0.8
                })
                .collect::<Vec<_>>();

            let snapshot = SpectrumAnalyzerService::analyze_samples(&samples, sample_rate as u32);
            let peak_value = snapshot.magnitudes.iter().copied().fold(MIN_DB, f32::max);

            assert!(peak_value > -35.0);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn analyze_samples_with_empty_input_returns_safe_defaults() {
            let snapshot = SpectrumAnalyzerService::analyze_samples(&[], 0);

            assert_eq!(snapshot.sample_rate_hz, 1);
            assert_eq!(snapshot.magnitudes.len(), ANALYZER_BINS);
            assert!(snapshot.magnitudes.iter().all(|value| *value == MIN_DB));
            assert_eq!(snapshot.level_db, MIN_DB);
        }

        #[test]
        fn magnitude_db_with_tiny_fft_input_returns_min_db_instead_of_panicking() {
            let tiny = vec![Complex::new(0.0_f32, 0.0_f32); 2];
            let db = magnitude_db_at_frequency(&tiny, 48_000.0, 1_000.0);
            assert_eq!(db, MIN_DB);
        }
    }
}
