use lib::kv::Kv;
use lib::types::Response;

use std::process::Command;
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = false)]
    record: bool,

    #[arg(long, default_value_t = false)]
    stop: bool,

    #[arg(long, default_value_t = false)]
    listen: bool,
}

fn output(secs: Option<i64>) {
    let mut text = String::new();
    let mut class = "not-recording";

    if let Some(secs) = secs {
        text = format!("   {:0>2}:{:0>2}", secs / 60, secs - (secs / 60));
        class = "recording";
    }

    println!(
        "{}",
        serde_json::to_string(&Response {
            class,
            text: text.as_str()
        })
        .unwrap()
    );
}

fn main() {
    let db = Kv::new();

    let args = Args::parse();

    if args.stop {
        if let Ok(pid) = db.get_i64("recording:pid") {
            db.del("recording:pid");
            Command::new("kill")
                .arg(pid.to_string())
                .output()
                .expect("failed to execute process");
            db.put_bool("recording", false);
        }

        return;
    }

    if args.listen {
        loop {
            if let Ok(recording) = db.get_bool("recording") {
                if recording {
                    let since_the_epoch = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");

                    let start = db.get_i64("recording:start").unwrap();
                    let duration = since_the_epoch.as_secs() as i64 - start;

                    output(Some(duration));
                } else {
                    output(None);
                }

                std::thread::sleep(std::time::Duration::from_millis(350));
            }
        }
    }

    let output = Command::new("slurp")
        .output()
        .expect("failed to execute process");

    let dimensions = String::from_utf8_lossy(&output.stdout);
    if dimensions.trim().len() < 1 {
        return;
    }

    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    db.put_bool("recording", true);
    db.put_i64("recording:start", since_the_epoch.as_secs() as i64);
    db.put_i64("recording:pid", since_the_epoch.as_secs() as i64);

    let mut child = Command::new("wf-recorder")
        .arg("-g")
        .arg(format!("{}", dimensions))
        .arg("-x")
        .arg("yuv420p")
        .arg("-f")
        .arg(format!(
            "{}/{}.mp4",
            xdg_user::videos().unwrap().unwrap().to_string_lossy(),
            since_the_epoch.as_secs()
        ))
        .spawn()
        .expect("failed to execute process");

    let pid = child.id() as i64;
    db.put_i64("recording:pid", pid);

    child.wait().expect("failed to wait on child");

    db.put_bool("recording", false);
}
