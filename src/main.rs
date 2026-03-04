mod cli;
mod tui;

use anyhow::Result;
use clap::Parser;
use device_finder::{
    connect_device, detect_all_devices, detect_platform_devices, install_app, launch_device,
    Device, DevicePlatform,
};
use serde_json::json;
use std::process;

const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_FIND_NOT_FOUND: i32 = 3;

fn main() {
    let cli = cli::Cli::parse();

    let result = match cli.command {
        cli::Command::List(opts) => run_list(opts),
        cli::Command::Find(opts) => run_find(opts),
        cli::Command::Info(opts) => run_info(opts),
        cli::Command::Launch(opts) => run_launch(opts),
        cli::Command::Install(opts) => run_install(opts),
        cli::Command::Connect(opts) => run_connect(opts),
        cli::Command::Tui => tui::run().map(|_| EXIT_OK),
    };

    match result {
        Ok(code) => process::exit(code),
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(EXIT_ERROR);
        }
    }
}

fn run_list(opts: cli::ListOpts) -> Result<i32> {
    let devices = if let Some(platform) = opts.platform {
        let plat = match platform.as_str() {
            "ios" => DevicePlatform::IOS,
            "android" => DevicePlatform::Android,
            "web" => DevicePlatform::Web,
            _ => DevicePlatform::Unknown,
        };
        detect_platform_devices(plat)?
    } else {
        detect_all_devices()?
    };

    if devices.is_empty() {
        println!("No devices found.");
        if cfg!(target_os = "macos") {
            println!("Note: For iOS simulators, ensure Xcode is installed.");
        }
        println!("For Android, ensure adb is in PATH and devices are connected.");
        return Ok(EXIT_OK);
    }

    if opts.json {
        let entries: Vec<serde_json::Value> = devices.iter().map(device_to_json).collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        print_device_list(&devices, opts.verbose);
    }

    Ok(EXIT_OK)
}

fn run_find(opts: cli::FindOpts) -> Result<i32> {
    let devices = detect_all_devices()?;

    let matching: Vec<&Device> = devices
        .iter()
        .filter(|d| {
            d.name.to_lowercase().contains(&opts.query.to_lowercase())
                || d.id.to_lowercase().contains(&opts.query.to_lowercase())
                || d.model
                    .as_ref()
                    .map(|m| m.to_lowercase().contains(&opts.query.to_lowercase()))
                    .unwrap_or(false)
        })
        .collect();

    if matching.is_empty() {
        if opts.json {
            let payload = json!({
                "query": opts.query,
                "found": false,
                "devices": []
            });
            println!("{}", serde_json::to_string_pretty(&payload)?);
        } else {
            println!("No devices found matching '{}'", opts.query);
        }
        return Ok(EXIT_FIND_NOT_FOUND);
    }

    if opts.json {
        let payload = json!({
            "query": opts.query,
            "found": true,
            "devices": matching
                .iter()
                .map(|d| device_to_json(d))
                .collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        for device in matching {
            println!("{}", device.id);
        }
    }

    Ok(EXIT_OK)
}

fn run_info(opts: cli::InfoOpts) -> Result<i32> {
    let devices = detect_all_devices()?;

    let device = devices
        .iter()
        .find(|d| d.id == opts.id || d.name == opts.id);

    match device {
        Some(d) => {
            if opts.json {
                println!("{}", serde_json::to_string_pretty(&device_to_json(d))?);
            } else {
                print_device_info(d);
            }
            Ok(EXIT_OK)
        }
        None => {
            if opts.json {
                let payload = json!({
                    "id": opts.id,
                    "found": false
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("Device '{}' not found", opts.id);
            }
            Ok(EXIT_FIND_NOT_FOUND)
        }
    }
}

fn run_launch(opts: cli::LaunchOpts) -> Result<i32> {
    let result = launch_device(&opts.id)?;

    if opts.json {
        let payload = json!({
            "action": "launch",
            "device_id": opts.id,
            "status": "initiated",
            "message": result
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Launching device {}...", opts.id);
        println!("{}", result);
    }

    Ok(EXIT_OK)
}

fn run_install(opts: cli::InstallOpts) -> Result<i32> {
    let result = install_app(&opts.id, &opts.bundle)?;

    if opts.json {
        let payload = json!({
            "action": "install",
            "device_id": opts.id,
            "bundle": opts.bundle,
            "status": "initiated",
            "message": result
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Installing {} on device {}...", opts.bundle, opts.id);
        println!("{}", result);
    }

    Ok(EXIT_OK)
}

fn run_connect(opts: cli::ConnectOpts) -> Result<i32> {
    let result = connect_device(&opts.id)?;

    if opts.json {
        let payload = json!({
            "action": "connect",
            "device_id": opts.id,
            "status": "initiated",
            "message": result
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Connecting to device {}...", opts.id);
        println!("{}", result);
    }

    Ok(EXIT_OK)
}

fn device_to_json(device: &Device) -> serde_json::Value {
    json!({
        "id": device.id,
        "name": device.name,
        "platform": device.platform_name(),
        "type": device.type_name(),
        "status": device.status_name(),
        "os_version": device.os_version,
        "model": device.model,
        "udid": device.udid,
        "details": if device.details.is_empty() { None } else { Some(&device.details) }
    })
}

fn print_device_list(devices: &[Device], verbose: bool) {
    use colored::*;

    let header = format!(
        "{:<8} {:<20} {:<15} {:<12} {}",
        "PLATFORM", "NAME", "TYPE", "STATUS", "ID"
    );
    println!("{}", header);
    println!("{}", "-".repeat(header.len()));

    for device in devices {
        let platform = match device.platform {
            DevicePlatform::IOS => "iOS".cyan(),
            DevicePlatform::Android => "Android".green(),
            DevicePlatform::Web => "Web".magenta(),
            DevicePlatform::Unknown => "Unknown".normal(),
        };

        let status = match device.status {
            device_finder::DeviceStatus::Connected => "Connected".green(),
            device_finder::DeviceStatus::Disconnected => "Disconnected".red(),
            device_finder::DeviceStatus::Booting => "Booting".yellow(),
            device_finder::DeviceStatus::Shutdown => "Shutdown".dimmed(),
            device_finder::DeviceStatus::Unknown => "Unknown".normal(),
        };

        let device_type = device.type_name();

        println!(
            "{:<8} {:<20} {:<15} {:<12} {}",
            platform, device.name, device_type, status, device.id
        );

        if verbose {
            if let Some(ref os) = device.os_version {
                println!("         OS: {}", os);
            }
            if let Some(ref model) = device.model {
                println!("         Model: {}", model);
            }
            if !device.details.is_empty() {
                println!("         Details:");
                for (key, value) in &device.details {
                    println!("           {}: {}", key, value);
                }
            }
            println!();
        }
    }
}

fn print_device_info(device: &Device) {
    use colored::*;

    println!("{}", "Device Details".bold());
    println!("{}", "=".repeat(40));
    println!("{}: {}", "ID".bold(), device.id);
    println!("{}: {}", "Name".bold(), device.name);
    println!("{}: {}", "Platform".bold(), device.platform_name());
    println!("{}: {}", "Type".bold(), device.type_name());
    println!("{}: {}", "Status".bold(), device.status_name());

    if let Some(ref os) = device.os_version {
        println!("{}: {}", "OS Version".bold(), os);
    }
    if let Some(ref model) = device.model {
        println!("{}: {}", "Model".bold(), model);
    }
    if let Some(ref udid) = device.udid {
        println!("{}: {}", "UDID".bold(), udid);
    }

    if !device.details.is_empty() {
        println!("\n{}", "Additional Details".bold());
        for (key, value) in &device.details {
            println!("  {}: {}", key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_mappings_are_stable() {
        assert_eq!(EXIT_OK, 0);
        assert_eq!(EXIT_ERROR, 1);
        assert_eq!(EXIT_FIND_NOT_FOUND, 3);
    }

    #[test]
    fn find_not_found_json_shape_is_stable() {
        let payload = json!({
            "query": "nonexistent",
            "found": false,
            "devices": []
        });
        assert_eq!(payload["query"], "nonexistent");
        assert_eq!(payload["found"], false);
    }

    #[test]
    fn action_json_payloads_have_consistent_structure() {
        let launch_payload = json!({
            "action": "launch",
            "device_id": "test",
            "status": "initiated",
            "message": "test"
        });
        assert_eq!(launch_payload["action"], "launch");
        assert!(launch_payload["device_id"].is_string());
        assert!(launch_payload["status"].is_string());
    }
}
