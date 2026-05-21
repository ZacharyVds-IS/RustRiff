use crate::domain::dto::audio_device_dto::AudioDeviceDto;
use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::{available_hosts, default_host, host_from_id, Device, Host};
use std::sync::Mutex;
use tracing::error;

const AUDIO_DRIVER_DEFAULT: &str = "Default";
const AUDIO_DRIVER_ASIO: &str = "ASIO";

/// Service for managing audio device enumeration and lookup.
///
/// `DeviceService` wraps a CPAL [`Host`] and provides convenient methods to:
/// - List available input and output devices
/// - Look up devices by their ID
/// - Convert device information into [`AudioDeviceDto`] for frontend consumption
#[derive(Default)]
pub struct DeviceService {
    selected_audio_driver: Mutex<String>,
}

impl DeviceService {
    /// Creates a new `DeviceService` initialized to the default driver.
    ///
    /// # Returns
    ///
    /// A new instance of `DeviceService`.
    pub fn new() -> Self {
        Self {
            selected_audio_driver: Mutex::new(AUDIO_DRIVER_DEFAULT.to_string()),
        }
    }

    /// Retrieves a list of available audio driver names supported by the system.
    ///
    /// On Windows, this includes both "Default" (WASAPI) and "ASIO". On other operating
    /// systems, only the "Default" option is available.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of [`String`] containing the available driver names.
    pub fn available_audio_drivers(&self) -> Vec<String> {
        if cfg!(target_os = "windows") {
            vec![
                AUDIO_DRIVER_DEFAULT.to_string(),
                AUDIO_DRIVER_ASIO.to_string(),
            ]
        } else {
            vec![AUDIO_DRIVER_DEFAULT.to_string()]
        }
    }

    /// Gets the currently selected audio driver name.
    ///
    /// If the internal lock is poisoned, it falls back to returning the default driver name.
    ///
    /// # Returns
    ///
    /// A [`String`] representing the active audio driver.
    pub fn selected_audio_driver(&self) -> String {
        self.selected_audio_driver
            .lock()
            .map(|driver| driver.clone())
            .unwrap_or_else(|_| AUDIO_DRIVER_DEFAULT.to_string())
    }

    /// Sets the selected audio driver to the specified driver name.
    ///
    /// The input string is normalized and validated against the platform's supported drivers.
    ///
    /// # Arguments
    ///
    /// * `driver` - A string slice representing the desired driver name.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver is unsupported or if the internal lock is poisoned.
    pub fn set_selected_audio_driver(&self, driver: &str) -> Result<(), String> {
        let normalized = Self::normalize_driver(driver)
            .ok_or_else(|| format!("Unsupported audio driver '{}'.", driver))?;

        let mut selected = self
            .selected_audio_driver
            .lock()
            .map_err(|_| "Failed to lock selected audio driver".to_string())?;
        *selected = normalized.to_string();
        Ok(())
    }

    /// Checks whether the ASIO audio driver is currently selected.
    ///
    /// This will always return `false` on non-Windows platforms.
    ///
    /// # Returns
    ///
    /// `true` if the system is Windows and ASIO is selected, `false` otherwise.
    pub fn is_asio_selected(&self) -> bool {
        cfg!(target_os = "windows") && self.selected_audio_driver() == AUDIO_DRIVER_ASIO
    }

    /// Normalizes and validates an audio driver name string.
    ///
    /// Matches the string case-insensitively against available options based on the platform.
    ///
    /// # Arguments
    ///
    /// * `driver` - The driver name string slice to normalize.
    ///
    /// # Returns
    ///
    /// `Some(&'static str)` containing the canonical driver name if valid, otherwise `None`.
    fn normalize_driver(driver: &str) -> Option<&'static str> {
        if driver.eq_ignore_ascii_case(AUDIO_DRIVER_DEFAULT) {
            return Some(AUDIO_DRIVER_DEFAULT);
        }

        if cfg!(target_os = "windows") && driver.eq_ignore_ascii_case(AUDIO_DRIVER_ASIO) {
            return Some(AUDIO_DRIVER_ASIO);
        }

        None
    }

    /// Obtains a CPAL [`Host`] instance matching the currently selected driver.
    ///
    /// # Errors
    ///
    /// Returns an error if the host initialization fails or the driver is unavailable.
    fn host_for_selected_driver(&self) -> Result<Host, String> {
        Self::host_for_driver(&self.selected_audio_driver())
    }

    /// Obtains a CPAL [`Host`] instance for a given driver name.
    ///
    /// Maps "ASIO" to the Asio host and "Default" to the Wasapi host on Windows.
    /// On other platforms, it returns the platform's default host.
    ///
    /// # Arguments
    ///
    /// * `driver` - The driver name to find a host for.
    ///
    /// # Errors
    ///
    /// Returns an error if the required host cannot be located or initialized.
    fn host_for_driver(driver: &str) -> Result<Host, String> {
        if cfg!(target_os = "windows") {
            if driver.eq_ignore_ascii_case(AUDIO_DRIVER_ASIO) {
                return Self::host_from_backend_name("Asio");
            }

            return Self::host_from_backend_name("Wasapi").or_else(|_| Ok(default_host()));
        }

        Ok(default_host())
    }

    /// Iterates through available CPAL host IDs to find one matching the requested backend name.
    ///
    /// # Arguments
    ///
    /// * `backend_name` - The case-insensitive debug name of the desired backend (e.g., "Wasapi", "Asio").
    ///
    /// # Errors
    ///
    /// Returns an error if the host is not found or fails to initialize.
    fn host_from_backend_name(backend_name: &str) -> Result<Host, String> {
        for host_id in available_hosts() {
            if format!("{:?}", host_id).eq_ignore_ascii_case(backend_name) {
                return host_from_id(host_id)
                    .map_err(|e| format!("Failed to initialize {} host: {}", backend_name, e));
            }
        }

        Err(format!("{} host is not available", backend_name))
    }

    /// Converts a CPAL [`Device`] and sample rate into an [`AudioDeviceDto`].
    ///
    /// # Arguments
    ///
    /// * `device` - The CPAL device instance.
    /// * `sample_rate` - The sample rate to assign to the DTO.
    ///
    /// # Returns
    ///
    /// `Some(AudioDeviceDto)` if the device properties were successfully queried, otherwise `None`.
    fn device_to_audio_device_dto(device: Device, sample_rate: u32) -> Option<AudioDeviceDto> {
        let desc = device.description().ok()?;
        let name = desc.name().to_string();
        let device_id = device.id().ok()?;
        let id = format!("{:?}", device_id);

        Some(AudioDeviceDto {
            id,
            name,
            sample_rate,
        })
    }

    /// Enumerates duplex devices that support both input and output operations.
    ///
    /// This is primarily used for ASIO handling, where a single device handles both stream directions.
    /// The assigned sample rate will be the minimum of the default input and output configurations.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of [`AudioDeviceDto`] objects representing valid duplex devices.
    fn get_duplex_devices(&self) -> Vec<AudioDeviceDto> {
        let host = match self.host_for_selected_driver() {
            Ok(host) => host,
            Err(e) => {
                error!("Failed to initialize host for duplex device listing: {}", e);
                return vec![];
            }
        };

        match host.devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let input_config = device.default_input_config().ok()?;
                    let output_config = device.default_output_config().ok()?;
                    let sample_rate = input_config.sample_rate().min(output_config.sample_rate());
                    Self::device_to_audio_device_dto(device, sample_rate)
                })
                .collect(),
            Err(e) => {
                error!("Failed to get duplex devices: {}", e);
                vec![]
            }
        }
    }

    /// Retrieves the default input and output devices for the currently selected driver.
    ///
    /// For ASIO, this searches for the first device supporting both input and output configurations
    /// and returns it for both tuple elements. For standard drivers, it retrieves the host's separate
    /// default input and output devices.
    ///
    /// # Errors
    ///
    /// Returns an error if the host cannot be reached, device enumeration fails, or a default device
    /// cannot be found.
    pub fn default_devices_for_selected_driver(&self) -> Result<(Device, Device), String> {
        let host = self.host_for_selected_driver()?;

        if self.is_asio_selected() {
            let device = host
                .devices()
                .map_err(|e| format!("Failed to enumerate ASIO devices: {}", e))?
                .find(|device| {
                    device.default_input_config().is_ok() && device.default_output_config().is_ok()
                })
                .ok_or_else(|| {
                    "No ASIO device with both input and output support was found".to_string()
                })?;

            return Ok((device.clone(), device));
        }

        let input = host
            .default_input_device()
            .ok_or_else(|| "No default input device found".to_string())?;
        let output = host
            .default_output_device()
            .ok_or_else(|| "No default output device found".to_string())?;
        Ok((input, output))
    }

    /// Retrieves a list of all available input devices.
    ///
    /// Queries the CPAL host for input devices, converts them to [`AudioDeviceDto`],
    /// and returns them. If device enumeration fails, an empty list is returned
    /// and an error is added to the logs.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of [`AudioDeviceDto`] representing available input devices.
    pub fn get_input_devices(&self) -> Vec<AudioDeviceDto> {
        if self.is_asio_selected() {
            return self.get_duplex_devices();
        }

        let host = match self.host_for_selected_driver() {
            Ok(host) => host,
            Err(e) => {
                error!("Failed to initialize host for input device listing: {}", e);
                return vec![];
            }
        };

        match host.input_devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let device_config = device.default_input_config().ok()?;
                    Self::device_to_audio_device_dto(device, device_config.sample_rate())
                })
                .collect(),
            Err(e) => {
                error!("Failed to get input devices: {}", e);
                vec![]
            }
        }
    }

    /// Retrieves a list of all available output devices.
    ///
    /// Queries the CPAL host for output devices, converts them to [`AudioDeviceDto`],
    /// and returns them. If device enumeration fails, an empty list is returned
    /// and an error is printed to stderr.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of [`AudioDeviceDto`] representing available output devices.
    pub fn get_output_devices(&self) -> Vec<AudioDeviceDto> {
        if self.is_asio_selected() {
            return self.get_duplex_devices();
        }

        let host = match self.host_for_selected_driver() {
            Ok(host) => host,
            Err(e) => {
                error!("Failed to initialize host for output device listing: {}", e);
                return vec![];
            }
        };

        match host.output_devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let device_config = device.default_output_config().ok()?;
                    Self::device_to_audio_device_dto(device, device_config.sample_rate())
                })
                .collect(),
            Err(e) => {
                error!("Failed to get output devices: {}", e);
                vec![]
            }
        }
    }

    /// Finds an input device by its string ID.
    ///
    /// Searches through the host's input devices for one whose debug-formatted
    /// ID matches the given string.
    ///
    /// # Arguments
    ///
    /// * `id` - The device ID string to search for (debug-formatted CPAL device ID).
    ///
    /// # Returns
    ///
    /// `Some(device)` if a matching input device is found, `None` otherwise.
    pub fn find_input_device_by_id(&self, id: &str) -> Option<Device> {
        if self.is_asio_selected() {
            return self.find_duplex_device_by_id(id);
        }

        let host = self.host_for_selected_driver().ok()?;
        let devices = host.input_devices().ok()?;

        for device in devices {
            let device_id = device.id().ok()?;
            if format!("{:?}", device_id) == id {
                return Some(device);
            }
        }

        None
    }

    /// Finds an output device by its string ID.
    ///
    /// Searches through the host's output devices for one whose debug-formatted
    /// ID matches the given string.
    ///
    /// # Arguments
    ///
    /// * `id` - The device ID string to search for (debug-formatted CPAL device ID).
    ///
    /// # Returns
    ///
    /// `Some(device)` if a matching output device is found, `None` otherwise.
    pub fn find_output_device_by_id(&self, id: &str) -> Option<Device> {
        if self.is_asio_selected() {
            return self.find_duplex_device_by_id(id);
        }

        let host = self.host_for_selected_driver().ok()?;
        let devices = host.output_devices().ok()?;

        for device in devices {
            let device_id = device.id().ok()?;
            if format!("{:?}", device_id) == id {
                return Some(device);
            }
        }

        None
    }

    /// Finds a duplex device supporting both input and output by its string ID.
    ///
    /// Utilized during ASIO configurations where inputs and outputs belong to the same hardware device.
    ///
    /// # Arguments
    ///
    /// * `id` - The device ID string to search for (debug-formatted CPAL device ID).
    ///
    /// # Returns
    ///
    /// `Some(device)` if a matching duplex device is found, `None` otherwise.
    fn find_duplex_device_by_id(&self, id: &str) -> Option<Device> {
        let host = self.host_for_selected_driver().ok()?;
        let devices = host.devices().ok()?;

        for device in devices {
            if device.default_input_config().is_err() || device.default_output_config().is_err() {
                continue;
            }

            let device_id = device.id().ok()?;
            if format!("{:?}", device_id) == id {
                return Some(device);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_test_host_available() -> bool {
        if let Ok(host) = DeviceService::new().host_for_selected_driver() {
            if let Ok(mut devices) = host.devices() {
                return devices.next().is_some();
            }
        }
        false
    }

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn test_new_initialization() {
            let service = DeviceService::new();
            assert_eq!(service.selected_audio_driver(), "Default");
        }

        #[test]
        fn test_available_audio_drivers() {
            let service = DeviceService::new();
            let drivers = service.available_audio_drivers();
            assert!(!drivers.is_empty());
            assert!(drivers.contains(&"Default".to_string()));
            if cfg!(target_os = "windows") {
                assert!(drivers.contains(&"ASIO".to_string()));
            }
        }

        #[test]
        fn test_set_and_get_audio_driver() {
            let service = DeviceService::new();
            let result = service.set_selected_audio_driver("DeFaUlT");
            assert!(result.is_ok());
            assert_eq!(service.selected_audio_driver(), "Default");

            if cfg!(target_os = "windows") {
                let asio_result = service.set_selected_audio_driver("AsIo");
                assert!(asio_result.is_ok());
                assert_eq!(service.selected_audio_driver(), "ASIO");
                assert!(service.is_asio_selected());
            }
        }

        #[test]
        fn test_host_for_selected_driver() {
            let service = DeviceService::new();
            let host_res = service.host_for_selected_driver();
            assert!(host_res.is_ok());
        }

        #[test]
        fn test_device_to_audio_device_dto() {
            let service = DeviceService::new();
            let host = service.host_for_selected_driver().unwrap();
            if let Ok(mut devices) = host.devices() {
                if let Some(device) = devices.next() {
                    let dto = DeviceService::device_to_audio_device_dto(device, 44100);
                    if let Some(d) = dto {
                        assert!(!d.id.is_empty());
                        assert!(!d.name.is_empty());
                        assert_eq!(d.sample_rate, 44100);
                    }
                }
            }
        }

        #[test]
        fn test_get_input_and_output_devices_execution() {
            let service = DeviceService::new();
            let inputs = service.get_input_devices();
            let outputs = service.get_output_devices();
            if is_test_host_available() {
                // Fixed: Removed the redundant `|| true` to properly assert execution outcomes.
                assert!(!inputs.is_empty());
                assert!(!outputs.is_empty());
            }
        }

        #[test]
        fn test_default_devices_for_selected_driver() {
            let service = DeviceService::new();
            if is_test_host_available() {
                let result = service.default_devices_for_selected_driver();
                // Fixed: Asserting that the result is an Ok variant instead of executing `assert!(true)` inside a match block.
                assert!(result.is_ok());
            }
        }

        #[test]
        fn test_find_device_by_id_matching() {
            let service = DeviceService::new();
            let inputs = service.get_input_devices();
            if let Some(first_dto) = inputs.first() {
                let found = service.find_input_device_by_id(&first_dto.id);
                assert!(found.is_some());
            }
            let outputs = service.get_output_devices();
            if let Some(first_dto) = outputs.first() {
                let found = service.find_output_device_by_id(&first_dto.id);
                assert!(found.is_some());
            }
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn test_set_invalid_audio_driver() {
            let service = DeviceService::new();
            let result = service.set_selected_audio_driver("InvalidDriverName");
            assert!(result.is_err());
            assert_eq!(service.selected_audio_driver(), "Default");
        }

        #[test]
        fn test_is_asio_selected_on_non_windows() {
            let service = DeviceService::new();
            if !cfg!(target_os = "windows") {
                let _ = service.set_selected_audio_driver("ASIO");
                assert!(!service.is_asio_selected());
            }
        }

        #[test]
        fn test_find_input_device_by_non_existent_id() {
            let service = DeviceService::new();
            let found = service.find_input_device_by_id("NonExistentId12345");
            assert!(found.is_none());
        }

        #[test]
        fn test_find_output_device_by_non_existent_id() {
            let service = DeviceService::new();
            let found = service.find_output_device_by_id("NonExistentId12345");
            assert!(found.is_none());
        }

        #[test]
        fn test_host_from_backend_name_not_found() {
            let result = DeviceService::host_from_backend_name("NonExistentBackendName");
            assert!(result.is_err());
        }
    }
}
