use std::error::Error;
use tokio::signal;

pub async fn process_signals() -> Result<(), Box<dyn Error>> {
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;

    tokio::select! {
        _ = sigint.recv() => {},
        _ = sigterm.recv() => {},
    }

    Ok(())
}
