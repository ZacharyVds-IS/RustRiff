//! State machine types and per-sample driver for the round-trip measurement protocol.
//!
//! [`RoundTripMeasurementState`] owns all transient data for one measurement session
//! and is advanced sample-by-sample via [`RoundTripMeasurementState::tick`].

use crate::services::round_trip_latency_session::constants::{
    CALIBRATION_SAMPLES, GUARD_SAMPLES, IMPULSE_AMPLITUDE, IMPULSE_COUNT, INTER_IMPULSE_GAP,
};
use std::time::{Duration, Instant};

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
    /// `None` before the first impulse is emitted and between echo detection and the next
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

        self.threshold = (self.ambient_peak * 2.0).clamp(0.05, IMPULSE_AMPLITUDE * 0.5);

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
                    self.phase = RoundTripMeasurementPhase::WaitingForEcho(0);
                }
                push_output(0.0);
                RoundTripTickOutcome::Ongoing
            }
            RoundTripMeasurementPhase::WaitingForEcho(idx) => {
                if self.impulse_sent_at.is_none() {
                    if self
                        .next_impulse_not_before
                        .map(|t| Instant::now() < t)
                        .unwrap_or(false)
                    {
                        push_output(0.0);
                        return RoundTripTickOutcome::Ongoing;
                    }

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
                    push_output(0.0);

                    if self.check_echo(sample) {
                        let elapsed_ms =
                            self.impulse_sent_at.take().unwrap().elapsed().as_secs_f64() * 1000.0;
                        self.impulse_deadline = None;
                        self.echo_durations_ms.push(elapsed_ms);

                        println!(
                            "[RT-MEASURE] Echo {}/{}: {:.2} ms",
                            idx + 1,
                            IMPULSE_COUNT,
                            elapsed_ms
                        );

                        if self.echo_durations_ms.len() >= IMPULSE_COUNT {
                            let avg = self.echo_durations_ms.iter().sum::<f64>()
                                / self.echo_durations_ms.len() as f64;
                            println!("[RT-MEASURE] Done. Avg round-trip: {:.2} ms", avg);
                            self.phase = RoundTripMeasurementPhase::Idle;
                            RoundTripTickOutcome::Complete(avg)
                        } else {
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
                push_output(0.0);
                RoundTripTickOutcome::Ongoing
            }
        }
    }
}

impl Default for RoundTripMeasurementState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn complete_calibration(state: &mut RoundTripMeasurementState) {
        for _ in 0..CALIBRATION_SAMPLES {
            state.tick(0.0, &mut |_| true, Duration::from_secs(10));
        }
    }

    fn complete_calibration_with_peak(state: &mut RoundTripMeasurementState, peak: f32) {
        state.tick(peak, &mut |_| true, Duration::from_secs(10));
        for _ in 1..CALIBRATION_SAMPLES {
            state.tick(0.0, &mut |_| true, Duration::from_secs(10));
        }
    }

    fn emit_impulse(state: &mut RoundTripMeasurementState, timeout: Duration) {
        state.tick(0.0, &mut |_| true, timeout);
    }

    fn drain_guard(state: &mut RoundTripMeasurementState) {
        for _ in 0..GUARD_SAMPLES {
            state.tick(0.0, &mut |_| true, Duration::from_secs(10));
        }
    }

    fn drive_to_idle_via_timeout(state: &mut RoundTripMeasurementState) {
        complete_calibration(state);
        emit_impulse(state, Duration::ZERO);
        for _ in 0..(GUARD_SAMPLES + 2) {
            if matches!(
                state.tick(0.0, &mut |_| true, Duration::ZERO),
                RoundTripTickOutcome::TimedOut
            ) {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Initial state
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod initial_state {
        use super::*;

        #[test]
        fn new_starts_in_calibration_phase() {
            let state = RoundTripMeasurementState::new();
            assert_eq!(state.phase, RoundTripMeasurementPhase::CalibrationAmbient);
        }

        #[test]
        fn new_has_zero_threshold() {
            let state = RoundTripMeasurementState::new();
            assert_eq!(state.threshold, 0.0);
        }

        #[test]
        fn new_has_no_impulse_timer() {
            let state = RoundTripMeasurementState::new();
            assert!(state.impulse_sent_at.is_none());
        }
    }

    // -----------------------------------------------------------------------
    // Calibration phase
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod calibration_phase {
        use super::*;

        #[test]
        fn stays_in_calibration_until_enough_samples() {
            let mut state = RoundTripMeasurementState::new();
            for _ in 0..(CALIBRATION_SAMPLES - 1) {
                let outcome = state.tick(0.0, &mut |_| true, Duration::from_secs(10));
                assert!(matches!(outcome, RoundTripTickOutcome::Ongoing));
                assert_eq!(state.phase, RoundTripMeasurementPhase::CalibrationAmbient);
            }
        }

        #[test]
        fn transitions_to_waiting_for_echo_after_calibration_samples() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            assert_eq!(state.phase, RoundTripMeasurementPhase::WaitingForEcho(0));
        }

        #[test]
        fn outputs_silence_during_calibration() {
            let mut state = RoundTripMeasurementState::new();
            let mut emitted = Vec::new();
            for _ in 0..CALIBRATION_SAMPLES {
                state.tick(
                    0.0,
                    &mut |v| {
                        emitted.push(v);
                        true
                    },
                    Duration::from_secs(10),
                );
            }
            assert!(
                emitted.iter().all(|&v| v == 0.0),
                "calibration must output only silence"
            );
        }

        #[test]
        fn threshold_is_double_the_ambient_peak_clamped_to_min() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            assert!((state.threshold - 0.05).abs() < 1e-6);
        }

        #[test]
        fn threshold_is_double_ambient_peak_when_above_minimum() {
            let mut state = RoundTripMeasurementState::new();
            let peak = 0.1_f32;
            complete_calibration_with_peak(&mut state, peak);
            let expected = (peak * 2.0).clamp(0.05, IMPULSE_AMPLITUDE * 0.5);
            assert!((state.threshold - expected).abs() < 1e-6);
        }

        #[test]
        fn threshold_is_clamped_to_max_when_ambient_peak_is_very_high() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration_with_peak(&mut state, IMPULSE_AMPLITUDE);
            assert!((state.threshold - IMPULSE_AMPLITUDE * 0.5).abs() < 1e-6);
        }
    }

    // -----------------------------------------------------------------------
    // WaitingForEcho — impulse emission
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod impulse_emission {
        use super::*;

        #[test]
        fn first_tick_after_calibration_emits_impulse() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);

            let mut emitted_values: Vec<f32> = Vec::new();
            state.tick(
                0.0,
                &mut |v| {
                    emitted_values.push(v);
                    true
                },
                Duration::from_secs(10),
            );

            assert!(
                emitted_values.contains(&IMPULSE_AMPLITUDE),
                "first post-calibration tick must emit the impulse"
            );
        }

        #[test]
        fn impulse_timer_is_armed_after_emission() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            emit_impulse(&mut state, Duration::from_secs(10));
            assert!(state.impulse_sent_at.is_some());
        }

        #[test]
        fn output_is_silence_while_waiting_for_echo() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            emit_impulse(&mut state, Duration::from_secs(10));

            let mut emitted: Vec<f32> = Vec::new();
            state.tick(
                0.0,
                &mut |v| {
                    emitted.push(v);
                    true
                },
                Duration::from_secs(10),
            );
            assert!(emitted.iter().all(|&v| v == 0.0));
        }
    }

    // -----------------------------------------------------------------------
    // WaitingForEcho — echo detection
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod echo_detection {
        use super::*;

        #[test]
        fn echo_is_not_detected_during_guard_window() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            emit_impulse(&mut state, Duration::from_secs(10));

            let outcome = state.tick(1.0, &mut |_| true, Duration::from_secs(10));
            assert!(matches!(outcome, RoundTripTickOutcome::Ongoing));
            assert_eq!(state.phase, RoundTripMeasurementPhase::WaitingForEcho(0));
        }

        #[test]
        fn echo_above_threshold_detected_after_guard_window() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            emit_impulse(&mut state, Duration::from_secs(10));
            drain_guard(&mut state);

            let outcome = state.tick(1.0, &mut |_| true, Duration::from_secs(10));
            assert!(!matches!(outcome, RoundTripTickOutcome::TimedOut));
        }

        #[test]
        fn sample_below_threshold_is_not_accepted_as_echo() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            emit_impulse(&mut state, Duration::from_secs(10));
            drain_guard(&mut state);

            let outcome = state.tick(0.01, &mut |_| true, Duration::from_secs(10));
            assert!(matches!(outcome, RoundTripTickOutcome::Ongoing));
        }

        #[test]
        fn completing_all_impulses_returns_complete_with_positive_average() {
            let mut state = RoundTripMeasurementState::new();
            let test_deadline = Instant::now() + Duration::from_secs(5);

            loop {
                assert!(
                    Instant::now() < test_deadline,
                    "test timed out — Complete outcome was never reached"
                );

                // Feed a strong signal whenever we are listening for an echo so it is
                // detected as soon as the guard window expires.  Inter-impulse gaps
                // (200 ms) pass naturally while we spin.
                let input = match state.phase {
                    RoundTripMeasurementPhase::WaitingForEcho(_)
                        if state.impulse_sent_at.is_some() =>
                    {
                        1.0
                    }
                    _ => 0.0,
                };

                match state.tick(input, &mut |_| true, Duration::from_secs(10)) {
                    RoundTripTickOutcome::Complete(avg) => {
                        assert!(avg >= 0.0, "average round-trip must be non-negative");
                        return;
                    }
                    RoundTripTickOutcome::TimedOut => panic!("unexpected timeout during test"),
                    RoundTripTickOutcome::Ongoing => {}
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // WaitingForEcho — timeout
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod timeout {
        use super::*;

        #[test]
        fn zero_timeout_causes_timed_out_outcome_after_impulse() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            emit_impulse(&mut state, Duration::ZERO);

            let mut outcome = RoundTripTickOutcome::Ongoing;
            for _ in 0..(GUARD_SAMPLES + 2) {
                outcome = state.tick(0.0, &mut |_| true, Duration::ZERO);
                if matches!(outcome, RoundTripTickOutcome::TimedOut) {
                    break;
                }
            }
            assert!(
                matches!(outcome, RoundTripTickOutcome::TimedOut),
                "expected TimedOut but measurement kept running"
            );
        }

        #[test]
        fn timed_out_transitions_to_idle() {
            let mut state = RoundTripMeasurementState::new();
            complete_calibration(&mut state);
            emit_impulse(&mut state, Duration::ZERO);

            for _ in 0..(GUARD_SAMPLES + 2) {
                if matches!(
                    state.tick(0.0, &mut |_| true, Duration::ZERO),
                    RoundTripTickOutcome::TimedOut
                ) {
                    break;
                }
            }
            assert_eq!(state.phase, RoundTripMeasurementPhase::Idle);
        }
    }

    // -----------------------------------------------------------------------
    // Idle phase
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod idle_phase {
        use super::*;

        #[test]
        fn idle_phase_outputs_silence() {
            let mut state = RoundTripMeasurementState::new();
            drive_to_idle_via_timeout(&mut state);

            let mut emitted: Vec<f32> = Vec::new();
            state.tick(
                0.0,
                &mut |v| {
                    emitted.push(v);
                    true
                },
                Duration::from_secs(10),
            );
            assert!(emitted.iter().all(|&v| v == 0.0));
        }

        #[test]
        fn idle_phase_returns_ongoing() {
            let mut state = RoundTripMeasurementState::new();
            drive_to_idle_via_timeout(&mut state);
            let outcome = state.tick(0.0, &mut |_| true, Duration::from_secs(10));
            assert!(matches!(outcome, RoundTripTickOutcome::Ongoing));
        }
    }
}
