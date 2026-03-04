use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "df")]
#[command(version = "0.1.0")]
#[command(about = "Cross-platform device discovery and management for developers", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all devices
    List(ListOpts),
    /// Find device by name, ID, or model
    Find(FindOpts),
    /// Show detailed device information
    Info(InfoOpts),
    /// Launch/boot a device
    Launch(LaunchOpts),
    /// Install app on device
    Install(InstallOpts),
    /// Connect to device
    Connect(ConnectOpts),
    /// Interactive terminal UI
    Tui,
}

#[derive(clap::Args)]
pub struct ListOpts {
    /// Filter by platform (ios, android, web)
    #[arg(short, long)]
    pub platform: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Show verbose output with details
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(clap::Args)]
pub struct FindOpts {
    /// Search query (device name, id, or model)
    pub query: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct InfoOpts {
    /// Device ID or name
    pub id: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct LaunchOpts {
    /// Device ID
    pub id: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct InstallOpts {
    /// Device ID
    pub id: String,

    /// Bundle/app path
    pub bundle: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args)]
pub struct ConnectOpts {
    /// Device ID
    pub id: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}
