use lib::types::Response;

use lib::pulse::is_output_muted;
use lib::utils::process_signals;

use std::error::Error;

use async_process::{ChildStdout, Command, Stdio};
use futures_lite::{future, io::BufReader, prelude::*};

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

async fn process_lines(stdout: ChildStdout) -> Result<(), Box<dyn Error>> {
    let mut lines = BufReader::new(stdout).lines();

    while let Some(Ok(line)) = lines.next().await {
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

    match child.stdout.take() {
        Some(stdout) => {
            let result = future::race(process_signals(&mut child), process_lines(stdout)).await;
            child.kill()?;
            return result;
        }
        _ => child.kill()?,
    }

    Ok(())
}

fn main() {
    future::block_on(run()).expect("Got unexpected error");
}
