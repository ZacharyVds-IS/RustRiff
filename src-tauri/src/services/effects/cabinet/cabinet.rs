use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::cabinet_dto::CabinetDto;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use crate::infrastructure::file_loader::{FileLoader, FileLoaderTrait};
use crate::services::processors::resampler::resampler::ResamplerImpl;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

const DEFAULT_IR_FILE: &str = "reverb_oxford_lean.wav";
const IR_RESAMPLER_CHUNK_SIZE: usize = 256;

pub struct Cabinet {
	id: u32,
	name: String,
	is_active: Arc<AtomicBool>,
	color: String,
	ir_buffer: Vec<f32>,
	// Convolution kernel in frequency domain (complex: magnitude + phase).
	ir_fft_kernel: Vec<Complex<f32>>,
	ir_fft_size: usize,
	dsp_sample_rate: u32,
}

impl Cabinet {
	pub fn new(id: u32, name: String, is_active: bool, color: String, dsp_sample_rate: u32) -> Self {
		info!("init cabinet simulation");
		let file_loader = FileLoader::new();
		let temp_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("resources")
			.join("default_ir")
			.join(DEFAULT_IR_FILE);

		let ir_buffer = file_loader.read_wav_to_buffer(&temp_file_path);
		let ir_sample_rate = file_loader.read_wav_sample_rate(&temp_file_path).unwrap_or(dsp_sample_rate);
		let (ir_buffer, resampling_applied) =
			Self::resample_if_needed(ir_buffer, ir_sample_rate, dsp_sample_rate);
		let (ir_fft_kernel, ir_fft_size) = Self::convert_ir_to_fft_kernel(&ir_buffer);

		info!(
			"Cabinet rates -> ir_sample_rate={}, dsp_sample_rate={}, resampling_applied={}, fft_kernel_len={}, fft_size={}",
			ir_sample_rate,
			dsp_sample_rate,
			resampling_applied,
			ir_fft_kernel.len(),
			ir_fft_size
		);

		Self {
			id,
			name,
			is_active: Arc::new(AtomicBool::new(is_active)),
			color,
			ir_buffer,
			ir_fft_kernel,
			ir_fft_size,
			dsp_sample_rate,
		}
	}

	fn convert_ir_to_fft_kernel(ir_buffer: &[f32]) -> (Vec<Complex<f32>>, usize) {
		if ir_buffer.is_empty() {
			return (Vec::new(), 0);
		}
		
		let fft_size = ir_buffer.len().next_power_of_two().max(2);
		let mut planner = FftPlanner::<f32>::new();
		let fft = planner.plan_fft_forward(fft_size);

		let mut buffer = vec![Complex::new(0.0_f32, 0.0_f32); fft_size];
		for (idx, sample) in ir_buffer.iter().enumerate() {
			buffer[idx].re = *sample;
		}

		fft.process(&mut buffer);
		(buffer, fft_size)
	}

	fn resample_if_needed(buffer: Vec<f32>, source_rate: u32, target_rate: u32) -> (Vec<f32>, bool) {
		if buffer.len() < 2 || source_rate == 0 || target_rate == 0 || source_rate == target_rate {
			return (buffer, false);
		}

		let mut resampler = match ResamplerImpl::new(source_rate, target_rate, IR_RESAMPLER_CHUNK_SIZE)
		{
			Ok(resampler) => resampler,
			Err(err) => {
				warn!(
					"Failed to initialize cabinet IR resampler ({} -> {}): {}. Using original IR buffer.",
					source_rate,
					target_rate,
					err
				);
				return (buffer, false);
			}
		};

		let mut out = Vec::new();
		for &sample in &buffer {
			out.extend(resampler.process_sample(sample));
		}
		out.extend(resampler.flush());

		if out.is_empty() {
			warn!(
				"Cabinet IR resampling produced no output ({} -> {}). Using original IR buffer.",
				source_rate,
				target_rate
			);
			return (buffer, false);
		}

		(out, true)
	}

	pub fn sample_rate(&self) -> u32 {
		self.dsp_sample_rate
	}


	pub fn ir_fft_kernel(&self) -> &[Complex<f32>] {
		&self.ir_fft_kernel
	}

	pub fn ir_fft_size(&self) -> usize {
		self.ir_fft_size
	}
}

impl AudioProcessor for Cabinet {
	fn process(&mut self, sample: f32) -> f32 {
		sample
	}
}

impl Effect for Cabinet {
	fn id(&self) -> u32 {
		self.id
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn get_color(&self) -> String {
		self.color.clone()
	}

	fn to_dto(&self) -> EffectDto {
		EffectDto::Cabinet(CabinetDto {
			id: self.id,
			name: self.name.clone(),
			is_active: self.is_active.load(Ordering::Relaxed),
			color: self.color.clone()
		})
	}

	fn active_flag(&self) -> Arc<AtomicBool> {
		Arc::clone(&self.is_active)
	}
}

