use crate::types::Source;

use std::error::Error;

use async_process::{Child, Command};
use async_signal::{Signal, Signals};
use futures_lite::prelude::*;
use signal_hook::low_level;

pub async fn process_signals(child: &mut Child) -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new(&[Signal::Int, Signal::Term])?;

    while let Some(signal) = signals.next().await {
        child.kill()?;
        low_level::emulate_default_handler(signal.unwrap() as i32).unwrap();
    }

    Ok(())
}

pub async fn list_sinks() -> Result<Vec<Source>, Box<dyn Error>> {
    let output = Command::new("pactl")
        .arg("list")
        .arg("sinks")
        .output()
        .await?;
    let text = String::from_utf8_lossy(&output.stdout);

    // Parse pactl list sources into Source structs
    let mut sources = Vec::new();

    let mut name = "".to_string();

    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("Name: ") {
            name = line.split_whitespace().last().unwrap().to_string();
        }

        if line.starts_with("Mute: ") {
            sources.push(Source {
                name: name.clone(),
                mute: line.contains("Mute: yes"),
            });
        }
    }

    Ok(sources)
}

pub async fn list_sources() -> Result<Vec<Source>, Box<dyn Error>> {
    let output = Command::new("pactl")
        .arg("list")
        .arg("sources")
        .output()
        .await?;
    let text = String::from_utf8_lossy(&output.stdout);

    // Parse pactl list sources into Source structs
    let mut sources = Vec::new();

    let mut name = "".to_string();

    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("Name: ") {
            name = line.split_whitespace().last().unwrap().to_string();
        }

        if line.starts_with("Mute: ") {
            sources.push(Source {
                name: name.clone(),
                mute: line.contains("Mute: yes"),
            });
        }
    }

    Ok(sources)
}

pub async fn default_sink() -> Result<String, Box<dyn Error>> {
    let output = Command::new("pactl").arg("info").output().await?;
    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.starts_with("Default Sink:") {
            return Ok(line.split_whitespace().last().unwrap().to_string());
        }
    }

    Err("No default source found".into())
}

pub async fn default_source() -> Result<String, Box<dyn Error>> {
    let output = Command::new("pactl").arg("info").output().await?;
    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.starts_with("Default Source:") {
            return Ok(line.split_whitespace().last().unwrap().to_string());
        }
    }

    Err("No default source found".into())
}

// pamixer: is output muted
pub async fn is_output_muted() -> Result<bool, Box<dyn Error>> {
    let sink = default_sink().await?;
    let sinks = list_sinks().await?;

    for s in sinks {
        if s.name == sink {
            return Ok(s.mute);
        }
    }

    Ok(true)
}

pub async fn is_input_muted() -> Result<bool, Box<dyn Error>> {
    let source = default_source().await?;
    let sources = list_sources().await?;

    for s in sources {
        if s.name == source {
            return Ok(s.mute);
        }
    }

    Ok(true)
}
