use std::error::Error;

use async_process::Child;
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
