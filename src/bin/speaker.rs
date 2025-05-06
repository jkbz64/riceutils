use lib::pulse::is_output_muted;
use lib::types::Response;
use lib::utils::process_signals;

use core_affinity::CoreId;

use std::error::Error;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

fn output(muted: bool) {
    let mut text = "";
    let mut class = "not-muted";

    if muted {
        text = "󰝟";
        class = "muted";
    }

    println!(
        "{}",
        serde_json::to_string(&Response { class, text }).unwrap()
    );
}

async fn process_lines(child: &mut Child) -> Result<(), Box<dyn Error>> {
    let stdout = child
        .stdout
        .take()
        .expect("Child process stdout is not available");
    let mut lines = BufReader::new(stdout).lines();

    while let Some(line) = lines.next_line().await? {
        if !line.contains("Event 'change' on sink") {
            continue;
        }

        output(is_output_muted().await?);
    }

    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    output(is_output_muted().await?);

    let mut child = Command::new("pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .spawn()?;

    tokio::select! {
        result = process_signals() => result,
        result = process_lines(&mut child) => result,
    }?;

    // Ensure the child process is killed
    let _ = child.kill().await;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .worker_threads(1)
        .on_thread_start(|| {
            core_affinity::set_for_current(CoreId { id: 0 });
        })
        .build()
        .unwrap()
        .block_on(run())?)
}
