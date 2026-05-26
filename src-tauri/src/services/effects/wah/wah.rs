use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::dto::effect::wah_dto::WahDto;
use crate::domain::effect::Effect;
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

/// # Wah-Wah Filter Effect
///
/// `Wah` implements a classic auto-wah / expression pedal emulation using a resonant
/// State-Variable Filter (SVF). It features a runtime variable center frequency modulated by
/// a `pedal_position` parameter, mimicking a physical potentiometer treadle.
///
/// ## Signal Chain & Architecture
///
/// The processing leverages a state-variable filter topological structure (Chamberlin form)
/// configured to produce a mix heavily emphasizing the resonant band-pass region:
///
/// 1. **Dynamic Center Frequency Calculation**
///    - The `pedal_position` `[0.0, 1.0]` is mapped exponentially to an audible frequency range.
///    - This logarithmic swept response closely matches human perception of pitch changes and
///      the behavior of vintage analog hardware inductors.
///
/// 2. **State Variable Filtering (SVF)**
///    - The input sample is resolved concurrently into high-pass, band-pass, and low-pass states.
///    - Damping is applied inversely to a fixed quality factor ($Q = 3.0$), creating a sharp,
///      expressive resonant peak.
///
/// 3. **Output Gain / Make-up Gain**
///    - The isolated band-pass output is extracted and scaled by a fixed scalar factor of `2.5`.
///    - This compensates for energy attenuation in high-pass/low-pass splits and helps the classic
///      "quack" and vocal characteristics pierce cleanly through an effects mix.
///
/// ## Parameter Ranges
///
/// | Parameter        | Range        | UI / MIDI Display | Effect |
/// |------------------|--------------|-------------------|--------|
/// | `pedal_position` | `[0.0, 1.0]`  | Pedal 0–100%      | `0.0` (Heel/Bass, ~350Hz) to `1.0` (Toe/Treble, ~2200Hz) |
/// | `sample_rate`    | `> 0.0`       | Internal Metric   | Drives internal discrete-time angular frequency coefficient calculations |
///
/// ## Thread-Safe Atomic Updates
///
/// All mutable state variables exposed to control changes utilize lock-free atomics:
/// - `is_active`: [`Arc<AtomicBool>`] — Bypasses or engages the DSP state.
/// - `pedal_position`: [`Arc<AtomicF32>`] — Direct control sweep shared with [`f32_params`](Self::f32_params).
///
/// Internal DSP state registers (`s1`, `s2`) remain plain local `f32` primitives since they
/// are strictly mutated, retained, and owned by the isolated real-time audio thread executing [`process`](Self::process).
///
/// ## In simple terms
/// The effect splits the signal, droping the deep basses and the piercing high tones. After which the middle becomes isolated and focused on the chosen frequency by the pedal (configured due to our pedal_position_variable)
/// After which the signal is bosted to get that vocal "wah" character.
pub struct Wah {
    id: Uuid,
    name: String,
    color: String,
    is_active: Arc<AtomicBool>,
    pedal_position: Arc<AtomicF32>,
    sample_rate: f32,
    s1: f32,
    s2: f32,
}

impl Wah {
    /// Creates a new `Wah` effect instance.
    ///
    /// # Parameters
    ///
    /// * `id` — Unique identifier for this effect instance
    /// * `name` — Human-readable name (e.g., "Crybaby Emulation")
    /// * `color` — Used as the visual color of the effect pedal by the UI.
    /// * `is_active` — Initial bypass/active state of the filter
    /// * `initial_position` — Starting position of the treadle `[0.0, 1.0]`
    /// * `sample_rate` — Digital audio processing rate (Hz) for coefficient scaling
    pub fn new(
        id: Uuid,
        name: String,
        color: String,
        is_active: bool,
        initial_position: f32,
        sample_rate: f32,
    ) -> Self {
        Self {
            id,
            name,
            color,
            is_active: Arc::new(AtomicBool::new(is_active)),
            pedal_position: Arc::new(AtomicF32::new(initial_position.clamp(0.0, 1.0))),
            sample_rate,
            s1: 0.0,
            s2: 0.0,
        }
    }

    /// Sets the pedal treadle position from software control loops or UI commands.
    ///
    /// Changes take effect instantly on the next processed sample frame.
    ///
    /// # Parameters
    ///
    /// * `pos` — Normalised position `[0.0, 1.0]`. Values outside are clamped.
    pub fn set_position(&self, pos: f32) {
        self.pedal_position
            .store(pos.clamp(0.0, 1.0), Ordering::Relaxed);
    }
    /// Returns the current normalised position of the pedal treadle.
    ///
    /// # Returns
    ///
    /// A value between `0.0` (Heel down, bass response) and `1.0` (Toe down, treble emphasis).
    pub fn position(&self) -> f32 {
        self.pedal_position.load(Ordering::Relaxed)
    }
}

impl AudioProcessor for Wah {
    /// Processes a single audio sample through the State-Variable resonant filter.
    ///
    /// # Algorithm
    ///
    /// 1. **Load Position** — Fetches the atomic position value without lock overhead.
    /// 2. **Map Frequency** — Logarithmically interpolates between $350\text{ Hz}$ and $2200\text{ Hz}$ via:
    ///    $$f_c = 350 \times \left(\frac{2200}{350}\right)^{\text{pos}}$$
    /// 3. **Calculate Coefficients** — Computes the discrete angular scalar $f = 2 \cdot \sin(\pi \cdot f_c / f_s)$.
    /// 4. **Solve Topology** — Sequentially advances state equations for high, band, and low components.
    /// 5. **Scale Resonance** — Amplifies the band-pass state by $2.5$ to hit output targets.
    ///
    /// # Parameters
    ///
    /// * `sample` — Normalised floating-point input sample.
    ///
    /// # Returns
    ///
    /// Band-pass filtered and scaled audio frame.
    fn process(&mut self, sample: f32) -> f32 {
        let pos = self.pedal_position.load(Ordering::Relaxed);
        let min_freq: f32 = 350.0;
        let max_freq: f32 = 2200.0;
        let center_freq = min_freq * (max_freq / min_freq).powf(pos);
        let q: f32 = 3.0;
        let f = 2.0 * (std::f32::consts::PI * center_freq / self.sample_rate).sin();
        let damping = 1.0 / q;
        let high_pass = sample - self.s1 * damping - self.s2;
        let band_pass = f * high_pass + self.s1;
        let low_pass = f * band_pass + self.s2;
        self.s1 = band_pass;
        self.s2 = low_pass;
        let wah_signal = band_pass * 2.5;

        wah_signal
    }
}

impl Effect for Wah {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
    /// Provides a hex chassis color representation for visualizers.
    ///
    /// # Returns
    ///
    /// A copper/orange hex string (`"#e67e22"`) reminiscent of classic vintage inductor pedals.
    fn get_color(&self) -> String {
        self.color.clone()
    }

    fn active_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_active)
    }
    /// Exposes internal atomic parameters to systemic command dispatchers.
    ///
    /// # Returns
    ///
    /// A `HashMap` containing:
    /// * `"pedal_position"` — Points directly to the shared control thread atomic block.
    fn f32_params(&self) -> HashMap<&'static str, Arc<AtomicF32>> {
        let mut map = HashMap::new();
        // This key is what your MIDI mapping/UI thread will look for to move the foot pedal!
        map.insert("pedal_position", Arc::clone(&self.pedal_position));
        map
    }
    /// Serializes current instance state into a data transfer object.
    ///
    /// # Returns
    ///
    /// [`EffectDto::Wah`] wrapping updated operational properties.
    fn to_dto(&self) -> EffectDto {
        EffectDto::Wah(WahDto {
            id: self.id.to_string(),
            name: self.name.clone(),
            is_active: self.is_active.load(Ordering::Relaxed),
            color: self.get_color(),
            pedal_position: self.position(),
        })
    }
    /// Conditional execution pipeline matching global system state.
    ///
    /// # Returns
    ///
    /// Filtered frequency signal output if engaged; otherwise passes through pristine unmodified audio.
    fn process_if_active(&mut self, sample: f32) -> f32 {
        if self.is_active.load(Ordering::Relaxed) {
            self.process(sample)
        } else {
            sample
        }
    }
}
