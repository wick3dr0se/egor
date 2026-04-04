use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
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
        #[arg(long)]
        release: bool,
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

#[derive(Deserialize)]
struct Manifest {
    package: Package,
    #[serde(default, rename = "bin")]
    bins: Vec<Bin>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

#[derive(Deserialize)]
struct Bin {
    name: Option<String>,
    path: Option<String>,
}

fn read_bin_name(demo_dir: &Path) -> String {
    let manifest_path = demo_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", manifest_path.display()));

    let manifest: Manifest = toml::from_str(&manifest_str)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", manifest_path.display()));

    manifest
        .bins
        .iter()
        .find(|bin| bin.path.as_deref() == Some("src/main.rs"))
        .and_then(|bin| bin.name.clone())
        .unwrap_or(manifest.package.name)
}

impl Cmd {
    fn run(&self) {
        let Cmd::Run {
            demo,
            release,
            features,
            ..
        } = self;

        let mut features = features.clone();
        let demo_dir = PathBuf::from("demos").join(demo);
        let index_path = demo_dir.join("index.html");
        let bin_name = read_bin_name(&demo_dir);

        let mut cmd = match self {
            Cmd::Run { wasm: true, .. } => {
                fs::write(
                    &index_path,
                    format!(
                        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><link data-trunk rel="rust" data-bin="{bin_name}"/><style>body{{margin:0}}canvas{{display:block;width:100vw;height:100vh}}</style></head></html>"#
                    ),
                )
                .expect("failed to create index.html");

                let path = index_path.clone();
                ctrlc::set_handler(move || {
                    let _ = fs::remove_file(&path);
                    process::exit(1);
                })
                .expect("failed to set Ctrl-C handler");

                let mut cmd = Command::new("trunk");
                cmd.arg("serve");
                cmd
            }
            Cmd::Run {
                android: Some(device),
                ..
            } => {
                let mut cmd = Command::new("x");
                cmd.args(["run", "--arch", "arm64", "--device", device]);
                cmd
            }
            Cmd::Run {
                hot_reload: true, ..
            } => {
                features.push("hot_reload".to_string());

                let mut cmd = Command::new("dx");
                cmd.args(["serve", "--hot-patch", "--bin", &bin_name]);
                cmd
            }
            _ => {
                let mut cmd = Command::new("cargo");
                cmd.args(["run", "--bin", &bin_name]);
                cmd
            }
        };

        if *release {
            cmd.arg("--release");
        }

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
