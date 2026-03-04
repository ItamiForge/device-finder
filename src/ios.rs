use crate::{Device, DeviceDetector, DevicePlatform, DeviceStatus, DeviceType};
use anyhow::{Context, Result};
use std::process::Command;

pub struct IosDetector {
    platform: String,
}

impl Default for IosDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl IosDetector {
    pub fn new() -> Self {
        Self {
            platform: std::env::consts::OS.to_string(),
        }
    }

    fn run_simctl(&self, args: &[&str]) -> Result<String> {
        let mut cmd_args = vec!["simctl"];
        cmd_args.extend(args.iter().copied());
        let output = Command::new("xcrun")
            .args(&cmd_args)
            .output()
            .context("Failed to run xcrun simctl")?;

        if !output.status.success() {
            return Ok(String::new());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_device_list(&self, output: &str) -> Vec<Device> {
        let mut devices = Vec::new();
        let mut current_runtime = String::new();

        for line in output.lines() {
            let line = line.trim();

            // Skip empty lines and section headers
            if line.is_empty() || line.starts_with("==") || line.starts_with("Devices:") {
                continue;
            }

            // Detect runtime version header like "-- iOS 18.4 --"
            if line.starts_with("-- iOS") && line.ends_with("--") {
                current_runtime = line
                    .trim_start_matches("-- iOS ")
                    .trim_end_matches(" --")
                    .to_string();
                continue;
            }

            // Format: "    iPhone 16 Pro (UDID) (Status)"
            if let Some(paren_idx) = line.find(" (") {
                let name = line[..paren_idx].to_string();
                let rest = &line[paren_idx + 2..];

                if let Some((udid, status_part)) = rest.split_once(") (") {
                    let udid = udid.to_string();
                    let status_str = status_part.trim_end_matches(')');

                    let status = match status_str {
                        "Booted" => DeviceStatus::Connected,
                        "Shutdown" => DeviceStatus::Shutdown,
                        _ => DeviceStatus::Unknown,
                    };

                    let device_type = DeviceType::Simulator;

                    let mut device = Device::new(udid.clone(), name, DevicePlatform::IOS)
                        .with_type(device_type)
                        .with_status(status)
                        .with_udid(udid);

                    if !current_runtime.is_empty() {
                        device = device.with_os_version(current_runtime.clone());
                    }

                    devices.push(device);
                }
            }
        }

        devices
    }

    fn get_device_info(&self, udid: &str) -> Result<Device> {
        let output = self.run_simctl(&["list", "devices", "info", udid])?;

        let mut device = Device::new(udid.to_string(), String::new(), DevicePlatform::IOS)
            .with_type(DeviceType::Simulator)
            .with_udid(udid.to_string());

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("name:") {
                device.name = line.replace("name:", "").trim().to_string();
            } else if line.starts_with("deviceType:") {
                let dt = line.replace("deviceType:", "").trim().to_string();
                if dt.contains("iPhone") {
                    device.model = Some("iPhone".to_string());
                } else if dt.contains("iPad") {
                    device.model = Some("iPad".to_string());
                }
            } else if line.starts_with("runtime:") || line.starts_with("version:") {
                let version = line
                    .replace("runtime:", "")
                    .replace("version:", "")
                    .trim()
                    .to_string();
                if !version.is_empty() {
                    device = device.with_os_version(version);
                }
            }
        }

        if device.name.is_empty() {
            device.name = "iOS Simulator".to_string();
        }

        Ok(device)
    }
}

impl DeviceDetector for IosDetector {
    fn name(&self) -> &'static str {
        "iOS"
    }

    fn detect(&self) -> Result<Vec<Device>> {
        if self.platform != "macos" {
            return Ok(Vec::new());
        }

        let output = self.run_simctl(&["list", "devices", "available"])?;

        if output.is_empty() {
            return Ok(Vec::new());
        }

        let mut devices = Vec::new();
        let mut current_booted: Option<String> = None;

        for line in output.lines() {
            let line = line.trim();

            if line.contains("(Booted)") {
                if let Some((udid, _)) = line.split_once('(') {
                    current_booted = Some(udid.trim().to_string());
                }
            }
        }

        let parsed = self.parse_device_list(&output);

        for mut device in parsed {
            if let Some(ref booted_udid) = current_booted {
                if device.id == *booted_udid {
                    device.status = DeviceStatus::Connected;
                }
            }

            if let Ok(info) = self.get_device_info(&device.id) {
                if !info.name.is_empty() && info.name != "iOS Simulator" {
                    device.name = info.name;
                }
                if info.os_version.is_some() {
                    device.os_version = info.os_version.clone();
                }
                if info.model.is_some() {
                    device.model = info.model.clone();
                }
            }

            devices.push(device);
        }

        Ok(devices)
    }

    fn is_available(&self) -> bool {
        if self.platform != "macos" {
            return false;
        }

        Command::new("xcrun")
            .args(["simctl", "list", "devices"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
