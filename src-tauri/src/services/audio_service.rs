use crate::domain::audio_processor::AudioProcessor;
use crate::domain::channel::Channel;
use crate::domain::dto::amp_config_dto::AmpConfigDto;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use crate::services::effects::cabinet::cabinet::Cabinet;
use crate::services::effects::distortion::hc_distortion::HCDistortion;
use crate::services::processors::gain::gain_processor::GainProcessor;
use crate::services::processors::resampler::resampler::ResamplePolicy;
use crate::services::processors::tone_stack::tone_stack_processor::ToneStackProcessor;
use atomic_float::AtomicF32;
use cpal::{BufferSize, Device, StreamConfig};
use derive_getters::Getters;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{error, info};

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
/// [`AudioHandlerTrait`]: crate::infrastructure::audio_handler::AudioHandlerTrait
/// [`ResamplePolicy`]: crate::services::processors::resampler::resampler::ResamplePolicy
/// [`stop_loopback`]: AudioService::stop_loopback
#[derive(Getters)]
pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
    loopback_thread: Option<JoinHandle<()>>,
    is_active: bool,
    channels: Vec<Channel>,
    current_channel_id: u32,
    master_volume: Arc<AtomicF32>,
    next_channel_id: u32,
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
    ///
    /// # Arguments
    ///
    /// * `input_device` - The CPAL device to capture audio from.
    /// * `output_device` - The CPAL device to send processed audio to.
    /// * `input_config` - The [`StreamConfig`] used for the input stream.
    /// * `output_config` - The [`StreamConfig`] used for the output stream.
    pub fn new(
        input_device: Device,
        output_device: Device,
        input_config: StreamConfig,
        output_config: StreamConfig,
    ) -> Self {
        let handler = AudioHandler::new(input_device, output_device, input_config, output_config);
        Self::new_with_handler(Arc::new(handler))
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
    pub fn new_with_handler(handler: Arc<dyn AudioHandlerTrait>) -> Self {
        Self {
            audio_handler: handler,
            loopback_thread: None,
            is_active: false,
            channels: vec![Channel::new(0, "Default".to_string(), None, None)],
            master_volume: Arc::new(AtomicF32::new(1.0)),
            current_channel_id: 0,
            next_channel_id: 1,
        }
    }

    /// Starts the audio loopback on a dedicated background thread.
    ///
    /// On startup the service:
    /// 1. Reads the input and output sample rates from the active [`AudioHandlerTrait`].
    /// 2. Selects a [`ResamplePolicy`] based on those rates (logged at `info` level).
    /// 3. Builds a pair of lock-free ring buffers sized to the larger of the two rates.
    /// 4. Asks the handler to open the input and output CPAL streams.
    /// 5. Spawns a worker thread that:
    ///    - Pops samples from the input buffer.
    ///    - Routes them through the [`ResamplePolicy`] which interleaves `run_dsp` at
    ///      the correct point (before or after resampling).
    ///    - Pushes results into the output buffer.
    ///    - On shutdown, flushes any remaining resampler tail before exiting.
    ///
    /// If the loopback is already active this method is a no-op.
    ///
    /// [`AudioHandlerTrait`]: crate::infrastructure::audio_handler::AudioHandlerTrait
    /// [`ResamplePolicy`]: crate::services::processors::resampler::resampler::ResamplePolicy
    pub fn start_loopback(&mut self) {
        if self.is_active {
            return;
        }

        info!("Starting audio loopback");
        self.is_active = true;

        let handler = self.audio_handler.clone();
        let channel_id = self.current_channel_id;
        let master_volume_arc = self.master_volume.clone();

        let (gain_arc, volume_arc, tone_stack_arc, effect_chain_arc) = {
            let channel = self
                .channels
                .iter_mut()
                .find(|c| c.id() == channel_id)
                .unwrap();

            (
                channel.gain(),
                channel.volume(),
                channel.tone_stack(),
                channel.effect_chain(),
            )
        };

        let thread = thread::spawn(move || {
            // How many input samples to batch before the resampler produces output.
            // Larger = better quality, more latency. Smaller = lower latency, cheaper.
            const RESAMPLER_CHUNK_SIZE: usize = 256;

            let ringbuffer_size = handler
                .input_sample_rate()
                .max(handler.output_sample_rate()) as usize;

            // ── Resampling decision ──────────────────────────────────────────────
            // `ResamplePolicy::from_rates` compares input and output sample rates
            // and picks one of three strategies (logged at startup):
            //
            //   input == output  →  Bypass   – no resampler created at all
            //   input  > output  →  PreDsp   – downsample BEFORE the DSP chain
            //                                  (DSP runs at the lower output rate → cheaper)
            //   input  < output  →  PostDsp  – upsample AFTER the DSP chain
            //                                  (DSP runs at the lower input rate → cheaper)
            //
            // The chosen policy is the only place a `RubatoResampler` is created.
            let mut policy = ResamplePolicy::from_rates(
                handler.input_sample_rate(),
                handler.output_sample_rate(),
                RESAMPLER_CHUNK_SIZE,
            );

            let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);
            let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);

            let input_stream = handler.build_input_stream(i_producer);
            let output_stream = handler.build_output_stream(o_consumer);

            let shutdown = Arc::new(AtomicBool::new(false));
            let worker_shutdown = shutdown.clone();

            let worker = thread::spawn(move || {
                let mut gain = GainProcessor::new(gain_arc);
                let mut volume = GainProcessor::new(volume_arc);
                let mut master_volume = GainProcessor::new(master_volume_arc);
                let mut tone_stack = ToneStackProcessor::new(tone_stack_arc);

                let mut run_dsp = |sample: f32| -> f32 {
                    let sample = gain.process(sample);
                    let mut sample = tone_stack.process(sample);
                    if let Ok(mut chain) = effect_chain_arc.lock() {
                        for effect in chain.iter_mut() {
                            sample = effect.process_if_active(sample);
                        }
                    }
                    let sample = volume.process(sample);
                    master_volume.process(sample)
                };

                loop {
                    if worker_shutdown.load(Ordering::SeqCst) {
                        break;
                    }

                    if let Some(sample) = i_consumer.try_pop() {
                        // `policy.process` applies the resampler at the right moment:
                        //   PreDsp  → resamples first, then calls `dsp.process` on each result
                        //   PostDsp → calls `dsp.process` first, then resamples the output
                        //   Bypass  → calls `dsp.process` directly, returns a single sample
                        for processed_sample in policy
                            .process(sample, &mut |resampled_sample| run_dsp(resampled_sample))
                        {
                            let _ = o_producer.try_push(processed_sample);
                        }
                    } else {
                        thread::yield_now();
                    }
                }

                // Drain any samples still sitting in the resampler's input buffer
                // when the loopback is stopped so we don't lose the tail.
                for processed_sample in
                    policy.flush(&mut |resampled_sample| run_dsp(resampled_sample))
                {
                    let _ = o_producer.try_push(processed_sample);
                }
            });

            input_stream.play();
            output_stream.play();

            thread::park();

            shutdown.store(true, Ordering::SeqCst);
            let _ = worker.join();
        });

        self.loopback_thread = Some(thread);
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

        info!("Stopping audio loopback");

        if let Some(handle) = self.loopback_thread.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }


        self.is_active = false;
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
        let was_active = self.is_active;
        if was_active {
            self.stop_loopback();
        }

        self.audio_handler = new_handler;

        if was_active {
            self.start_loopback();
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
    pub fn add_channel(&mut self, channel_name: String) -> u32 {
        if channel_name.len() <= 30 {
            let id = self.next_channel_id;
            self.next_channel_id += 1;

            let new_channel = Channel::new(id, channel_name.into(), None, None);

            self.channels.push(new_channel);
            self.set_current_channel_id(id);
            id
        } else {
            error!("Channel name must be 30 characters or less");
            panic!("Channel name must be 30 characters or less");
        }
    }

    /// Returns a mutable reference to the channel list, allowing channels to be modified or reordered.
    pub fn channels_mut(&mut self) -> &mut Vec<Channel> {
        &mut self.channels
    }

    /// Removes the channel with the given id from the channel list and sets `current_channel_id` to 0 (default channel).
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The id of the channel to remove. Cannot be 0 (default channel).
    ///
    /// [`set_current_channel_id`]: AudioService::set_current_channel_id
    pub fn remove_channel(&mut self, channel_id: u32) {
        if channel_id != 0 {
            self.channels.retain(|c| c.id() != channel_id);
            self.set_current_channel_id(0);
        } else {
            error!("Cannot remove default channel");
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
    pub fn set_current_channel_id(&mut self, new_current_channel_id: u32) {
        let was_on = self.is_active;
        self.stop_loopback();
        self.current_channel_id = new_current_channel_id;
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
        self.set_audio_handler(std::sync::Arc::new(new_handler));
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
    ///
    /// Note that the current JSON persistence implementation always loads with
    /// `is_active = false`, so persisted sessions restart with loopback turned
    /// off even though this method is capable of applying either state.
    pub fn apply_amp_config(&mut self, config: AmpConfigDto) {
        let mut restored_channels = Vec::new();
        let dsp_sample_rate = self.dsp_chain_sample_rate();

        // Backward compatibility: older snapshots stored tone values as 0..100.
        // New normalized format is 0.0..1.0 end-to-end.
        let normalize_tone_value = |value: f32| -> f32 {
            if value > 1.0 {
                (value / 100.0).clamp(0.0, 1.0)
            } else {
                value.clamp(0.0, 1.0)
            }
        };

        for channel_dto in config.channels {
            let mut channel = Channel::new(
                channel_dto.id,
                channel_dto.name,
                Some(channel_dto.gain.max(0.0001)),
                Some(channel_dto.volume.max(0.0001)),
            );

            channel.set_bass(normalize_tone_value(channel_dto.tone_stack.bass));
            channel.set_middle(normalize_tone_value(channel_dto.tone_stack.middle));
            channel.set_treble(normalize_tone_value(channel_dto.tone_stack.treble));

            let restored_effects = channel_dto
                .effect_chain
                .into_iter()
                .map(|effect| match effect {
                    EffectDto::HCDistortion(distortion) => Box::new(HCDistortion::new(
                        distortion.id,
                        distortion.name,
                        distortion.is_active,
                        distortion.threshold,
                        distortion.level,
                        distortion.color,
                    ))
                        as Box<dyn crate::domain::effect::Effect>,
                    EffectDto::Cabinet(cabinet) => Box::new(
                        Cabinet::new(
                            cabinet.id,
                            cabinet.name,
                            cabinet.is_active,
                            cabinet.color,
                            cabinet.ir_file_path,
                            dsp_sample_rate,
                        ),
                    ) as Box<dyn crate::domain::effect::Effect>,
                })
                .collect::<Vec<_>>();

            if !restored_effects.is_empty() {
                channel.replace_effect_chain(restored_effects);
            }
            restored_channels.push(channel);
        }

        if restored_channels.is_empty() {
            restored_channels.push(Channel::new(0, "Default".to_string(), None, None));
        }

        let current_channel = if restored_channels
            .iter()
            .any(|c| c.id() == config.current_channel)
        {
            config.current_channel
        } else {
            restored_channels[0].id()
        };

        self.channels = restored_channels;
        self.current_channel_id = current_channel;
        self.next_channel_id = self.channels.iter().map(|c| c.id()).max().unwrap_or(0) + 1;
        self.master_volume
            .store(config.master_volume.max(0.0001), Ordering::Relaxed);

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
    use crate::domain::dto::amp_config_dto::AmpConfigDto;
    use crate::domain::dto::channel_dto::ChannelDto;
    use crate::domain::dto::effect::cabinet_dto::CabinetDto;
    use crate::domain::dto::effect::effect_dto::EffectDto;
    use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
    use crate::domain::dto::tone_stack_dto::ToneStackDto;
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use crate::tests::mock::make_mock_handler;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    fn build_service(handler: MockAudioHandlerTrait) -> AudioService {
        AudioService::new_with_handler(Arc::new(handler))
    }

    fn tone_stack(bass: f32, middle: f32, treble: f32) -> ToneStackDto {
        ToneStackDto {
            bass,
            middle,
            treble,
        }
    }

    fn distortion_effect(
        id: u32,
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

    fn cabinet_effect(id: u32, name: &str, is_active: bool, color: &str, ir_file_path: &str) -> EffectDto {
        EffectDto::Cabinet(CabinetDto {
            id,
            name: name.to_string(),
            is_active,
            color: color.to_string(),
            ir_file_path: ir_file_path.to_string(),
        })
    }

    fn channel_dto(
        id: u32,
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

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn master_volume_set_to_positive_value_should_succeed() {
            let mock = MockAudioHandlerTrait::new();
            let service = AudioService::new_with_handler(Arc::new(mock));
            service.set_master_volume(0.5);
            assert_eq!(service.master_volume().load(Ordering::Relaxed), 0.5);
        }

        #[test]
        fn add_channel_should_add_a_channel_with_correct_values_and_sets_current_channel_id_to_new_id(
        ) {
            let mock = MockAudioHandlerTrait::new();
            let mut service = AudioService::new_with_handler(Arc::new(mock));
            let test_channel_id = service.add_channel("TestChannel".to_string());
            let test_channel = service
                .channels
                .iter()
                .find(|c| c.id() == test_channel_id)
                .unwrap();

            assert_eq!(service.channels.len(), 2);
            assert_eq!(test_channel.name(), "TestChannel");
            assert_eq!(test_channel.id(), 1);
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
            assert_eq!(*service.current_channel_id(), test_channel.id());
        }

        #[test]
        fn remove_channel_removes_channel_and_sets_current_channel_id_to_0() {
            let mock = MockAudioHandlerTrait::new();
            let mut service = AudioService::new_with_handler(Arc::new(mock));
            let test_channel_id = service.add_channel("TestChannel".to_string());
            service.remove_channel(test_channel_id);

            assert_eq!(service.channels.len(), 1);
            assert_eq!(*service.current_channel_id(), 0);
        }

        #[test]
        fn apply_amp_config_restores_channels_tones_effects_and_master_volume() {
            let mut service = build_service(make_mock_handler());
            let config = AmpConfigDto {
                master_volume: 0.42,
                is_active: false,
                channels: vec![
                    channel_dto(4, "Clean", 1.25, 0.8, tone_stack(25.0, 0.45, 130.0), vec![]),
                    channel_dto(
                        7,
                        "Lead",
                        2.0,
                        0.65,
                        tone_stack(0.6, 80.0, -0.5),
                        vec![distortion_effect(11, "Drive", true, 0.33, 0.7, "#ff6600")],
                    ),
                ],
                current_channel: 7,
            };

            service.apply_amp_config(config);

            let snapshot = AmpConfigDto::from_service(&service);
            let clean = snapshot
                .channels
                .iter()
                .find(|channel| channel.id == 4)
                .unwrap();
            let lead = snapshot
                .channels
                .iter()
                .find(|channel| channel.id == 7)
                .unwrap();

            assert_eq!(snapshot.channels.len(), 2);
            assert_eq!(snapshot.current_channel, 7);
            assert!(!snapshot.is_active);
            assert_eq!(service.next_channel_id, 8);
            assert!((snapshot.master_volume - 0.42).abs() < f32::EPSILON);

            assert_eq!(clean.name, "Clean");
            assert!((clean.gain - 1.25).abs() < f32::EPSILON);
            assert!((clean.volume - 0.8).abs() < f32::EPSILON);
            assert!((clean.tone_stack.bass - 0.25).abs() < 1e-6);
            assert!((clean.tone_stack.middle - 0.45).abs() < 1e-6);
            assert!((clean.tone_stack.treble - 1.0).abs() < 1e-6);

            assert_eq!(lead.name, "Lead");
            assert!((lead.tone_stack.bass - 0.6).abs() < 1e-6);
            assert!((lead.tone_stack.middle - 0.8).abs() < 1e-6);
            assert!((lead.tone_stack.treble - 0.0).abs() < 1e-6);
            assert_eq!(lead.effect_chain.len(), 1);
            // Compare effect fields individually so floating-point round-trips through
            // the internal gain mapping (level → 1.0+level → level-1.0) don't fail.
            if let EffectDto::HCDistortion(dto) = &lead.effect_chain[0] {
                assert_eq!(dto.id, 11);
                assert_eq!(dto.name, "Drive");
                assert!(dto.is_active);
                assert_eq!(dto.color, "#ff6600");
                assert!((dto.threshold - 0.33).abs() < 1e-6);
                assert!((dto.level - 0.7).abs() < 1e-5);
            } else {
                panic!("Expected HCDistortion effect");
            }
        }

        #[test]
        fn apply_amp_config_restores_cabinet_effect_ir_file_path() {
            let mut service = build_service(make_mock_handler());
            let config = AmpConfigDto {
                master_volume: 0.8,
                is_active: false,
                channels: vec![channel_dto(
                    2,
                    "Cab Channel",
                    1.0,
                    1.0,
                    tone_stack(0.5, 0.5, 0.5),
                    vec![cabinet_effect(9, "Cab", true, "#445566", "Vox-ac30.wav")],
                )],
                current_channel: 2,
            };

            service.apply_amp_config(config);

            let snapshot = AmpConfigDto::from_service(&service);
            assert_eq!(snapshot.channels.len(), 1);
            assert_eq!(snapshot.channels[0].effect_chain.len(), 1);

            if let EffectDto::Cabinet(dto) = &snapshot.channels[0].effect_chain[0] {
                assert_eq!(dto.id, 9);
                assert_eq!(dto.name, "Cab");
                assert!(dto.is_active);
                assert_eq!(dto.color, "#445566");
                assert_eq!(dto.ir_file_path, "Vox-ac30.wav");
            } else {
                panic!("Expected Cabinet effect");
            }
        }

        #[test]
        fn apply_amp_config_clamps_non_positive_levels_and_falls_back_to_first_channel() {
            let mut service = build_service(make_mock_handler());
            let config = AmpConfigDto {
                master_volume: 0.0,
                is_active: false,
                channels: vec![channel_dto(
                    4,
                    "Crunch",
                    -2.0,
                    0.0,
                    tone_stack(0.2, 0.4, 0.6),
                    vec![],
                )],
                current_channel: 999,
            };

            service.apply_amp_config(config);

            let channel = service
                .channels
                .iter()
                .find(|channel| channel.id() == 4)
                .unwrap();

            assert_eq!(service.channels.len(), 1);
            assert_eq!(*service.current_channel_id(), 4);
            assert_eq!(service.next_channel_id, 5);
            assert!((channel.gain().load(Ordering::Relaxed) - 0.0001).abs() < 1e-6);
            assert!((channel.volume().load(Ordering::Relaxed) - 0.0001).abs() < 1e-6);
            assert!((service.master_volume().load(Ordering::Relaxed) - 0.0001).abs() < 1e-6);
        }

        #[test]
        fn apply_amp_config_with_no_channels_creates_default_channel() {
            let mut service = build_service(make_mock_handler());

            service.apply_amp_config(AmpConfigDto {
                master_volume: 0.75,
                is_active: false,
                channels: vec![],
                current_channel: 321,
            });

            assert_eq!(service.channels.len(), 1);
            assert_eq!(service.channels[0].id(), 0);
            assert_eq!(service.channels[0].name(), "Default");
            assert_eq!(*service.current_channel_id(), 0);
            assert_eq!(service.next_channel_id, 1);
            assert!((service.master_volume().load(Ordering::Relaxed) - 0.75).abs() < f32::EPSILON);
        }

        #[test]
        fn apply_amp_config_with_active_flag_starts_loopback() {
            let mut service = build_service(make_mock_handler());

            service.apply_amp_config(AmpConfigDto {
                master_volume: 0.9,
                is_active: true,
                channels: vec![channel_dto(
                    2,
                    "Loopback",
                    1.0,
                    1.0,
                    tone_stack(0.5, 0.5, 0.5),
                    vec![],
                )],
                current_channel: 2,
            });

            assert!(*service.is_active());

            service.stop_loopback();

            assert!(!*service.is_active());
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        #[should_panic(expected = "Master volume must be positive")]
        fn master_volume_set_to_negative_value_should_panic() {
            let mock = MockAudioHandlerTrait::new();
            let service = AudioService::new_with_handler(Arc::new(mock));
            service.set_master_volume(-0.5);
        }

        #[test]
        fn removing_default_channel_should_do_nothing() {
            let mock = MockAudioHandlerTrait::new();
            let mut service = AudioService::new_with_handler(Arc::new(mock));
            service.remove_channel(0);

            assert_eq!(service.channels.len(), 1);
        }

        #[test]
        #[should_panic(expected = "Channel name must be 30 characters or less")]
        fn add_channel_should_panic_with_to_long_name() {
            let mock = MockAudioHandlerTrait::new();
            let mut service = AudioService::new_with_handler(Arc::new(mock));
            let test_channel =
                service.add_channel("Hippopotomonstrosesquippedaliophobia".to_string());
        }
    }
}
