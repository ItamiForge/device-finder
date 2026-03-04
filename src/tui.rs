use anyhow::Result;

/// Minimal TUI mode - just show a message and exit
/// Full TUI implementation can be added later
pub fn run() -> Result<()> {
    println!("Device Finder - Interactive TUI");
    println!("\nTUI mode is available. Use 'df list' for device listing.");
    println!("\nFull TUI implementation coming soon.");
    Ok(())
}
