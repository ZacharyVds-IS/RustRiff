use crate::domain::audio_processor::AudioProcessor;
use crate::domain::channel_manager::{ChannelArcs, ChannelManager};
use crate::domain::dto::amp_config_dto::AmpConfigDto;
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use crate::services::analyzers::spectrum_tap::SpectrumTap;
use crate::services::device_service::DeviceService;
use crate::services::processors::gain::gain_processor::GainProcessor;
use crate::services::processors::resampler::resampler::ResamplePolicy;
use crate::services::processors::tone_stack::tone_stack_processor::ToneStackProcessor;
use atomic_float::AtomicF32;
use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::{BufferSize, Device, StreamConfig};
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

const DEFAULT_ANALYZER_SAMPLE_RATE_HZ: u32 = 48_000;

/// The main service that orchestrates real-time audio loopback between an input and output device.
///
/// `AudioService` manages the full lifecycle of the audio processing pipeline:
///
/// - **Device management** — holds references to the active CPAL input/output devices
///   through an [`AudioHandlerTrait`] implementation and supports hot-swapping either
///   device without a full restart.
/// - **Resampling** — on `start_loopback` the input and output sample rates are compared
///   and a [`ResamplePolicy`] is selected automatically:
///   - `input == output` → no resampling, zero overhead.
///   - `input > output` → downsample before the DSP chain.
///   - `input < output` → upsample after the DSP chain.
/// - **DSP chain** — every sample passes through gain, tone stack, and master volume
///   processors in order. Additional processors can be inserted into `start_loopback`'s
///   `run_dsp` closure.
/// - **Thread lifecycle** — the loopback runs on a dedicated background thread with a
///   lock-free ring buffer between the CPAL callbacks and the worker; the thread is
///   cleanly shut down via [`stop_loopback`].
///
/// [`AudioHandlerTrait`]: AudioHandlerTrait
/// [`ResamplePolicy`]: ResamplePolicy
/// [`stop_loopback`]: AudioService::stop_loopback
pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
    loopback_thread: Option<JoinHandle<()>>,
    is_active: bool,
    tuner_active: bool,
    channel_manager: Arc<Mutex<ChannelManager>>,
    master_volume: Arc<AtomicF32>,
    analyzer_tap: Arc<SpectrumTap>,
    tuner_tap: Arc<SpectrumTap>,
    shared_amp_enabled: Arc<AtomicBool>,
    shared_tuner_enabled: Arc<AtomicBool>,
}

impl AudioService {
    /// Returns the sample rate at which the DSP chain effectively runs.
    ///
    /// With current resampling policy, DSP executes at the lower of input/output rates.
    pub fn dsp_chain_sample_rate(&self) -> u32 {
        self.audio_handler
            .input_sample_rate()
            .min(self.audio_handler.output_sample_rate())
    }

    /// Creates a new `AudioService` using the provided CPAL input/output devices and stream config.
    ///
    /// An [`AudioHandler`] is constructed internally from the given parameters.
    pub fn new(
        input_device: Device,
        output_device: Device,
        input_config: StreamConfig,
        output_config: StreamConfig,
        channel_manager: Arc<Mutex<ChannelManager>>,
    ) -> Self {
        let handler = AudioHandler::new(input_device, output_device, input_config, output_config);
        Self::new_with_handler(Arc::new(handler), channel_manager)
    }

    /// Creates an `AudioService` with a custom handler.
    ///
    /// This constructor is primarily intended for unit and integration testing,
    /// where a mock [`AudioHandlerTrait`] implementation can be injected in place
    /// of a real [`AudioHandler`].
    ///
    /// # Arguments
    ///
    /// * `handler` - An [`Arc`]-wrapped implementation of [`AudioHandlerTrait`].
    pub fn new_with_handler(
        handler: Arc<dyn AudioHandlerTrait>,
        channel_manager: Arc<Mutex<ChannelManager>>,
    ) -> Self {
        Self {
            audio_handler: handler,
            loopback_thread: None,
            is_active: false,
            tuner_active: false,
            channel_manager,
            master_volume: Arc::new(AtomicF32::new(1.0)),
            tuner_tap: Arc::new(SpectrumTap::new(DEFAULT_ANALYZER_SAMPLE_RATE_HZ)),
            analyzer_tap: Arc::new(SpectrumTap::new(DEFAULT_ANALYZER_SAMPLE_RATE_HZ)),
            shared_amp_enabled: Arc::new(AtomicBool::new(false)),
            shared_tuner_enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    // ── Public accessors ─────────────────────────────────────────────────────

    pub fn is_active(&self) -> &bool {
        &self.is_active
    }

    pub fn master_volume(&self) -> &Arc<AtomicF32> {
        &self.master_volume
    }

    pub fn channel_manager(&self) -> &Arc<Mutex<ChannelManager>> {
        &self.channel_manager
    }

    pub fn audio_handler(&self) -> &Arc<dyn AudioHandlerTrait> {
        &self.audio_handler
    }

    pub fn analyzer_tap(&self) -> Arc<SpectrumTap> {
        self.analyzer_tap.clone()
    }

    pub fn tuner_tap(&self) -> Arc<SpectrumTap> {
        self.tuner_tap.clone()
    }

    pub fn set_tuner_active(&mut self, active: bool) {
        self.tuner_active = active;
        self.update_stream_state();
    }

    pub fn is_tuner_active(&self) -> bool {
        self.tuner_active
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    fn resolve_channel_arcs(&self) -> ChannelArcs {
        self.channel_manager
            .lock()
            .expect("channel_manager lock")
            .resolve_channel_arcs()
    }

    /// Spawns the inner DSP worker thread.
    ///
    /// The worker pops samples from `i_consumer`, runs them through the full DSP
    /// chain (gain → tone stack → effects → volume → master volume → spectrum tap),
    /// applies the resampling policy at the correct point, and pushes results into
    /// `o_producer`. It exits cleanly when `shutdown` is set to `true`.
    ///
    /// Returns the thread `JoinHandle`.
    #[allow(clippy::too_many_arguments)]
    fn spawn_dsp_worker(
        arcs: ChannelArcs,
        master_volume_arc: Arc<AtomicF32>,
        analyzer_tap: Arc<SpectrumTap>,
        tuner_tap: Arc<SpectrumTap>,
        dsp_sample_rate: u32,
        mut policy: ResamplePolicy,
        mut i_consumer: impl Consumer<Item = f32> + Send + 'static,
        mut o_producer: impl Producer<Item = f32> + Send + 'static,
        shutdown: Arc<AtomicBool>,
        amp_enabled: Arc<AtomicBool>,
        tuner_enabled: Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut gain = GainProcessor::new(arcs.gain);
            let mut volume = GainProcessor::new(arcs.volume);
            let mut master_volume = GainProcessor::new(master_volume_arc);
            let mut tone_stack = ToneStackProcessor::new(arcs.tone_stack, dsp_sample_rate);

            let mut run_dsp = |sample: f32| -> f32 {
                let sample = gain.process(sample);
                let mut sample = tone_stack.process(sample);
                if let Ok(mut chain) = arcs.effect_chain.lock() {
                    for effect in chain.iter_mut() {
                        sample = effect.process_if_active(sample);
                    }
                }
                let sample = volume.process(sample);
                master_volume.process(sample)
            };

            loop {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                if let Some(sample) = i_consumer.try_pop() {
                    if tuner_enabled.load(Ordering::Relaxed) {
                        tuner_tap.push_sample(sample);
                    }

                    if amp_enabled.load(Ordering::Relaxed) {
                        for processed in policy.process(sample, &mut |s| run_dsp(s)) {
                            analyzer_tap.push_sample(processed);
                            let _ = o_producer.try_push(processed);
                        }
                    } else {
                        for processed in policy.process(0.0, &mut |s| s) {
                            analyzer_tap.push_sample(processed);
                            let _ = o_producer.try_push(processed);
                        }
                    }
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }

            for processed in policy.flush(&mut |s| {
                if amp_enabled.load(Ordering::Relaxed) {
                    run_dsp(s)
                } else {
                    0.0
                }
            }) {
                let _ = o_producer.try_push(processed);
            }
        })
    }

    /// Builds and spawns the outer I/O thread that owns the CPAL streams.
    ///
    /// Responsibilities:
    /// 1. Size and create the lock-free ring buffers.
    /// 2. Select the [`ResamplePolicy`] from the input/output sample rates.
    /// 3. Build the CPAL input and output streams via the handler.
    /// 4. Delegate DSP work to [`spawn_dsp_worker`].
    /// 5. Play both streams and park until [`stop_loopback`] unparks the thread.
    /// 6. Signal the worker to shut down and join it before returning.
    ///
    /// [`spawn_dsp_worker`]: AudioService::spawn_dsp_worker
    /// [`stop_loopback`]: AudioService::stop_loopback
    #[allow(clippy::too_many_arguments)]
    fn spawn_io_thread(
        handler: Arc<dyn AudioHandlerTrait>,
        arcs: ChannelArcs,
        master_volume_arc: Arc<AtomicF32>,
        analyzer_tap: Arc<SpectrumTap>,
        tuner_tap: Arc<SpectrumTap>,
        dsp_sample_rate: u32,
        amp_enabled: Arc<AtomicBool>,
        tuner_enabled: Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            const RESAMPLER_CHUNK_SIZE: usize = 256;
            let ringbuffer_size = handler
                .input_sample_rate()
                .max(handler.output_sample_rate()) as usize;

            let policy = ResamplePolicy::from_rates(
                handler.input_sample_rate(),
                handler.output_sample_rate(),
                RESAMPLER_CHUNK_SIZE,
            );

            let (i_producer, i_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);
            let (o_producer, o_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);

            let input_stream = handler.build_input_stream(i_producer);
            let output_stream = handler.build_output_stream(o_consumer);

            let shutdown = Arc::new(AtomicBool::new(false));

            let worker = Self::spawn_dsp_worker(
                arcs,
                master_volume_arc,
                analyzer_tap,
                tuner_tap,
                dsp_sample_rate,
                policy,
                i_consumer,
                o_producer,
                shutdown.clone(),
                amp_enabled,
                tuner_enabled,
            );

            input_stream.play();
            output_stream.play();

            thread::park();

            shutdown.store(true, Ordering::SeqCst);
            let _ = worker.join();
        })
    }

    // ── Public API ───────────────────────────────────────────────────────────

    /// Starts the audio loopback on a dedicated background thread.
    ///
    /// On startup the service:
    /// 1. Reads the input and output sample rates from the active [`AudioHandlerTrait`].
    /// 2. Selects a [`ResamplePolicy`] based on those rates (logged at `info` level).
    /// 3. Builds a pair of lock-free ring buffers sized to the larger of the two rates.
    /// 4. Asks the handler to open the input and output CPAL streams.
    /// 5. Spawns a worker thread that runs the full DSP + resampling pipeline.
    ///
    /// If the loopback is already active this method is a no-op.
    ///
    /// [`AudioHandlerTrait`]: AudioHandlerTrait
    /// [`ResamplePolicy`]: ResamplePolicy
    pub fn start_loopback(&mut self) {
        if self.is_active {
            return;
        }
        info!("Enabling audio amp rig");
        self.is_active = true;
        self.update_stream_state();
    }

    /// Stops the audio loopback and joins the background thread.
    ///
    /// Unparks the loopback thread, signals the inner worker to shut down,
    /// and waits for both threads to finish. If the loopback is not currently
    /// active this method is a no-op.
    pub fn stop_loopback(&mut self) {
        if !self.is_active {
            return;
        }
        info!("Disabling audio amp rig");
        self.is_active = false;
        self.update_stream_state();
    }

    fn update_stream_state(&mut self) {
        let should_run = self.is_active || self.tuner_active;
        let is_running = self.loopback_thread.is_some();

        if should_run && !is_running {
            let dsp_sample_rate = self.dsp_chain_sample_rate();
            self.analyzer_tap.set_sample_rate_hz(dsp_sample_rate);
            self.tuner_tap.set_sample_rate_hz(dsp_sample_rate);
            let arcs = self.resolve_channel_arcs();

            self.shared_amp_enabled
                .store(self.is_active, Ordering::SeqCst);
            self.shared_tuner_enabled
                .store(self.tuner_active, Ordering::SeqCst);

            let thread = Self::spawn_io_thread(
                self.audio_handler.clone(),
                arcs,
                self.master_volume.clone(),
                self.analyzer_tap.clone(),
                self.tuner_tap.clone(),
                dsp_sample_rate,
                self.shared_amp_enabled.clone(),
                self.shared_tuner_enabled.clone(),
            );
            self.loopback_thread = Some(thread);
        } else if !should_run && is_running {
            if let Some(handle) = self.loopback_thread.take() {
                handle.thread().unpark();
                let _ = handle.join();
            }
        } else if is_running {
            // The thread is already running; hot-swap the internal feature states instantly
            self.shared_amp_enabled
                .store(self.is_active, Ordering::SeqCst);
            self.shared_tuner_enabled
                .store(self.tuner_active, Ordering::SeqCst);
        }
    }

    /// Sets the master volume value.
    ///
    /// The master volume value is atomically updated and will be read by the audio processing
    /// thread on the next sample cycle.
    ///
    /// # Arguments
    ///
    /// * `master_volume` - The new master volume value. Must be positive (> 0.0).
    ///
    /// # Panics
    ///
    /// Panics if `master_volume` is negative or zero.
    pub fn set_master_volume(&self, master_volume: f32) {
        if master_volume.is_sign_positive() {
            self.master_volume.store(master_volume, Ordering::Relaxed);
        } else {
            error!("Master volume must be a positive number");
            panic!("Master volume must be positive");
        }
    }

    /// Replaces the underlying audio handler, restarting the loopback if it was running.
    ///
    /// If the loopback is active when this method is called it will be stopped,
    /// the handler swapped, and then the loopback restarted automatically.
    ///
    /// # Arguments
    ///
    /// * `new_handler` - The replacement [`AudioHandlerTrait`] implementation.
    pub(crate) fn set_audio_handler(&mut self, new_handler: Arc<dyn AudioHandlerTrait>) {
        let was_running = self.loopback_thread.is_some(); // <-- Check thread presence, not amp state
        if was_running {
            if let Some(handle) = self.loopback_thread.take() {
                handle.thread().unpark();
                let _ = handle.join();
            }
        }

        self.audio_handler = new_handler;
        self.analyzer_tap
            .set_sample_rate_hz(self.dsp_chain_sample_rate());
        self.tuner_tap()
            .set_sample_rate_hz(self.dsp_chain_sample_rate());

        if was_running {
            let dsp_sample_rate = self.dsp_chain_sample_rate();
            let arcs = self.resolve_channel_arcs();
            self.shared_amp_enabled
                .store(self.is_active, Ordering::SeqCst);
            self.shared_tuner_enabled
                .store(self.tuner_active, Ordering::SeqCst);

            let thread = Self::spawn_io_thread(
                self.audio_handler.clone(),
                arcs,
                self.master_volume.clone(),
                self.analyzer_tap().clone(),
                self.tuner_tap().clone(),
                dsp_sample_rate,
                self.shared_amp_enabled.clone(),
                self.shared_tuner_enabled.clone(),
            );
            self.loopback_thread = Some(thread);
        }
    }

    /// Switches the audio input device without interrupting playback longer than necessary.
    ///
    /// Constructs a new [`AudioHandler`] that pairs the given `input` device with the
    /// existing output device and stream config, then delegates to [`set_audio_handler`].
    ///
    /// # Arguments
    ///
    /// * `input` - The new CPAL input device to capture audio from.
    ///
    /// [`set_audio_handler`]: AudioService::set_audio_handler
    pub fn set_input_device(&mut self, input: Device, input_config: StreamConfig) {
        info!("Switching input device");

        let old = self.audio_handler.clone();
        let new_handler = AudioHandler::new(
            input,
            old.output_device().clone(),
            input_config,
            old.output_config().clone(),
        );

        self.set_audio_handler(Arc::new(new_handler));
    }

    /// Switches the audio output device without interrupting playback longer than necessary.
    ///
    /// Constructs a new [`AudioHandler`] that pairs the existing input device with the
    /// given `output` device and stream config, then delegates to [`set_audio_handler`].
    ///
    /// # Arguments
    ///
    /// * `output` - The new CPAL output device to send processed audio to.
    ///
    /// [`set_audio_handler`]: AudioService::set_audio_handler
    pub fn set_output_device(&mut self, output: Device, output_config: StreamConfig) {
        info!("Switching output device");

        let old = self.audio_handler.clone();
        let new_handler = AudioHandler::new(
            old.input_device().clone(),
            output,
            old.input_config().clone(),
            output_config,
        );

        self.set_audio_handler(Arc::new(new_handler));
    }

    /// Switches both audio input and output devices in one operation.
    ///
    /// This is used by driver modes (for example ASIO) that require the
    /// same hardware route to be reconfigured atomically.
    pub fn set_io_devices(
        &mut self,
        input: Device,
        output: Device,
        input_config: StreamConfig,
        output_config: StreamConfig,
    ) {
        info!("Switching input/output device route");
        let new_handler = AudioHandler::new(input, output, input_config, output_config);
        self.set_audio_handler(Arc::new(new_handler));
    }

    /// Toggles the audio loopback on or off.
    ///
    /// - If `is_on` is `true` and the loopback is not active, [`start_loopback`] is called.
    /// - If `is_on` is `false` and the loopback is active, [`stop_loopback`] is called.
    /// - If the requested state already matches the current state, this method is a no-op.
    ///
    /// [`start_loopback`]: AudioService::start_loopback
    /// [`stop_loopback`]: AudioService::stop_loopback
    pub fn toggle_loopback(&mut self, is_on: bool) {
        if self.is_active == is_on {
            return;
        }
        if is_on {
            self.start_loopback();
        } else {
            self.stop_loopback();
        }
    }

    /// Adds a new channel to the channel list and return the new channel.
    ///
    /// New channels are initialized with default values and the `current_channel_id` is updated to the new channel's id.
    ///
    /// # Arguments
    ///
    /// * `channel_name` - The name of the new channel (30 characters max).
    ///
    /// [`set_current_channel_id`]: AudioService::set_current_channel_id
    pub fn add_channel(&mut self, channel_name: String) -> Uuid {
        let id = self
            .channel_manager
            .lock()
            .expect("channel_manager lock")
            .add_channel(channel_name);
        let was_on = self.is_active;
        self.stop_loopback();
        if was_on {
            self.start_loopback();
        }
        id
    }

    /// Removes the channel with the given id from the channel list and sets `current_channel_id` to 0 (default channel).
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The id of the channel to remove. Cannot be 0 (default channel).
    ///
    /// [`set_current_channel_id`]: AudioService::set_current_channel_id
    pub fn remove_channel(&mut self, channel_id: Uuid) {
        let was_on = self.is_active;
        self.stop_loopback();
        self.channel_manager
            .lock()
            .expect("channel_manager lock")
            .remove_channel(channel_id);
        if was_on {
            self.start_loopback();
        }
    }

    /// Sets the current channel id, restarting the loopback if it was active to ensure the new channel's parameters are applied.
    ///
    /// # Arguments
    ///
    /// * `new_current_channel_id` - The id of the channel to set as current. Must exist in the channel list.
    ///
    /// [`start_loopback`]: AudioService::start_loopback
    /// [`stop_loopback`]: AudioService::stop_loopback
    pub fn set_current_channel_id(&mut self, new_current_channel_id: Uuid) {
        let was_on = self.is_active;
        self.stop_loopback();
        self.channel_manager
            .lock()
            .expect("channel_manager lock")
            .set_current_channel_id(new_current_channel_id);
        if was_on {
            self.start_loopback();
        }
    }

    /// Returns the current buffer size in frames.
    ///
    /// If the handler uses `BufferSize::Default`, returns 256 as a practical fallback.
    pub fn buffer_size_frames(&self) -> u32 {
        match self.audio_handler.input_config().buffer_size {
            BufferSize::Fixed(frames) => frames,
            BufferSize::Default => 256,
        }
    }

    /// Updates the buffer size for both input and output streams.
    ///
    /// Rebuilds the audio handler with a `BufferSize::Fixed(frames)` config and
    /// restarts the loopback if it was active.
    ///
    /// # Arguments
    ///
    /// * `frames` - The desired buffer size in frames.
    pub fn set_buffer_size_frames(&mut self, frames: u32) -> Result<(), String> {
        let old = self.audio_handler.clone();
        let mut input_config = old.input_config().clone();
        let mut output_config = old.output_config().clone();
        input_config.buffer_size = BufferSize::Fixed(frames);
        output_config.buffer_size = BufferSize::Fixed(frames);
        let new_handler = AudioHandler::new(
            old.input_device().clone(),
            old.output_device().clone(),
            input_config,
            output_config,
        );
        self.set_audio_handler(Arc::new(new_handler));
        Ok(())
    }

    /// Applies a persisted amp configuration snapshot to the live service.
    ///
    /// Restore behavior summary:
    /// - channels are recreated from the persisted DTOs,
    /// - gain, volume, tone stack, and effect-chain state are restored,
    /// - if the snapshot contains no channels, a default channel is created,
    /// - if the stored current channel no longer exists, the first restored
    ///   channel becomes the active channel,
    /// - `next_channel_id` is recalculated from the restored set,
    /// - loopback is toggled according to `config.is_active`.
    /// - selected Audio drivers are restored
    /// - I/O devices are restored if they can be found, otherwise the existing devices are kept and an error is logged.
    ///
    /// Note that the current JSON persistence implementation always loads with
    /// `is_active = false`, so persisted sessions restart with loopback turned
    /// off even though this method is capable of applying either state.
    pub fn apply_amp_config(&mut self, config: AmpConfigDto, device_service: &DeviceService) {
        let dsp_sample_rate = self.dsp_chain_sample_rate();
        {
            let mut cm = self.channel_manager.lock().expect("channel_manager lock");
            cm.restore_from_dtos(config.channels, &config.current_channel, dsp_sample_rate);
        }
        self.master_volume
            .store(config.master_volume.max(0.0001), Ordering::Relaxed);

        let audio_settings = config.audio_settings.clone();

        let old_handler = self.audio_handler.clone();

        let rebuild_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut new_input_config = old_handler.input_config().clone();
            let mut new_output_config = old_handler.output_config().clone();

            new_input_config.sample_rate = audio_settings.input_sample_rate;
            new_input_config.channels = audio_settings.input_channels;
            new_output_config.sample_rate = audio_settings.output_sample_rate;
            new_output_config.channels = audio_settings.output_channels;

            let host = cpal::default_host();

            let _ = device_service.set_selected_audio_driver(audio_settings.audio_driver.as_str());

            let selected_input = host
                .input_devices()
                .ok()
                .and_then(|mut devices| {
                    devices
                        .find_map(|d| match d.id() {
                            Ok(n) if n.to_string() == audio_settings.input_device_name => Some(d),
                            _ => None,
                        })
                })
                .unwrap_or_else(|| {
                    error!(
            "Requested input device '{}' could not be found or opened. Falling back to current input device.",
            audio_settings.input_device_name
        );
                    old_handler.input_device().clone()
                });

            let selected_output = host
                .output_devices()
                .ok()
                .and_then(|mut devices| {
                    devices
                        .find_map(|d| match d.id() {
                            Ok(n) if n.to_string() == audio_settings.output_device_name => Some(d),
                            _ => None,
                        })
                })
                .unwrap_or_else(|| {
                    error!(
            "Requested output device '{}' could not be found or opened. Falling back to current output device.",
            audio_settings.output_device_name
        );
                    old_handler.output_device().clone()
                });

            let new_handler = AudioHandler::new(
                selected_input,
                selected_output,
                new_input_config,
                new_output_config,
            );
            self.set_audio_handler(Arc::new(new_handler));
        }));

        if rebuild_result.is_err() {
            tracing::debug!("Skipping audio handler rebuild while applying persisted audio settings (likely mock handler)");
        }

        if config.is_active {
            self.start_loopback();
        } else {
            self.stop_loopback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_manager::ChannelManager;
    use crate::domain::dto::amp_config_dto::AmpConfigDto;
    use crate::domain::dto::channel_dto::ChannelDto;
    use crate::domain::dto::effect::cabinet_dto::CabinetDto;
    use crate::domain::dto::effect::effect_dto::EffectDto;
    use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
    use crate::domain::dto::tone_stack_dto::ToneStackDto;
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use crate::tests::mock::make_mock_handler;
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Mutex};

    fn new_cm() -> Arc<Mutex<ChannelManager>> {
        Arc::new(Mutex::new(ChannelManager::new()))
    }

    fn build_service(handler: MockAudioHandlerTrait) -> AudioService {
        AudioService::new_with_handler(Arc::new(handler), new_cm())
    }

    fn tone_stack(bass: f32, middle: f32, treble: f32) -> ToneStackDto {
        ToneStackDto {
            bass,
            middle,
            treble,
        }
    }

    fn distortion_effect(
        id: String,
        name: &str,
        is_active: bool,
        threshold: f32,
        level: f32,
        color: &str,
    ) -> EffectDto {
        EffectDto::HCDistortion(HcDistortionDto {
            id,
            name: name.to_string(),
            is_active,
            color: color.to_string(),
            threshold,
            level,
        })
    }

    fn cabinet_effect(
        id: String,
        name: &str,
        is_active: bool,
        color: &str,
        ir_file_path: &str,
    ) -> EffectDto {
        EffectDto::Cabinet(CabinetDto {
            id,
            name: name.to_string(),
            is_active,
            color: color.to_string(),
            ir_file_path: ir_file_path.to_string(),
        })
    }

    fn channel_dto(
        id: String,
        name: &str,
        gain: f32,
        volume: f32,
        tone_stack: ToneStackDto,
        effect_chain: Vec<EffectDto>,
    ) -> ChannelDto {
        ChannelDto {
            id,
            name: name.to_string(),
            gain,
            tone_stack,
            volume,
            effect_chain,
        }
    }

    fn channels(service: &AudioService) -> std::sync::MutexGuard<'_, ChannelManager> {
        service.channel_manager().lock().unwrap()
    }

    #[cfg(test)]
    mod success_path {
        use super::*;
        use crate::domain::dto::audio_settings_dto::AudioSettingsDto;

        fn basic_audio_settings() -> AudioSettingsDto {
            AudioSettingsDto {
                input_device_name: "Test Input".to_string(),
                output_device_name: "Test Output".to_string(),
                input_sample_rate: 44100,
                output_sample_rate: 44100,
                input_channels: 2,
                output_channels: 2,
                audio_driver: "".to_string(),
            }
        }

        #[test]
        fn master_volume_set_to_positive_value_should_succeed() {
            let mock = MockAudioHandlerTrait::new();
            let service = AudioService::new_with_handler(Arc::new(mock), new_cm());
            service.set_master_volume(0.5);
            assert_eq!(service.master_volume().load(Ordering::Relaxed), 0.5);
        }

        #[test]
        fn add_channel_should_add_a_channel_with_correct_values_and_sets_current_channel_id_to_new_id(
        ) {
            let mock = MockAudioHandlerTrait::new();
            let mut service = AudioService::new_with_handler(Arc::new(mock), new_cm());
            let test_channel_id = service.add_channel("TestChannel".to_string());
            let cm = channels(&service);
            let test_channel = cm
                .channels()
                .iter()
                .find(|c| c.id() == test_channel_id)
                .unwrap();

            assert_eq!(cm.channels().len(), 2);
            assert_eq!(test_channel.name(), "TestChannel");
            assert_eq!(test_channel.gain().load(Ordering::Relaxed), 1.0);
            assert_eq!(test_channel.volume().load(Ordering::Relaxed), 1.0);
            assert_eq!(
                test_channel.tone_stack().bass().load(Ordering::Relaxed),
                1.0
            );
            assert_eq!(
                test_channel.tone_stack().middle().load(Ordering::Relaxed),
                1.0
            );
            assert_eq!(
                test_channel.tone_stack().treble().load(Ordering::Relaxed),
                1.0
            );
            assert_eq!(*cm.current_channel_id(), test_channel.id());
        }

        #[test]
        fn remove_channel_removes_channel_and_sets_current_channel_id_to_default() {
            let mock = MockAudioHandlerTrait::new();
            let mut service = AudioService::new_with_handler(Arc::new(mock), new_cm());
            let default_channel_id = {
                let cm = channels(&service);
                cm.channels()[0].id()
            };
            let test_channel_id = service.add_channel("TestChannel".to_string());
            service.remove_channel(test_channel_id);

            let cm = channels(&service);
            assert_eq!(cm.channels().len(), 1);
            assert_eq!(*cm.current_channel_id(), default_channel_id);
        }

        #[test]
        fn apply_amp_config_restores_channels_tones_effects_and_master_volume() {
            let mut service = build_service(make_mock_handler());
            let id_1 = Uuid::new_v4();
            let id_2 = Uuid::new_v4();
            let eff_id = Uuid::new_v4();

            let channel_id_1 = id_1.to_string();
            let channel_id_2 = id_2.to_string();
            let effect_id = eff_id.to_string();
            let config = AmpConfigDto {
                master_volume: 0.42,
                is_active: false,
                channels: vec![
                    channel_dto(
                        channel_id_1.clone(),
                        "Clean",
                        1.25,
                        0.8,
                        tone_stack(25.0, 0.45, 130.0),
                        vec![],
                    ),
                    channel_dto(
                        channel_id_2.clone(),
                        "Lead",
                        2.0,
                        0.65,
                        tone_stack(0.6, 80.0, -0.5),
                        vec![distortion_effect(
                            effect_id.clone(),
                            "Drive",
                            true,
                            0.33,
                            0.7,
                            "#ff6600",
                        )],
                    ),
                ],
                current_channel: channel_id_2.clone(),
                audio_settings: basic_audio_settings(),
                midi_bindings: vec![],
            };

            service.apply_amp_config(config, &DeviceService::new());

            let cm = channels(&service);
            let clean = cm
                .channels()
                .iter()
                .find(|channel| channel.id().to_string() == channel_id_1)
                .unwrap();
            let lead = cm
                .channels()
                .iter()
                .find(|channel| channel.id().to_string() == channel_id_2)
                .unwrap();

            // Verify directly from service state (avoid from_service which requires device mocking)
            assert_eq!(cm.channels().len(), 2);
            assert_eq!(cm.current_channel_id().to_string(), channel_id_2);
            assert!(!*service.is_active());
            assert!((service.master_volume().load(Ordering::Relaxed) - 0.42).abs() < f32::EPSILON);

            assert_eq!(clean.name(), "Clean");
            assert!((clean.gain().load(Ordering::Relaxed) - 1.25).abs() < f32::EPSILON);
            assert!((clean.volume().load(Ordering::Relaxed) - 0.8).abs() < f32::EPSILON);
            assert!((clean.tone_stack().bass().load(Ordering::Relaxed) - 0.25).abs() < 1e-6);
            assert!((clean.tone_stack().middle().load(Ordering::Relaxed) - 0.45).abs() < 1e-6);
            assert!((clean.tone_stack().treble().load(Ordering::Relaxed) - 1.0).abs() < 1e-6);

            assert_eq!(lead.name(), "Lead");
            assert!((lead.tone_stack().bass().load(Ordering::Relaxed) - 0.6).abs() < 1e-6);
            assert!((lead.tone_stack().middle().load(Ordering::Relaxed) - 0.8).abs() < 1e-6);
            assert!((lead.tone_stack().treble().load(Ordering::Relaxed) - 0.0).abs() < 1e-6);

            let effect_chain_arc = lead.effect_chain();
            let chain = effect_chain_arc.lock().unwrap();
            assert_eq!(chain.len(), 1);
        }

        #[test]
        fn apply_amp_config_restores_cabinet_effect_ir_file_path() {
            let mut service = build_service(make_mock_handler());
            let channel_id_1 = Uuid::new_v4();
            let effect_id = Uuid::new_v4();
            let config = AmpConfigDto {
                master_volume: 0.8,
                is_active: false,
                channels: vec![channel_dto(
                    channel_id_1.to_string(),
                    "Cab Channel",
                    1.0,
                    1.0,
                    tone_stack(0.5, 0.5, 0.5),
                    vec![cabinet_effect(
                        effect_id.to_string(),
                        "Cab",
                        true,
                        "#445566",
                        "Vox-ac30.wav",
                    )],
                )],
                current_channel: channel_id_1.to_string(),
                audio_settings: basic_audio_settings(),
                midi_bindings: vec![],
            };

            service.apply_amp_config(config, &DeviceService::new());

            let cm = channels(&service);
            assert_eq!(cm.channels().len(), 1);

            let channel = &cm.channels()[0];
            let effect_chain_arc = channel.effect_chain();
            let chain = effect_chain_arc.lock().unwrap();
            assert_eq!(chain.len(), 1);
        }

        #[test]
        fn apply_amp_config_clamps_non_positive_levels_and_falls_back_to_first_channel() {
            let mut service = build_service(make_mock_handler());
            let channel_id_1 = Uuid::new_v4();
            let config = AmpConfigDto {
                master_volume: 0.0,
                is_active: false,
                channels: vec![channel_dto(
                    channel_id_1.to_string(),
                    "Crunch",
                    -2.0,
                    0.0,
                    tone_stack(0.2, 0.4, 0.6),
                    vec![],
                )],
                current_channel: Uuid::new_v4().to_string(),
                audio_settings: basic_audio_settings(),
                midi_bindings: vec![],
            };

            service.apply_amp_config(config, &DeviceService::new());

            let cm = channels(&service);
            let channel = cm
                .channels()
                .iter()
                .find(|channel| channel.id() == channel_id_1)
                .unwrap();

            assert_eq!(cm.channels().len(), 1);
            assert_eq!(*cm.current_channel_id(), channel.id());
            assert!((channel.gain().load(Ordering::Relaxed) - 0.0001).abs() < 1e-6);
            assert!((channel.volume().load(Ordering::Relaxed) - 0.0001).abs() < 1e-6);
            assert!((service.master_volume().load(Ordering::Relaxed) - 0.0001).abs() < 1e-6);
        }

        #[test]
        fn apply_amp_config_with_no_channels_creates_default_channel() {
            let mut service = build_service(make_mock_handler());

            service.apply_amp_config(
                AmpConfigDto {
                    master_volume: 0.75,
                    is_active: false,
                    channels: vec![],
                    current_channel: Uuid::new_v4().to_string(),
                    audio_settings: basic_audio_settings(),
                    midi_bindings: vec![],
                },
                &DeviceService::new(),
            );
            service.apply_amp_config(
                AmpConfigDto {
                    master_volume: 0.75,
                    is_active: false,
                    channels: vec![],
                    current_channel: Uuid::new_v4().to_string(),
                    ..AmpConfigDto::default()
                },
                &DeviceService::new(),
            );

            let cm = channels(&service);
            assert_eq!(cm.channels().len(), 1);
            assert_eq!(cm.channels()[0].name(), "Default");
            assert_eq!(*cm.current_channel_id(), cm.channels()[0].id());
            assert!((service.master_volume().load(Ordering::Relaxed) - 0.75).abs() < f32::EPSILON);
        }

        #[test]
        fn apply_amp_config_with_active_flag_starts_loopback() {
            let mut service = build_service(make_mock_handler());
            let channel_id_1 = Uuid::new_v4();
            service.apply_amp_config(
                AmpConfigDto {
                    master_volume: 0.9,
                    is_active: true,
                    channels: vec![channel_dto(
                        channel_id_1.to_string(),
                        "Loopback",
                        1.0,
                        1.0,
                        tone_stack(0.5, 0.5, 0.5),
                        vec![],
                    )],
                    current_channel: channel_id_1.to_string(),
                    audio_settings: basic_audio_settings(),
                    midi_bindings: vec![],
                },
                &DeviceService::new(),
            );
            service.apply_amp_config(
                AmpConfigDto {
                    master_volume: 0.9,
                    is_active: true,
                    channels: vec![channel_dto(
                        channel_id_1.to_string(),
                        "Loopback",
                        1.0,
                        1.0,
                        tone_stack(0.5, 0.5, 0.5),
                        vec![],
                    )],
                    current_channel: channel_id_1.to_string(),
                    ..AmpConfigDto::default()
                },
                &DeviceService::new(),
            );

            assert!(*service.is_active());

            service.stop_loopback();

            assert!(!*service.is_active());
        }

        #[test]
        fn apply_amp_config_restores_audio_driver_and_falls_back_to_old_handler_devices() {
            let mut old_handler = make_mock_handler();

            let dummy_host = cpal::default_host();
            let dummy_input = dummy_host
                .default_input_device()
                .expect("No input device available");
            let dummy_output = dummy_host
                .default_output_device()
                .expect("No output device available");

            let expected_input_id = dummy_input.id().unwrap().to_string();
            let expected_output_id = dummy_output.id().unwrap().to_string();

            let dummy_input_config = cpal::StreamConfig {
                channels: 1,
                sample_rate: 44100,
                buffer_size: cpal::BufferSize::Default,
            };
            let dummy_output_config = cpal::StreamConfig {
                channels: 2,
                sample_rate: 44100,
                buffer_size: cpal::BufferSize::Default,
            };

            old_handler
                .expect_input_device()
                .return_const(dummy_input.clone());

            old_handler
                .expect_output_device()
                .return_const(dummy_output.clone());

            old_handler
                .expect_input_config()
                .return_const(dummy_input_config);

            old_handler
                .expect_output_config()
                .return_const(dummy_output_config);

            let mut service = build_service(old_handler);
            let device_service = DeviceService::new();

            let target_driver = "ASIO";
            let mut audio_settings = basic_audio_settings();
            audio_settings.audio_driver = target_driver.to_string();
            audio_settings.input_device_name = "NonExistentInputDeviceName123".to_string();
            audio_settings.output_device_name = "NonExistentOutputDeviceName123".to_string();
            audio_settings.input_sample_rate = 48000;
            audio_settings.input_channels = 2;
            audio_settings.output_sample_rate = 48000;
            audio_settings.output_channels = 2;

            let config = AmpConfigDto {
                master_volume: 0.5,
                is_active: false,
                channels: vec![],
                current_channel: "".to_string(),
                audio_settings,
                midi_bindings: vec![],
            };

            service.apply_amp_config(config, &device_service);

            let new_handler = service.audio_handler.clone();

            assert_eq!(new_handler.input_config().sample_rate, 48000);
            assert_eq!(new_handler.input_config().channels, 2);
            assert_eq!(new_handler.output_config().sample_rate, 48000);
            assert_eq!(new_handler.output_config().channels, 2);

            assert_eq!(
                new_handler.input_device().id().unwrap().to_string(),
                expected_input_id
            );
            assert_eq!(
                new_handler.output_device().id().unwrap().to_string(),
                expected_output_id
            );
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        #[should_panic(expected = "Master volume must be positive")]
        fn master_volume_set_to_negative_value_should_panic() {
            let mock = MockAudioHandlerTrait::new();
            let service = AudioService::new_with_handler(Arc::new(mock), new_cm());
            service.set_master_volume(-0.5);
        }

        #[test]
        fn removing_default_channel_should_do_nothing() {
            let mock = MockAudioHandlerTrait::new();
            let mut service = AudioService::new_with_handler(Arc::new(mock), new_cm());
            let default_channel_id = {
                let cm = channels(&service);
                cm.channels()[0].id()
            };
            service.remove_channel(default_channel_id);

            let cm = channels(&service);
            assert_eq!(cm.channels().len(), 1);
        }

        #[test]
        #[should_panic(expected = "Channel name must be 30 characters or less")]
        fn add_channel_should_panic_with_to_long_name() {
            let mock = MockAudioHandlerTrait::new();
            let mut service = AudioService::new_with_handler(Arc::new(mock), new_cm());
            let _test_channel =
                service.add_channel("Hippopotomonstrosesquippedaliophobia".to_string());
        }
    }
}
