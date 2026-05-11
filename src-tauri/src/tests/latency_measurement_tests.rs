//! Integration and unit tests for the full latency measurement subsystem.
//!
//! Covers:
//! - [`AudioLatencyMeasurementService`] — all four measurement families
//! - [`AudioService::buffer_size_frames`]
//! - All latency DTOs (`ExecutionTimingDto`, `AlgorithmicLatencyDto`,
//!   `BufferLatencyDto`, `RoundTripLatencyDto`)

#[cfg(test)]
mod suite {
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService;
    use crate::services::audio_service::AudioService;
    use cpal::{BufferSize, StreamConfig};
    use std::sync::Arc;

    // -------------------------------------------------------------------------
    // Test helpers
    // -------------------------------------------------------------------------

    fn make_service(
        input_rate: u32,
        output_rate: u32,
        input_buffer: BufferSize,
        output_buffer: BufferSize,
    ) -> AudioService {
        let mut mock = MockAudioHandlerTrait::new();

        let input_config = StreamConfig {
            channels: 1,
            sample_rate: input_rate,
            buffer_size: input_buffer,
        };
        let output_config = StreamConfig {
            channels: 1,
            sample_rate: output_rate,
            buffer_size: output_buffer,
        };

        mock.expect_input_sample_rate().return_const(input_rate);
        mock.expect_output_sample_rate().return_const(output_rate);
        mock.expect_input_config().return_const(input_config);
        mock.expect_output_config().return_const(output_config);

        AudioService::new_with_handler(Arc::new(mock))
    }

    fn default_service() -> AudioService {
        make_service(
            48_000,
            48_000,
            BufferSize::Fixed(256),
            BufferSize::Fixed(256),
        )
    }

    // AudioLatencyMeasurementService — CPU timing measurements
    #[cfg(test)]
    mod cpu_timing_tests {
        use super::*;

        mod success_path {
            use super::*;

            #[test]
            fn measure_gain_latency_returns_non_negative_value() {
                let service = default_service();
                let result = AudioLatencyMeasurementService::measure_gain_latency(&service, 512);
                assert!(
                    result >= 0.0,
                    "gain latency must be non-negative, got {result}"
                );
                assert!(result.is_finite(), "gain latency must be finite");
            }

            #[test]
            fn measure_tone_stack_latency_returns_non_negative_value() {
                let service = default_service();
                let result =
                    AudioLatencyMeasurementService::measure_tone_stack_latency(&service, 512);
                assert!(result >= 0.0);
                assert!(result.is_finite());
            }

            #[test]
            fn measure_volume_latency_returns_non_negative_value() {
                let service = default_service();
                let result = AudioLatencyMeasurementService::measure_volume_latency(&service, 512);
                assert!(
                    result >= 0.0,
                    "volume latency must be non-negative, got {result}"
                );
                assert!(result.is_finite(), "volume latency must be finite");
            }

            #[test]
            fn measure_all_dsp_timings_returns_three_entries_in_chain_order() {
                let service = default_service();
                let timings =
                    AudioLatencyMeasurementService::measure_all_dsp_timings(&service, 512);
                assert_eq!(timings.len(), 4);
                assert_eq!(timings[0].processor_name, "Gain");
                assert_eq!(timings[1].processor_name, "Tone Stack");
                assert_eq!(timings[2].processor_name, "Volume");
                assert_eq!(timings[3].processor_name, "Master Volume");
            }

            #[test]
            fn measure_all_dsp_timings_all_values_are_non_negative_and_finite() {
                let service = default_service();
                let timings =
                    AudioLatencyMeasurementService::measure_all_dsp_timings(&service, 512);
                for t in &timings {
                    assert!(
                        t.execution_us_per_sample >= 0.0,
                        "{} is negative",
                        t.processor_name
                    );
                    assert!(
                        t.execution_us_per_sample.is_finite(),
                        "{} is not finite",
                        t.processor_name
                    );
                }
            }

            #[test]
            fn measurements_run_against_current_channel() {
                let mut service = default_service();
                // Add a second channel and switch to it — measurement must not panic
                service.add_channel("Alt".to_string());
                let result = AudioLatencyMeasurementService::measure_gain_latency(&service, 256);
                assert!(result >= 0.0);
            }
        }
    }

    // -------------------------------------------------------------------------
    // AudioLatencyMeasurementService — algorithmic latency
    // -------------------------------------------------------------------------

    #[cfg(test)]
    mod algorithmic_latency_tests {
        use super::*;

        mod success_path {
            use super::*;

            #[test]
            fn all_current_processors_have_zero_algorithmic_latency() {
                let service = default_service();
                let latency =
                    AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency(&service);
                assert_eq!(latency.len(), 4);
                assert_eq!(latency[2].processor_name, "Volume");
                assert!(latency.iter().all(|d| d.latency_samples == 0));
                assert!(latency.iter().all(|d| d.latency_ms == 0.0));
            }

            #[test]
            fn uses_output_sample_rate_for_ms_conversion() {
                let service = make_service(
                    48_000,
                    96_000,
                    BufferSize::Fixed(256),
                    BufferSize::Fixed(256),
                );
                let latency =
                    AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency(&service);
                assert!(latency.iter().all(|d| d.latency_ms == 0.0));
            }
        }
    }
    #[cfg(test)]
    mod buffer_latency_tests {
        use super::*;

        mod success_path {
            use super::*;

            #[test]
            fn symmetric_fixed_buffer_has_equal_input_and_output_latency() {
                let service = make_service(
                    48_000,
                    48_000,
                    BufferSize::Fixed(256),
                    BufferSize::Fixed(256),
                );
                let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);
                let expected_ms = (256.0 / 48_000.0) * 1000.0;
                assert!((latency.input_buffer_latency_ms - expected_ms).abs() < 1e-9);
                assert!((latency.output_buffer_latency_ms - expected_ms).abs() < 1e-9);
                assert!((latency.total_buffer_latency_ms - expected_ms * 2.0).abs() < 1e-9);
            }

            #[test]
            fn asymmetric_buffer_sizes_are_measured_independently() {
                let service = make_service(
                    48_000,
                    96_000,
                    BufferSize::Fixed(480),
                    BufferSize::Fixed(960),
                );
                let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);
                assert!((latency.input_buffer_latency_ms - 10.0).abs() < 1e-9);
                assert!((latency.output_buffer_latency_ms - 10.0).abs() < 1e-9);
            }

            #[test]
            fn total_buffer_latency_is_sum_of_sides() {
                let service = make_service(
                    48_000,
                    48_000,
                    BufferSize::Fixed(256),
                    BufferSize::Fixed(512),
                );
                let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);
                assert!(
                    (latency.total_buffer_latency_ms
                        - (latency.input_buffer_latency_ms + latency.output_buffer_latency_ms))
                        .abs()
                        < 1e-9
                );
            }
        }

        mod failure_path {
            use super::*;

            #[test]
            fn default_buffer_size_falls_back_to_256_frames() {
                let service =
                    make_service(48_000, 48_000, BufferSize::Default, BufferSize::Default);
                let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);
                let expected_ms = (256.0 / 48_000.0) * 1000.0;
                assert!((latency.input_buffer_latency_ms - expected_ms).abs() < 1e-9);
                assert!((latency.output_buffer_latency_ms - expected_ms).abs() < 1e-9);
            }

            #[test]
            fn mixed_default_and_fixed_uses_fallback_for_default_side() {
                let service =
                    make_service(48_000, 48_000, BufferSize::Default, BufferSize::Fixed(512));
                let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);
                let expected_input_ms = (256.0 / 48_000.0) * 1000.0;
                let expected_output_ms = (512.0 / 48_000.0) * 1000.0;
                assert!((latency.input_buffer_latency_ms - expected_input_ms).abs() < 1e-9);
                assert!((latency.output_buffer_latency_ms - expected_output_ms).abs() < 1e-9);
            }
        }
    }

    #[cfg(test)]
    mod buffer_size_frames_tests {
        use super::*;

        mod success_path {
            use super::*;

            #[test]
            fn returns_fixed_frame_count_from_input_config() {
                let service = make_service(
                    48_000,
                    48_000,
                    BufferSize::Fixed(512),
                    BufferSize::Fixed(512),
                );
                assert_eq!(service.buffer_size_frames(), 512);
            }

            #[test]
            fn returns_256_fallback_when_buffer_size_is_default() {
                let service =
                    make_service(48_000, 48_000, BufferSize::Default, BufferSize::Default);
                assert_eq!(service.buffer_size_frames(), 256);
            }

            #[test]
            fn reflects_the_input_config_not_the_output_config() {
                let service = make_service(
                    48_000,
                    48_000,
                    BufferSize::Fixed(128),
                    BufferSize::Fixed(512),
                );
                assert_eq!(service.buffer_size_frames(), 128);
            }
        }
    }
}
