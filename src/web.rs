use crate::{Device, DeviceDetector, DevicePlatform, DeviceStatus, DeviceType};
use anyhow::Result;
use std::process::Command;

pub struct WebDetector;

impl Default for WebDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl WebDetector {
    pub fn new() -> Self {
        Self
    }

    fn detect_browsers(&self) -> Vec<Device> {
        let mut devices = Vec::new();

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("defaults")
                .args([
                    "read",
                    "/Library/Application Support/Google/Chrome/Default/Preferences",
                    "profiles",
                ])
                .output()
            {
                if output.status.success() {
                    devices.push(
                        Device::new(
                            "chrome-default".to_string(),
                            "Google Chrome".to_string(),
                            DevicePlatform::Web,
                        )
                        .with_type(DeviceType::WebContainer)
                        .with_status(DeviceStatus::Connected)
                        .with_detail("browser".to_string(), "chrome".to_string()),
                    );
                }
            }
        }

        devices
    }

    fn detect_containers(&self) -> Vec<Device> {
        let mut devices = Vec::new();

        if let Ok(output) = Command::new("docker")
            .args(["ps", "--format", "{{.Names}}"])
            .output()
        {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let name = line.trim();
                    if !name.is_empty()
                        && (name.contains("web")
                            || name.contains("browser")
                            || name.contains("playwright"))
                    {
                        devices.push(
                            Device::new(
                                name.to_string(),
                                format!("Docker: {}", name),
                                DevicePlatform::Web,
                            )
                            .with_type(DeviceType::WebContainer)
                            .with_status(DeviceStatus::Connected)
                            .with_detail("container_type".to_string(), "docker".to_string()),
                        );
                    }
                }
            }
        }

        devices
    }
}

impl DeviceDetector for WebDetector {
    fn name(&self) -> &'static str {
        "Web"
    }

    fn detect(&self) -> Result<Vec<Device>> {
        let mut devices = Vec::new();

        let browsers = self.detect_browsers();
        devices.extend(browsers);

        let containers = self.detect_containers();
        devices.extend(containers);

        if devices.is_empty() {
            devices.push(
                Device::new(
                    "web-default".to_string(),
                    "Web Browser".to_string(),
                    DevicePlatform::Web,
                )
                .with_type(DeviceType::WebContainer)
                .with_status(DeviceStatus::Unknown)
                .with_detail(
                    "note".to_string(),
                    "Use --refresh to detect browsers".to_string(),
                ),
            );
        }

        Ok(devices)
    }

    fn is_available(&self) -> bool {
        true
    }
}
