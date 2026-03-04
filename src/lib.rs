use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod android;
pub mod config;
pub mod ios;
pub mod tui;
pub mod web;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum DevicePlatform {
    IOS,
    Android,
    Web,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Physical,
    Simulator,
    Emulator,
    WebContainer,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Connected,
    Disconnected,
    Booting,
    Shutdown,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub platform: DevicePlatform,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub details: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udid: Option<String>,
}

impl Device {
    pub fn new(id: String, name: String, platform: DevicePlatform) -> Self {
        Self {
            id,
            name,
            platform,
            device_type: DeviceType::Unknown,
            status: DeviceStatus::Unknown,
            details: HashMap::new(),
            os_version: None,
            model: None,
            udid: None,
        }
    }

    pub fn with_type(mut self, device_type: DeviceType) -> Self {
        self.device_type = device_type;
        self
    }

    pub fn with_status(mut self, status: DeviceStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_os_version(mut self, version: String) -> Self {
        self.os_version = Some(version);
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_udid(mut self, udid: String) -> Self {
        self.udid = Some(udid);
        self
    }

    pub fn with_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }

    pub fn platform_name(&self) -> &str {
        match self.platform {
            DevicePlatform::IOS => "iOS",
            DevicePlatform::Android => "Android",
            DevicePlatform::Web => "Web",
            DevicePlatform::Unknown => "Unknown",
        }
    }

    pub fn type_name(&self) -> &str {
        match self.device_type {
            DeviceType::Physical => "Physical",
            DeviceType::Simulator => "Simulator",
            DeviceType::Emulator => "Emulator",
            DeviceType::WebContainer => "Web Container",
            DeviceType::Unknown => "Unknown",
        }
    }

    pub fn status_name(&self) -> &str {
        match self.status {
            DeviceStatus::Connected => "Connected",
            DeviceStatus::Disconnected => "Disconnected",
            DeviceStatus::Booting => "Booting",
            DeviceStatus::Shutdown => "Shutdown",
            DeviceStatus::Unknown => "Unknown",
        }
    }
}

pub trait DeviceDetector: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self) -> Result<Vec<Device>>;
    fn is_available(&self) -> bool;
}

pub fn all_detectors() -> Vec<Box<dyn DeviceDetector>> {
    vec![
        Box::new(ios::IosDetector::new()),
        Box::new(android::AndroidDetector::new()),
        Box::new(web::WebDetector::new()),
    ]
}

pub fn detect_all_devices() -> Result<Vec<Device>> {
    let mut all_devices = Vec::new();

    for detector in all_detectors() {
        if detector.is_available() {
            match detector.detect() {
                Ok(devices) => all_devices.extend(devices),
                Err(e) => {
                    eprintln!("Warning: {} detection failed: {}", detector.name(), e);
                }
            }
        }
    }

    Ok(all_devices)
}

pub fn detect_platform_devices(platform: DevicePlatform) -> Result<Vec<Device>> {
    for detector in all_detectors() {
        match platform {
            DevicePlatform::IOS => {
                if detector.name() == "iOS" && detector.is_available() {
                    return detector.detect();
                }
            }
            DevicePlatform::Android => {
                if detector.name() == "Android" && detector.is_available() {
                    return detector.detect();
                }
            }
            DevicePlatform::Web => {
                if detector.name() == "Web" && detector.is_available() {
                    return detector.detect();
                }
            }
            DevicePlatform::Unknown => {}
        }
    }
    Ok(Vec::new())
}

// Action commands (placeholder implementations)
pub fn launch_device(_id: &str) -> Result<String> {
    Ok("Device launch initiated".to_string())
}

pub fn install_app(_id: &str, _bundle: &str) -> Result<String> {
    Ok("App installation initiated".to_string())
}

pub fn connect_device(_id: &str) -> Result<String> {
    Ok("Device connection established".to_string())
}
