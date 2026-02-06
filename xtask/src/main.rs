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
        #[arg(long, value_parser, num_args = 0..)]
        features: Vec<String>,
        #[arg(long)]
        hot_reload: bool,
        #[arg(long)]
        wasm: bool,
    },
}

impl Cmd {
    fn run(&self) {
        let Cmd::Run {
            demo,
            features,
            hot_reload,
            wasm,
        } = self;
        let dir = format!("demos/{}", demo);
        let index_path = Path::new(&dir).join("index.html");

        if *wasm {
            fs::write(
                &index_path,
                format!(
                    r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <link data-trunk rel="rust" data-bin="demo_egor_{demo}"/>
    <style>body{{margin:0}}canvas{{display:block;width:100vw;height:100vh}}</style>
  </head>
</html>"#
                ),
            )
            .expect("failed to create index.html");

            let index_path = index_path.clone();
            ctrlc::set_handler(move || {
                let _ = fs::remove_file(&index_path);
                process::exit(1);
            })
            .expect("failed to set Ctrl-C handler");
        }

        let (cmd, arg) = if *wasm {
            ("trunk", "serve")
        } else if *hot_reload {
            ("dx", "serve")
        } else {
            ("cargo", "run")
        };
        let mut cmd = Command::new(cmd);
        cmd.arg(arg).current_dir(&dir);

        if !features.is_empty() {
            let prefixed = features
                .iter()
                .map(|f| format!("egor/{}", f))
                .collect::<Vec<_>>();
            cmd.arg("--features").arg(prefixed.join(","));
        }

        println!("> {:?}", cmd);
        let status = cmd.status().expect("failed to spawn command");

        if *wasm {
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
