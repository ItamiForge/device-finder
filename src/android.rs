use crate::{Device, DeviceDetector, DevicePlatform, DeviceStatus, DeviceType};
use anyhow::{Context, Result};
use std::process::Command;

pub struct AndroidDetector;

impl Default for AndroidDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AndroidDetector {
    pub fn new() -> Self {
        Self
    }

    fn run_adb(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("adb")
            .args(args)
            .output()
            .context("Failed to run adb")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn run_emulator(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("emulator")
            .args(args)
            .output()
            .context("Failed to run emulator")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_devices(&self, output: &str) -> Vec<Device> {
        let mut devices = Vec::new();
        let mut lines = output.lines().peekable();

        lines.next();

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let id = parts[0].to_string();
                let status_str = parts[1];

                let status = match status_str {
                    "device" => DeviceStatus::Connected,
                    "unauthorized" => DeviceStatus::Disconnected,
                    "offline" => DeviceStatus::Disconnected,
                    _ => DeviceStatus::Unknown,
                };

                let device_type = if id.contains("emulator") || id.contains("localhost") {
                    DeviceType::Emulator
                } else {
                    DeviceType::Physical
                };

                let device = Device::new(
                    id.clone(),
                    format!("Android Device {}", &id[..8.min(id.len())]),
                    DevicePlatform::Android,
                )
                .with_type(device_type)
                .with_status(status)
                .with_udid(id);

                devices.push(device);
            }
        }

        devices
    }

    fn get_device_properties(&self, device: &mut Device) -> Result<()> {
        let output = self.run_adb(&["-s", &device.id, "shell", "getprop"])?;

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with('[') && line.contains("]: [") {
                if let Some((key, value)) = line.trim_start_matches('[').split_once("]: [") {
                    let key = key.trim();
                    let value = value.trim_end_matches(']');

                    match key {
                        "ro.product.model" => {
                            device.model = Some(value.to_string());
                            device.name = value.to_string();
                        }
                        "ro.build.version.release" | "ro.system.build.version.release" => {
                            device.os_version = Some(value.to_string());
                        }
                        "ro.product.brand" => {
                            device
                                .details
                                .insert("brand".to_string(), value.to_string());
                        }
                        "ro.product.device" | "ro.product.name" => {
                            device
                                .details
                                .insert("device".to_string(), value.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn detect_emulators(&self) -> Result<Vec<Device>> {
        let mut devices = Vec::new();

        if let Ok(output) = self.run_emulator(&["-list-avds"]) {
            for line in output.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    let device =
                        Device::new(line.to_string(), line.to_string(), DevicePlatform::Android)
                            .with_type(DeviceType::Emulator)
                            .with_status(DeviceStatus::Shutdown)
                            .with_detail("avd_name".to_string(), line.to_string());

                    devices.push(device);
                }
            }
        }

        if let Ok(output) = self.run_adb(&["devices", "-l"]) {
            for line in output.lines() {
                let line = line.trim();
                if line.contains("emulator") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(id) = parts.first() {
                        let mut device = Device::new(
                            id.to_string(),
                            format!("Emulator {}", id),
                            DevicePlatform::Android,
                        )
                        .with_type(DeviceType::Emulator)
                        .with_status(DeviceStatus::Connected)
                        .with_udid(id.to_string());

                        for part in parts.iter().skip(1) {
                            if part.starts_with("port:") {
                                let port = part.replace("port:", "");
                                device = device.with_detail("port".to_string(), port);
                            }
                        }

                        let _ = self.get_device_properties(&mut device);
                        devices.push(device);
                    }
                }
            }
        }

        Ok(devices)
    }
}

impl DeviceDetector for AndroidDetector {
    fn name(&self) -> &'static str {
        "Android"
    }

    fn detect(&self) -> Result<Vec<Device>> {
        let mut all_devices = Vec::new();

        let output = self.run_adb(&["devices", "-l"])?;

        if !output.is_empty() && !output.contains("List of devices") {
            let devices = self.parse_devices(&output);

            for mut device in devices {
                let _ = self.get_device_properties(&mut device);
                all_devices.push(device);
            }
        }

        let emulators = self.detect_emulators()?;
        all_devices.extend(emulators);

        Ok(all_devices)
    }

    fn is_available(&self) -> bool {
        Command::new("adb")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
