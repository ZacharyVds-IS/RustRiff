//! Data transfer object (DTO) for audio devices.
//!
//! This type is serialized and sent to the frontend to populate
//! input/output device selectors.

use serde::Serialize;

/// Serializable representation of an audio device.
#[derive(Serialize, Clone)]
pub struct AudioDeviceDto {
    /// Stable device identifier used by backend commands for lookup.
    pub id: String,
    /// Human-readable device name shown in the UI.
    pub name: String,
    /// Default sample rate the device is configured in.
    pub sample_rate: u32
}