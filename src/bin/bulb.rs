use core_affinity::CoreId;

use lib::types::Response;
use yeelight::{Bulb, Properties, Property};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    ip: String,

    #[arg(long, default_value_t = 55443)]
    port: u16,

    #[arg(long, default_value_t = false)]
    toggle: bool,

    #[arg(long, default_value_t = false)]
    toggle_bg: bool,

    #[arg(long)]
    brightness: Option<u8>,

    #[arg(long)]
    color: Option<u16>,

    #[arg(long)]
    bg_brightness: Option<u8>,

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

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.listen {
        loop {
            if let Ok(mut bulb) = Bulb::connect(&args.ip, args.port).await {
                if let Ok(Ok(properties)) = tokio::time::timeout(
                    std::time::Duration::from_secs(1),
                    bulb.get_prop(&Properties(vec![Property::Power, Property::BgPower])),
                )
                .await
                {
                    if let Some(properties) = properties {
                        output(properties[0] == "on", properties[1] == "on");
                    } else {
                        output(false, false);
                    }
                }
            } else {
                output(false, false);
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    } else {
        let mut bulb = Bulb::connect(&args.ip, args.port)
            .await
            .expect("failed to connect to bulb");

        if args.toggle {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), bulb.toggle()).await?;
        }

        if args.toggle_bg {
            let _ =
                tokio::time::timeout(std::time::Duration::from_secs(5), bulb.bg_toggle()).await?;
        }

        if let Some(brightness) = args.brightness {
            bulb.set_bright(
                brightness,
                yeelight::Effect::Sudden,
                std::time::Duration::from_secs(1),
            )
            .await?;
        }

        if let Some(color) = args.color {
            bulb.bg_set_hsv(
                color,
                100,
                yeelight::Effect::Sudden,
                std::time::Duration::from_secs(1),
            )
            .await?;
        }

        if let Some(bg_brightness) = args.bg_brightness {
            bulb.bg_set_bright(
                bg_brightness,
                yeelight::Effect::Sudden,
                std::time::Duration::from_secs(1),
            )
            .await?;
        }
    }

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
