//! Round-trip latency measurement using dedicated CPAL streams.
//!
//! # Overview
//!
//! Round-trip latency is the total time a sound takes to travel from the software output,
//! through the DAC, over the physical audio cable (or internal loopback), back in through
//! the ADC, and arrive at the input ring buffer where the software can read it.
//!
//! This module measures that latency by:
//!
//! 1. **Opening dedicated streams** — a private input/output pair, completely separate from
//!    the main loopback in [`AudioService`].  The main engine is unaffected and can stay
//!    running during the measurement.
//! 2. **Warming up** — sleeping for `stream_warmup` after stream creation so the OS audio
//!    stack and hardware buffers reach a stable steady state before any timing begins.
//! 3. **Calibrating the noise floor** — collecting [`CALIBRATION_SAMPLES`] silent ambient
//!    samples from the input to measure the peak background noise.  The detection threshold
//!    is derived from that peak so a genuine echo can be distinguished from ambient noise.
//! 4. **Emitting impulses** — writing a single full-amplitude sample ([`IMPULSE_AMPLITUDE`])
//!    to the output ring buffer while recording the exact wall-clock time.
//! 5. **Detecting the echo** — reading incoming input samples and waiting for the first
//!    sample that crosses the derived threshold.  A guard window of [`GUARD_SAMPLES`] is
//!    skipped immediately after the impulse to avoid detecting the outgoing signal itself
//!    or very early electrical bleed-through.
//! 6. **Averaging** — repeating the impulse/echo cycle [`IMPULSE_COUNT`] times and
//!    returning the mean of all measured round-trip durations to smooth out callback jitter.
//!
//! # Physical setup requirement
//!
//! The output of the audio interface must be physically (or virtually) connected back to its
//! input for the echo to be heard.  On a typical guitar-amp setup this is the instrument
//! cable path: software → DAC → speaker/DI → microphone/DI → ADC → software.  On a
//! development machine a virtual loopback driver (e.g. VB-Audio Cable) can be used instead.
//!
//! # Isolation from the main audio engine
//!
//! [`RoundTripLatencySession::run`] is a **blocking, self-contained call**.  It opens its
//! own CPAL streams via the same [`AudioHandlerTrait`] the main engine uses, but does **not**
//! share ring buffers, threads, or any state with [`AudioService`].  The caller
//! (`measure_round_trip_latency` command) releases the `Mutex<AudioService>` lock before
//! spawning the measurement thread, so the UI and all other commands remain fully responsive
//! during the several-second measurement window.
//!
//! [`AudioService`]: crate::services::audio_service::AudioService
//! [`AudioHandlerTrait`]: crate::infrastructure::audio_handler::AudioHandlerTrait

use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use cpal::BufferSize;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use std::thread;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Tuning constants
// ---------------------------------------------------------------------------

/// Number of input samples collected during the ambient calibration phase.
///
/// At 44 100 Hz this is roughly 11 ms of listening time — long enough to capture
/// a representative noise-floor peak without delaying the measurement significantly.
const CALIBRATION_SAMPLES: usize = 512;

/// Number of impulse/echo cycles to run per measurement session.
///
/// The final reported latency is the arithmetic mean of all [`IMPULSE_COUNT`] individual
/// round-trip measurements, which reduces the impact of single-callback scheduling jitter.
const IMPULSE_COUNT: usize = 3;

/// Number of input samples to ignore immediately after emitting an impulse.
///
/// Electrical bleed-through and the outgoing impulse itself can appear on the input within
/// microseconds of being written.  Skipping these samples prevents a false-positive detection
/// before the signal has had time to traverse the physical audio path.
///
/// At 44 100 Hz this guard window is approximately 11 ms.
const GUARD_SAMPLES: usize = 512;

/// Minimum quiet time enforced between consecutive impulses.
///
/// After an echo is detected the previous impulse's reverb tail may still be decaying.
/// Waiting [`INTER_IMPULSE_GAP`] before the next emission prevents that tail from being
/// mistaken for the next echo.
const INTER_IMPULSE_GAP: Duration = Duration::from_millis(200);

/// Peak amplitude of the synthetic test impulse written to the output ring buffer.
///
/// A near-full-scale value is used so the echo stands well above the noise floor even after
/// passing through lossy physical paths.  The detection threshold is clamped to at most
/// `IMPULSE_AMPLITUDE * 0.5` so that a valid echo is always detectable.
pub const IMPULSE_AMPLITUDE: f32 = 0.95;

/// Result of processing a single input sample through [`RoundTripMeasurementState::tick`].
///
/// The session loop calls `tick` for every sample that arrives on the input ring buffer and
/// acts on the returned outcome to decide whether to continue, finish, or abort.
pub enum RoundTripTickOutcome {
    /// The measurement is still in progress; more input samples are required.
    Ongoing,
    /// All [`IMPULSE_COUNT`] echoes were detected successfully.
    ///
    /// Contains the arithmetic mean of the individual round-trip durations in milliseconds.
    Complete(f64),
    /// The current impulse timed out before an echo was detected.
    ///
    /// This usually means the output is not physically routed back to the input, or the
    /// signal level is too low to cross the derived threshold.
    TimedOut,
}

/// High-level phases of the [`RoundTripMeasurementState`] state machine.
///
/// The machine progresses linearly through these phases on each measurement session:
///
/// ```text
/// CalibrationAmbient  →  WaitingForEcho(0)  →  WaitingForEcho(1)  →  …  →  Idle
/// ```
///
/// It never transitions backwards.  Once `Idle` is reached the session loop exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundTripMeasurementPhase {
    /// The measurement has concluded (either successfully or via timeout).
    ///
    /// The session loop writes silence to the output and exits on the next iteration.
    Idle,
    /// The state machine is consuming ambient input samples to estimate the noise floor.
    ///
    /// No impulses are emitted during this phase.  Transitions to
    /// `WaitingForEcho(0)` once [`CALIBRATION_SAMPLES`] have been processed.
    CalibrationAmbient,
    /// An impulse has been (or is about to be) emitted and the state machine is listening
    /// for its return on the input.
    ///
    /// The `usize` payload is the zero-based index of the current impulse (0 …
    /// [`IMPULSE_COUNT`] − 1).  On a successful echo detection the index advances and
    /// the machine stays in this variant; after the last echo it transitions to `Idle`.
    WaitingForEcho(usize),
}

/// All mutable state required to run one complete round-trip measurement session.
///
/// `RoundTripMeasurementState` is entirely owned by the session thread — there is no
/// shared ownership, no `Arc`, and no locking.  It is driven sample-by-sample through
/// [`tick`] until a terminal outcome is reached.
///
/// [`tick`]: RoundTripMeasurementState::tick
pub struct RoundTripMeasurementState {
    /// Current phase in the calibration/measurement lifecycle.
    pub phase: RoundTripMeasurementPhase,
    /// Derived amplitude threshold an incoming sample must exceed to be accepted as an echo.
    ///
    /// Set at the end of [`CalibrationAmbient`] and held constant for the rest of the session.
    ///
    /// [`CalibrationAmbient`]: RoundTripMeasurementPhase::CalibrationAmbient
    pub threshold: f32,
    /// Wall-clock time at which the currently active impulse was written to the output buffer.
    ///
    /// `None` before the first impulse is emitted and between the echo detection and the next
    /// impulse emission.  The elapsed time from this instant to echo detection is the raw
    /// round-trip duration.
    pub impulse_sent_at: Option<Instant>,
    /// Peak absolute amplitude observed across all calibration samples.
    ambient_peak: f32,
    /// Running count of calibration samples consumed so far.
    ambient_count: usize,
    /// Remaining samples in the post-impulse guard window.
    ///
    /// Decremented on every call to [`check_echo`] while non-zero.  Echo detection is
    /// suppressed until this reaches zero.
    ///
    /// [`check_echo`]: RoundTripMeasurementState::check_echo
    guard_remaining: usize,
    /// Measured round-trip duration for each successfully detected echo, in milliseconds.
    ///
    /// Grows by one entry per successful impulse/echo pair.  The final result is the
    /// mean of all entries.
    echo_durations_ms: Vec<f64>,
    /// Deadline by which the current impulse must produce an echo before timing out.
    impulse_deadline: Option<Instant>,
    /// Earliest wall-clock time at which the next impulse may be emitted.
    ///
    /// Enforces the [`INTER_IMPULSE_GAP`] quiet period between consecutive impulses.
    next_impulse_not_before: Option<Instant>,
}

impl RoundTripMeasurementState {
    /// Creates a fresh measurement state, starting in [`CalibrationAmbient`] phase.
    ///
    /// All counters are zeroed and no impulse timer is active.  The first call to [`tick`]
    /// will begin consuming ambient samples.
    ///
    /// [`CalibrationAmbient`]: RoundTripMeasurementPhase::CalibrationAmbient
    /// [`tick`]: RoundTripMeasurementState::tick
    pub fn new() -> Self {
        Self {
            phase: RoundTripMeasurementPhase::CalibrationAmbient,
            threshold: 0.0,
            impulse_sent_at: None,
            ambient_peak: 0.0,
            ambient_count: 0,
            guard_remaining: 0,
            echo_durations_ms: Vec::with_capacity(IMPULSE_COUNT),
            impulse_deadline: None,
            next_impulse_not_before: None,
        }
    }

    /// Ingests one ambient sample and returns `true` when calibration is complete.
    ///
    /// Tracks the running peak absolute amplitude.  Once [`CALIBRATION_SAMPLES`] have been
    /// consumed the detection threshold is finalised as:
    ///
    /// ```text
    /// threshold = clamp(ambient_peak × 2, 0.05, IMPULSE_AMPLITUDE × 0.5)
    /// ```
    ///
    /// The lower bound (`0.05`) ensures a minimum detectable signal even in a perfectly
    /// silent environment.  The upper bound (`IMPULSE_AMPLITUDE × 0.5`) guarantees the
    /// threshold can never exceed half the impulse amplitude, keeping detection reachable
    /// even on a lossy signal path.
    fn feed_ambient_sample(&mut self, sample: f32) -> bool {
        self.ambient_peak = self.ambient_peak.max(sample.abs());
        self.ambient_count += 1;

        if self.ambient_count < CALIBRATION_SAMPLES {
            return false;
        }

        self.threshold = (self.ambient_peak * 2.0)
            .max(0.05)
            .min(IMPULSE_AMPLITUDE * 0.5);

        println!(
            "[RT-MEASURE] Calibration done. Peak: {:.4}, threshold: {:.4}",
            self.ambient_peak, self.threshold
        );
        true
    }

    /// Arms the impulse timer and guard window for a newly emitted impulse.
    ///
    /// Records the current wall-clock time as [`impulse_sent_at`], sets the
    /// [`impulse_deadline`] to `now + per_impulse_timeout`, and resets
    /// [`guard_remaining`] to [`GUARD_SAMPLES`].
    ///
    /// [`impulse_sent_at`]: RoundTripMeasurementState::impulse_sent_at
    /// [`impulse_deadline`]: RoundTripMeasurementState::impulse_deadline
    /// [`guard_remaining`]: RoundTripMeasurementState::guard_remaining
    fn arm_impulse(&mut self, per_impulse_timeout: Duration) {
        let now = Instant::now();
        self.impulse_sent_at = Some(now);
        self.impulse_deadline = Some(now + per_impulse_timeout);
        self.guard_remaining = GUARD_SAMPLES;
    }

    /// Returns `true` if `sample` exceeds the detection threshold and the guard window has elapsed.
    ///
    /// While [`guard_remaining`] is non-zero the function always returns `false` and
    /// decrements the counter, enforcing the post-impulse blind period.  Once the guard
    /// expires, any sample whose absolute value is ≥ [`threshold`] is accepted as an echo.
    ///
    /// [`guard_remaining`]: RoundTripMeasurementState::guard_remaining
    /// [`threshold`]: RoundTripMeasurementState::threshold
    fn check_echo(&mut self, sample: f32) -> bool {
        if self.guard_remaining > 0 {
            self.guard_remaining -= 1;
            return false;
        }
        sample.abs() >= self.threshold
    }

    /// Returns `true` if the current impulse deadline has passed without an echo.
    fn is_timed_out(&self) -> bool {
        self.impulse_deadline
            .map(|deadline| Instant::now() >= deadline)
            .unwrap_or(false)
    }

    /// Advances the measurement state machine by one input sample.
    ///
    /// This is the core driver of the entire measurement protocol.  The caller must
    /// invoke it for every sample that arrives from the input ring buffer.
    ///
    /// # Arguments
    ///
    /// * `sample` — The raw `f32` sample captured from the audio input.
    /// * `push_output` — A closure that writes one `f32` value to the output ring buffer
    ///   and returns `true` if the push succeeded.  The state machine uses this to emit
    ///   either silence (`0.0`) or the test impulse ([`IMPULSE_AMPLITUDE`]).
    /// * `per_impulse_timeout` — How long to wait for an echo after each impulse before
    ///   declaring a timeout.
    ///
    /// # State machine transitions
    ///
    /// | Current phase | Event | Next phase |
    /// |---|---|---|
    /// | `CalibrationAmbient` | `CALIBRATION_SAMPLES` consumed | `WaitingForEcho(0)` |
    /// | `WaitingForEcho(n)` | Impulse push succeeds | (same, timer armed) |
    /// | `WaitingForEcho(n)` | Echo detected, more impulses remain | `WaitingForEcho(n+1)` |
    /// | `WaitingForEcho(n)` | Echo detected, all impulses done | `Idle` → `Complete` |
    /// | `WaitingForEcho(n)` | Deadline exceeded | `Idle` → `TimedOut` |
    ///
    /// # Returns
    ///
    /// A [`RoundTripTickOutcome`] indicating whether the session should continue, has
    /// finished successfully, or has timed out.
    pub fn tick(
        &mut self,
        sample: f32,
        push_output: &mut impl FnMut(f32) -> bool,
        per_impulse_timeout: Duration,
    ) -> RoundTripTickOutcome {
        match self.phase {
            RoundTripMeasurementPhase::CalibrationAmbient => {
                if self.feed_ambient_sample(sample) {
                    // Calibration is complete; the next tick will be allowed to emit impulse 1.
                    self.phase = RoundTripMeasurementPhase::WaitingForEcho(0);
                }
                // Stay silent during calibration so the measurement does not feed input back out.
                push_output(0.0);
                RoundTripTickOutcome::Ongoing
            }
            RoundTripMeasurementPhase::WaitingForEcho(idx) => {
                if self.impulse_sent_at.is_none() {
                    // Enforce spacing between impulses so one return tail cannot contaminate the next.
                    if self
                        .next_impulse_not_before
                        .map(|t| Instant::now() < t)
                        .unwrap_or(false)
                    {
                        push_output(0.0);
                        return RoundTripTickOutcome::Ongoing;
                    }

                    // Emit exactly one impulse and start timing from that point forward.
                    if push_output(IMPULSE_AMPLITUDE) {
                        self.arm_impulse(per_impulse_timeout);
                        println!(
                            "[RT-MEASURE] Impulse {}/{} injected (threshold={:.4}).",
                            idx + 1,
                            IMPULSE_COUNT,
                            self.threshold
                        );
                    }

                    RoundTripTickOutcome::Ongoing
                } else {
                    // Stay silent while listening for the return signal to avoid creating a
                    // measurement-distorting feedback loop.
                    push_output(0.0);

                    if self.check_echo(sample) {
                        let elapsed_ms = self
                            .impulse_sent_at
                            .take()
                            .unwrap()
                            .elapsed()
                            .as_secs_f64()
                            * 1000.0;
                        self.impulse_deadline = None;
                        self.echo_durations_ms.push(elapsed_ms);

                        println!(
                            "[RT-MEASURE] Echo {}/{}: {:.2} ms",
                            idx + 1,
                            IMPULSE_COUNT,
                            elapsed_ms
                        );

                        if self.echo_durations_ms.len() >= IMPULSE_COUNT {
                            // Use the average to smooth out one-off jitter between callbacks.
                            let avg = self.echo_durations_ms.iter().sum::<f64>()
                                / self.echo_durations_ms.len() as f64;
                            println!("[RT-MEASURE] Done. Avg round-trip: {:.2} ms", avg);
                            self.phase = RoundTripMeasurementPhase::Idle;
                            RoundTripTickOutcome::Complete(avg)
                        } else {
                            // Prepare the next impulse after a short quiet gap.
                            self.next_impulse_not_before = Some(Instant::now() + INTER_IMPULSE_GAP);
                            self.phase = RoundTripMeasurementPhase::WaitingForEcho(idx + 1);
                            RoundTripTickOutcome::Ongoing
                        }
                    } else if self.is_timed_out() {
                        println!(
                            "[RT-MEASURE] TIMEOUT waiting for echo {} (threshold={:.4}).",
                            idx + 1,
                            self.threshold
                        );
                        self.phase = RoundTripMeasurementPhase::Idle;
                        RoundTripTickOutcome::TimedOut
                    } else {
                        RoundTripTickOutcome::Ongoing
                    }
                }
            }
            RoundTripMeasurementPhase::Idle => {
                // Once complete, continue writing silence until the session exits.
                push_output(0.0);
                RoundTripTickOutcome::Ongoing
            }
        }
    }
}

/// Self-contained round-trip latency measurement session.
///
/// `RoundTripLatencySession` has no fields; it acts as a namespace for the [`run`] function.
/// All state lives on the stack inside that call, making the session automatically torn down
/// when it returns — there is nothing to clean up manually.
///
/// # Thread safety
///
/// [`run`] is a blocking call designed to execute on a dedicated thread.  The caller
/// (`measure_round_trip_latency` Tauri command) clones the handler reference, releases the
/// `Mutex<AudioService>` lock, and then spawns a thread that calls this function.  This
/// means the main audio engine remains fully operational during the measurement.
///
/// [`run`]: RoundTripLatencySession::run
pub struct RoundTripLatencySession;

impl RoundTripLatencySession {
    /// Runs a complete round-trip latency measurement and returns the average in milliseconds.
    ///
    /// # What this function does
    ///
    /// 1. Determines a safe ring-buffer size from the handler's configured buffer frames
    ///    (falling back to 256 if `BufferSize::Default` is in use), then multiplies by 4 to
    ///    give the streams room to breathe during warmup and calibration.
    /// 2. Creates a dedicated input ring buffer (`i_producer` → `i_consumer`) and a dedicated
    ///    output ring buffer (`o_producer` → `o_consumer`), both completely separate from the
    ///    main loopback ring buffers.
    /// 3. Opens a CPAL input stream that pushes captured samples into `i_producer` and a CPAL
    ///    output stream that drains processed samples from `o_consumer`, then starts both.
    /// 4. Sleeps for `stream_warmup` to let the OS audio scheduler and hardware settle.
    /// 5. Drains all samples accumulated during warmup from `i_consumer` so that calibration
    ///    begins with fresh, stable ambient data.
    /// 6. Enters the main sample-processing loop, feeding each incoming sample to
    ///    [`RoundTripMeasurementState::tick`] until a terminal outcome is reached or the
    ///    `overall_deadline` expires.
    ///
    /// The `overall_deadline` is set to `per_impulse_timeout × IMPULSE_COUNT + 2 s` to
    /// account for calibration time and inter-impulse gaps while still guaranteeing the
    /// function cannot block indefinitely.
    ///
    /// # Arguments
    ///
    /// * `handler` — Audio I/O factory.  Used only to size ring buffers and build streams;
    ///   it is **not** the same handler instance that the main loopback uses concurrently.
    /// * `per_impulse_timeout` — Maximum time to wait for a single echo after the impulse is
    ///   emitted.  Recommended: 10 s for real hardware, shorter for unit tests.
    /// * `stream_warmup` — How long to sleep after starting streams before beginning
    ///   calibration.  Recommended: 1–2 s to allow ASIO/WASAPI buffers to stabilise.
    ///
    /// # Returns
    ///
    /// * `Ok(latency_ms)` — Averaged round-trip latency across all [`IMPULSE_COUNT`] cycles.
    /// * `Err(message)` — Human-readable failure reason; either a timeout, an undetectable
    ///   echo (signal too quiet or output not routed to input), or an overall deadline breach.
    pub fn run(
        handler: &dyn AudioHandlerTrait,
        per_impulse_timeout: Duration,
        stream_warmup: Duration,
    ) -> Result<f64, String> {
        // Convert CPAL buffer-size configuration into a usable frame count for our temporary
        // ring buffers. Default mode falls back to a practical, conservative size.
        fn frames_or_default(buffer_size: BufferSize) -> usize {
            match buffer_size {
                BufferSize::Fixed(frames) => frames as usize,
                BufferSize::Default => 256,
            }
        }

        // Size ring buffers relative to the configured hardware buffers so startup and warmup
        // traffic do not immediately overflow them.
        let configured_frames = frames_or_default(handler.input_config().buffer_size.clone())
            .max(frames_or_default(handler.output_config().buffer_size.clone()));
        let ringbuffer_size = (configured_frames * 4).max(512);

        let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);
        let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);

        let input_stream = handler.build_input_stream(i_producer);
        let output_stream = handler.build_output_stream(o_consumer);
        input_stream.play();
        output_stream.play();

        // Let the backend/device stack settle before starting calibration.
        println!(
            "[RT-MEASURE] Dedicated streams started. Warming up for {stream_warmup:?}..."
        );
        thread::sleep(stream_warmup);

        // Discard startup samples collected during warmup so timing starts from fresh data only.
        let mut drained = 0usize;
        while i_consumer.try_pop().is_some() {
            drained += 1;
        }
        println!(
            "[RT-MEASURE] Drained {drained} stale warmup samples. Starting calibration."
        );

        let mut state = RoundTripMeasurementState::new();
        // The full measurement may include several impulses, so the overall deadline is larger
        // than a single-impulse timeout.
        let overall_deadline =
            Instant::now() + per_impulse_timeout * IMPULSE_COUNT as u32 + Duration::from_secs(2);

        loop {
            if Instant::now() >= overall_deadline {
                return Err("Round-trip measurement timed out (no echo received).".to_string());
            }

            if let Some(sample) = i_consumer.try_pop() {
                match state.tick(sample, &mut |v| o_producer.try_push(v).is_ok(), per_impulse_timeout)
                {
                    RoundTripTickOutcome::Complete(avg_ms) => return Ok(avg_ms),
                    RoundTripTickOutcome::TimedOut => {
                        return Err(format!(
                            "Echo not detected above threshold {:.4}. Ensure output is physically routed back into input.",
                            state.threshold
                        ))
                    }
                    RoundTripTickOutcome::Ongoing => {}
                }
            } else {
                // Cooperatively yield while waiting for the input callback to deliver more data.
                thread::yield_now();
            }
        }
    }
}
