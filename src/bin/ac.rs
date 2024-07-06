use lib::types::Response;

use gree::{sync_client::*, GreeClientConfig, *};
use std::str::FromStr;
use std::{net::IpAddr, result::Result};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    ip: String,

    #[arg(long)]
    id: String,

    #[arg(long)]
    key: String,

    #[arg(long, default_value_t = false)]
    toggle: bool,

    #[arg(long, default_value_t = false)]
    listen: bool,
}

fn output(running: bool) {
    let text = "";
    let mut class = "ac-off";

    if running {
        class = "ac-on";
    }

    println!(
        "{}",
        serde_json::to_string(&Response { class, text: text }).unwrap()
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let cc = GreeClientConfig::default();
    let c = GreeClient::new(cc)?;

    let ip = IpAddr::from_str(args.ip.as_str())?;
    let id = args.id.as_str();
    let key = args.key.as_str();

    if args.toggle {
        let r = c.getvars(ip, id, key, &vec![vars::POW])?;

        if let Some(on) = r.dat[0].as_u64() {
            let alt = if on == 1 { 0 } else { 1 };
            c.setvars(ip, id, key, &vec![vars::POW], &[Value::Number(alt.into())])?;
        }

        return Ok(());
    }

    if args.listen {
        loop {
            if let Ok(r) = c.getvars(ip, id, key, &vec![vars::POW]) {
                output(r.dat[0].as_u64().unwrap() == 1);
            } else {
                output(false);
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    Ok(())
}
