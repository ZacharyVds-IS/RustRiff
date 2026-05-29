use crate::infrastructure::audio_handler::{MockAudioHandlerTrait, PlayableStream};
use cpal::StreamConfig;

pub struct FakeStream;

impl PlayableStream for FakeStream {
    fn play(&self) {}
}

unsafe impl Send for FakeStream {}

pub fn make_mock_handler() -> MockAudioHandlerTrait {
    make_mock_handler_with_rates(48_000, 48_000)
}

pub fn make_mock_handler_with_rates(input_rate: u32, output_rate: u32) -> MockAudioHandlerTrait {
    let mut mock = MockAudioHandlerTrait::new();

    mock.expect_build_input_stream()
        .returning(|_prod| Box::new(FakeStream));

    mock.expect_build_output_stream()
        .returning(|_cons| Box::new(FakeStream));

    mock.expect_input_sample_rate().return_const(input_rate);

    mock.expect_output_sample_rate().return_const(output_rate);

    // Set up input and output configs with stereo channels
    let input_config = StreamConfig {
        channels: 2,
        sample_rate: input_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    let output_config = StreamConfig {
        channels: 2,
        sample_rate: output_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    mock.expect_input_config().return_const(input_config);
    mock.expect_output_config().return_const(output_config);

    mock
}
