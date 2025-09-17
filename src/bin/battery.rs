use core_affinity::CoreId;
use lib::types::Response;

use std::{fs, path::PathBuf, thread, time::Duration};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Read instantaneous power (power_now)
    #[arg(long, default_value_t = false)]
    power: bool,

    /// Read battery capacity (capacity)
    #[arg(long, default_value_t = false)]
    capacity: bool,

    /// Path to the battery sysfs directory (contains `power_now` and `capacity`)
    #[arg(long)]
    path: Option<String>,

    /// Poll interval in seconds
    #[arg(long, default_value_t = 5)]
    interval: u64,

    /// Run once and exit instead of listening
    #[arg(long, default_value_t = false)]
    once: bool,
}

fn output(text: String, class: &str) {
    println!(
        "{}",
        serde_json::to_string(&Response {
            class,
            text: text.as_str()
        })
        .unwrap()
    );
}

fn read_power(path: &PathBuf) -> Result<Option<String>, ()> {
    let s = fs::read_to_string(path).map_err(|_| ())?;
    let s = s.trim();
    if s.is_empty() {
        return Err(());
    }

    let val = s.parse::<i64>().map_err(|_| ())?;
    // power_now is in microwatts; when discharging the value is negative.
    // Only show power when in use (i.e., val < 0). For display take abs().
    if val >= 0 {
        return Ok(None);
    }

    let watts = (val.abs()) as f64 / 1_000_000.0;
    Ok(Some(format!("{:.1}W", watts)))
}

fn read_capacity(path: &PathBuf) -> Result<String, ()> {
    let s = fs::read_to_string(path).map_err(|_| ())?;
    let s = s.trim();
    if s.is_empty() {
        return Err(());
    }

    let val = s.parse::<i64>().map_err(|_| ())?;
    Ok(format!("{}%", val))
}

fn main() {
    // Pin to 0 CPU (E-Core)
    core_affinity::set_for_current(CoreId { id: 0 });

    let args = Args::parse();

    // Determine mode: default to power if neither flag is provided
    if args.power && args.capacity {
        eprintln!("Only one of --power or --capacity may be provided");
        std::process::exit(2);
    }

    let mode_power = if !args.power && !args.capacity {
        true
    } else {
        args.power
    };

    let default_dir =
        "/sys/devices/platform/soc/290400000.smc/macsmc-power/power_supply/macsmc-battery";
    let dir = args
        .path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default_dir));

    if mode_power {
        let path = dir.join("power_now");

        if args.once {
            match read_power(&path) {
                Ok(Some(text)) => output(text, "power"),
                Ok(None) => output(String::from(" "), "power-idle"),
                Err(_) => output("ERR".to_string(), "power-err"),
            }
            return;
        }

        loop {
            match read_power(&path) {
                Ok(Some(text)) => output(text, "power"),
                Ok(None) => output(String::from(" "), "power-idle"),
                Err(_) => output("ERR".to_string(), "power-err"),
            }

            thread::sleep(Duration::from_secs(args.interval));
        }
    } else {
        let path = dir.join("capacity");

        if args.once {
            match read_capacity(&path) {
                Ok(text) => output(text, "battery"),
                Err(_) => output("ERR".to_string(), "battery-err"),
            }
            return;
        }

        loop {
            match read_capacity(&path) {
                Ok(text) => output(text, "battery"),
                Err(_) => output("ERR".to_string(), "battery-err"),
            }

            thread::sleep(Duration::from_secs(args.interval));
        }
    }
}
