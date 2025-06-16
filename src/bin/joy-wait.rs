use anyhow::Result;
use evdev::{Device, EventSummary};

fn main() -> Result<()> {
    let device_path = find_controller_device()?;
    let mut device = Device::open(&device_path)?;

    loop {
        for event in device.fetch_events()? {
            if let EventSummary::Key(_event, _key, value) = event.destructure() {
                if value == 1 {
                    return Ok(());
                }
            }
        }
    }
}

fn find_controller_device() -> Result<String> {
    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with("event") {
                if let Ok(device) = Device::open(&path) {
                    if let Some(name) = device.name() {
                        if name.to_lowercase().contains("controller")
                            || name.to_lowercase().contains("gamepad")
                        {
                            return Ok(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    anyhow::bail!("No controller found")
}
