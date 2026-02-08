use clap::{Parser, Subcommand};
use std::{
    fs,
    path::Path,
    process::{self, Command},
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Run {
        demo: String,
        #[arg(long, num_args = 0..)]
        features: Vec<String>,
        #[arg(long)]
        wasm: bool,
        #[arg(long)]
        android: Option<String>,
        #[arg(long)]
        hot_reload: bool,
    },
}
impl Cmd {
    fn run(&self) {
        let Cmd::Run { demo, features, .. } = self;
        let mut features = features.clone();
        let demo_dir = format!("demos/{}", demo);
        let index_path = Path::new(&demo_dir).join("index.html");

        let (cmd, args): (&str, &[&str]) = match self {
            Cmd::Run { wasm: true, .. } => {
                fs::write(&index_path, format!(
                    r#"<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><link data-trunk rel="rust" data-bin="{demo}"/><style>body{{margin:0}}canvas{{display:block;width:100vw;height:100vh}}</style></head></html>"#
                )).expect("failed to create index.html");

                let path = index_path.clone();
                ctrlc::set_handler(move || {
                    let _ = fs::remove_file(&path);
                    process::exit(1);
                })
                .expect("failed to set Ctrl-C handler");
                ("trunk", &["serve"])
            }
            Cmd::Run {
                android: Some(device),
                ..
            } => ("x", &["run", "--arch", "arm64", "--device", device]),
            Cmd::Run {
                hot_reload: true, ..
            } => {
                features.push("hot_reload".to_string());
                ("dx", &["serve", "--hot-patch"])
            }
            _ => ("cargo", &["run"]),
        };

        let mut cmd = Command::new(cmd);
        cmd.args(args);

        if !features.is_empty() {
            let prefixed: Vec<_> = features.iter().map(|f| format!("egor/{}", f)).collect();
            cmd.arg("--features").arg(prefixed.join(","));
        }

        cmd.current_dir(&demo_dir);

        println!("> {:?}", cmd);
        let status = cmd.status().expect("failed to spawn command");

        if let Cmd::Run { wasm: true, .. } = self {
            let _ = fs::remove_file(&index_path);
        }
        if !status.success() {
            process::exit(status.code().unwrap_or(1));
        }
    }
}
fn main() {
    Cli::parse().cmd.run();
}
