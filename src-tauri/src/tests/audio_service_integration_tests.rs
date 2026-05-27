#[cfg(test)]
mod suite {
    use std::sync::{Arc, Mutex};

    use crate::domain::channel_manager::ChannelManager;
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use crate::services::audio_service::AudioService;
    use crate::tests::mock::{make_mock_handler, FakeStream};

    fn build_service(handler: MockAudioHandlerTrait) -> AudioService {
        AudioService::new_with_handler(
            Arc::new(handler),
            Arc::new(Mutex::new(ChannelManager::new())),
        )
    }

    fn is_active(service: &AudioService) -> bool {
        *service.is_active()
    }

    fn make_timed_stream_mock(
        input_rate: u32,
        output_rate: u32,
        stream_build_times: usize,
    ) -> MockAudioHandlerTrait {
        let mut mock = MockAudioHandlerTrait::new();
        mock.expect_build_input_stream()
            .times(stream_build_times)
            .returning(|_| Box::new(FakeStream));
        mock.expect_build_output_stream()
            .times(stream_build_times)
            .returning(|_| Box::new(FakeStream));
        mock.expect_input_sample_rate().return_const(input_rate);
        mock.expect_output_sample_rate().return_const(output_rate);
        mock
    }

    #[cfg(test)]
    mod start_loopback_service {
        use super::*;

        #[test]
        fn start_loopback_makes_service_active() {
            let mut service = build_service(make_mock_handler());

            service.start_loopback();

            assert!(is_active(&service));
            service.stop_loopback();
        }

        #[test]
        fn start_loopback_twice_does_not_spawn_second_thread() {
            let mock = make_timed_stream_mock(48_000, 48_000, 1);
            let mut service = build_service(mock);

            service.start_loopback();
            service.start_loopback();

            assert!(is_active(&service));
            service.stop_loopback();
        }
    }

    #[cfg(test)]
    mod toggle_loopback_service {
        use super::*;

        #[test]
        fn toggle_true_activates_service() {
            let mut service = build_service(make_mock_handler());

            service.toggle_loopback(true);

            assert!(is_active(&service));
            service.stop_loopback();
        }

        #[test]
        fn toggle_false_deactivates_service() {
            let mut service = build_service(make_mock_handler());

            service.toggle_loopback(true);
            service.toggle_loopback(false);

            assert!(!is_active(&service));
        }

        #[test]
        fn toggle_true_when_already_active_is_no_op() {
            let mock = make_timed_stream_mock(48_000, 48_000, 1);
            let mut service = build_service(mock);

            service.toggle_loopback(true);
            service.toggle_loopback(true);

            assert!(is_active(&service));
            service.stop_loopback();
        }

        #[test]
        fn toggle_false_when_already_inactive_is_no_op() {
            let mut mock = MockAudioHandlerTrait::new();
            mock.expect_build_input_stream()
                .times(0)
                .returning(|_| Box::new(FakeStream));
            mock.expect_build_output_stream()
                .times(0)
                .returning(|_| Box::new(FakeStream));
            mock.expect_input_sample_rate().return_const(48_000u32);
            mock.expect_output_sample_rate().return_const(48_000u32);

            let mut service = build_service(mock);

            service.toggle_loopback(false);

            assert!(!is_active(&service));
        }
    }

    #[cfg(test)]
    mod set_audio_handler_service {
        use super::*;
        #[test]
        fn swap_handler_while_inactive_leaves_service_inactive() {
            let mut service = build_service(make_mock_handler());

            service.set_audio_handler(Arc::new(make_mock_handler()));

            assert!(!is_active(&service));
        }

        #[test]
        fn swap_handler_while_inactive_does_not_build_streams() {
            let mut service = build_service(make_mock_handler());
            let mut new_handler = MockAudioHandlerTrait::new();
            new_handler
                .expect_build_input_stream()
                .times(0)
                .returning(|_| Box::new(FakeStream));
            new_handler
                .expect_build_output_stream()
                .times(0)
                .returning(|_| Box::new(FakeStream));
            new_handler
                .expect_input_sample_rate()
                .return_const(48_000u32);
            new_handler
                .expect_output_sample_rate()
                .return_const(48_000u32);

            service.set_audio_handler(Arc::new(new_handler));
        }

        #[test]
        fn swap_handler_while_active_keeps_service_active() {
            let mut service = build_service(make_mock_handler());

            service.toggle_loopback(true);

            let new_handler = make_timed_stream_mock(48_000, 48_000, 1);
            service.set_audio_handler(Arc::new(new_handler));

            assert!(is_active(&service));
            service.stop_loopback();
        }

        #[test]
        fn swap_handler_while_active_restarts_with_new_handler_exactly_once() {
            let mut service = build_service(make_mock_handler());

            service.toggle_loopback(true);

            let first = make_timed_stream_mock(48_000, 48_000, 1);
            service.set_audio_handler(Arc::new(first));

            service.set_audio_handler(Arc::new(make_mock_handler()));

            assert!(is_active(&service));
            service.stop_loopback();
        }
    }
}
