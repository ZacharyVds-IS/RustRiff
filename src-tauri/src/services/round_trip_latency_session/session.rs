//! [`RoundTripLatencySession`] — the blocking, self-contained measurement runner.
//!
//! This module owns the CPAL stream lifecycle for the round-trip measurement.
//! It opens private streams, runs the warmup, drains stale samples, then drives
//! [`RoundTripMeasurementState`] sample-by-sample until a terminal outcome is reached.

use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use crate::services::round_trip_latency_session::constants::IMPULSE_COUNT;
use crate::services::round_trip_latency_session::measurement_state::{
    RoundTripMeasurementState, RoundTripTickOutcome,
};
use cpal::BufferSize;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use std::thread;
use std::time::{Duration, Instant};

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
        fn frames_or_default(buffer_size: BufferSize) -> usize {
            match buffer_size {
                BufferSize::Fixed(frames) => frames as usize,
                BufferSize::Default => 256,
            }
        }

        let configured_frames = frames_or_default(handler.input_config().buffer_size)
            .max(frames_or_default(handler.output_config().buffer_size));
        let ringbuffer_size = (configured_frames * 4).max(512);

        let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);
        let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);

        let input_stream = handler.build_input_stream(i_producer);
        let output_stream = handler.build_output_stream(o_consumer);
        input_stream.play();
        output_stream.play();

        println!("[RT-MEASURE] Dedicated streams started. Warming up for {stream_warmup:?}...");
        thread::sleep(stream_warmup);

        let mut drained = 0usize;
        while i_consumer.try_pop().is_some() {
            drained += 1;
        }
        println!("[RT-MEASURE] Drained {drained} stale warmup samples. Starting calibration.");

        let mut state = RoundTripMeasurementState::new();
        let overall_deadline =
            Instant::now() + per_impulse_timeout * IMPULSE_COUNT as u32 + Duration::from_secs(2);

        loop {
            if Instant::now() >= overall_deadline {
                return Err("Round-trip measurement timed out (no echo received).".to_string());
            }

            if let Some(sample) = i_consumer.try_pop() {
                match state.tick(sample, &mut |v| o_producer.try_push(v).is_ok(), per_impulse_timeout) {
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
                thread::yield_now();
            }
        }
    }
}
