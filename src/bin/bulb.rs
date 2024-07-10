use lib::types::Response;
use yeelight::{Bulb, Properties, Property};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    ip: String,

    #[arg(long, default_value_t = false)]
    toggle: bool,

    #[arg(long, default_value_t = false)]
    toggle_bg: bool,

    #[arg(long, default_value_t = false)]
    listen: bool,
}

fn output(main: bool, bg: bool) {
    let mut text = "󰹏";
    let mut class = "bulb-off";

    if bg {
        text = "󱩐";
        class = "bulb-bg-on";
    }

    if main {
        text = "";
        class = "bulb-on";
    }

    println!(
        "{}",
        serde_json::to_string(&Response { class, text: text }).unwrap()
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let ip = args.ip;

    if args.toggle {
        let mut bulb = Bulb::connect(&ip, 55443)
            .await
            .expect("failed to connect to bulb");

        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), bulb.toggle()).await?;
    }

    if args.toggle_bg {
        let mut bulb = Bulb::connect(&ip, 55443)
            .await
            .expect("failed to connect to bulb");

        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), bulb.bg_toggle()).await?;
    }

    if args.listen {
        loop {
            let result = {
                let mut bulb = Bulb::connect(&ip, 55443)
                    .await
                    .expect("failed to connect to bulb");

                tokio::time::timeout(
                    std::time::Duration::from_secs(1),
                    bulb.get_prop(&Properties(vec![Property::Power, Property::BgPower])),
                )
                .await
            };

            if let Ok(result) = result {
                if let Ok(properties) = result {
                    if let Some(properties) = properties {
                        output(properties[0] == "on", properties[1] == "on");
                    } else {
                        output(false, false);
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    Ok(())
}
