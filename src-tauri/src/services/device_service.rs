use crate::domain::dto::audio_device_dto::AudioDeviceDto;
use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::Host;
use tracing::error;

/// Service for managing audio device enumeration and lookup.
///
/// `DeviceService` wraps a CPAL [`Host`] and provides convenient methods to:
/// - List available input and output devices
/// - Look up devices by their ID
/// - Convert device information into [`AudioDeviceDto`] for frontend consumption
pub struct DeviceService {
    host:Host
}


impl DeviceService {
    /// Creates a new `DeviceService` with the given CPAL host.
    ///
    /// # Arguments
    ///
    /// * `host` - A CPAL [`Host`] instance.
    pub fn new(host: Host) -> Self {
        Self { host }
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
        match self.host.input_devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let desc = device.description().ok()?;
                    let name = desc.name().to_string();
                    let device_id = device.id().ok()?;
                    let id = format!("{:?}", device_id);
                    let device_config = device.default_input_config().ok()?;
                    let default_sample_rate = device_config.sample_rate();

                    Some(AudioDeviceDto {
                        id,
                        name,
                        sample_rate:default_sample_rate
                    })
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
        match self.host.output_devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let desc = device.description().ok()?;
                    let name = desc.name().to_string();
                    let device_id = device.id().ok()?;
                    let id = format!("{:?}", device_id);
                    let device_config = device.default_output_config().ok()?;
                    let default_sample_rate = device_config.sample_rate();

                    Some(AudioDeviceDto {
                        id,
                        name,
                        sample_rate: default_sample_rate,
                    })
                })
                .collect(),
            Err(e) => {
                eprintln!("Failed to get output devices: {}", e);
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
    pub fn find_input_device_by_id(&self, id: &str) -> Option<cpal::Device> {
        let devices = self.host.input_devices().ok()?;

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
    pub fn find_output_device_by_id(&self, id: &str) -> Option<cpal::Device> {
        let devices = self.host.output_devices().ok()?;

        for device in devices {
            let device_id = device.id().ok()?;
            if format!("{:?}", device_id) == id {
                return Some(device);
            }
        }

        None
    }


}