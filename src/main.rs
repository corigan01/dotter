use std::{io::Write, path::Path};

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CommandLine {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    New {
        // Optional configuration tag
        config_name: Option<String>,
    },
    Install {
        // Optional configuration tag to install from
        config_name: Option<String>,
    },
    List,
}

const DEFAULT_CONFIG_NAME: &str = "default";
const DOOT_NAME: &str = "Doot.toml";
const DEFAULT_CONFIG_CONTENTS: &str = r#"
[doot]
name = "defualt"
authors = ["your name"]
version = "0.0.1"

[my_config]
location = "~/.config/my_config/config.toml"
symlink = false
topic = "My example config for example program!"
ask = true
"#;

fn make_new_doot(file_name: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(
        Path::new(file_name)
            .parent()
            .context("Could not get parent")?,
    )?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_name)?;

    file.write_all(DEFAULT_CONFIG_CONTENTS.as_bytes())?;
    Ok(())
}

fn new(config_file: String) -> anyhow::Result<()> {
    println!("New config file {config_file}");
    if config_file.contains(".") {
        bail!("Config name '{config_file}' should not include a file type, as dotter makes a directory!");
    }

    let full_config_name = format!("{config_file}/{DOOT_NAME}");
    make_new_doot(&full_config_name)?;

    Ok(())
}

fn install(config_file: String) {
    println!("Installing config file {config_file}");
}

fn list() {
    println!("Listing configs");
}

fn main() -> anyhow::Result<()> {
    let command = CommandLine::parse().command;

    match command {
        Command::New { config_name } => {
            let config_name = config_name.unwrap_or(DEFAULT_CONFIG_NAME.into());
            new(config_name)
        }
        Command::Install { config_name } => {
            let config_name = config_name.unwrap_or(DEFAULT_CONFIG_NAME.into());
            install(config_name);
            Ok(())
        }
        Command::List => {
            list();
            Ok(())
        }
    }
}
