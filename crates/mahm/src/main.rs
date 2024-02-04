use mahm::monitor::HardwareMonitor;

#[macro_use]
extern crate log;

fn main() -> anyhow::Result<()> {
  std::env::set_var("RUST_LOG", "debug");
  pretty_env_logger::init();

  info!("hello awa");

  if let Ok(installation_path) = mahm::installation_path() {
    info!("MSI Afterburner is installed at {}", installation_path);
  } else {
    warn!("Could not find MSI Afterburner installation path");
  }

  let mut monitor = HardwareMonitor::new();

  monitor.refresh()?;

  if let Some(header) = &monitor.header {
    info!("header: {}", header);
  } else {
    warn!("no header");
  }

  for entry in &monitor.entries {
    info!("entry: {}", entry);
  }

  Ok(())
}
