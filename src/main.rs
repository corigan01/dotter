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
    Remove {
        // Configuration to remove
        config_name: String,
    },
    Install {
        // Optional configuration tag to install from
        config_name: Option<String>,
    },
    List,
}

const DEFAULT_CONFIG_NAME: &str = "default";
const DEFAULT_CONFIG_CONTENTS: &str = r#"
[doot]
name = "example"
authors = ["your name"]
version = "0.0.1"

[config]
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

    let full_config_name = format!("./{config_file}/{config_file}.toml");
    make_new_doot(&full_config_name)?;

    Ok(())
}

fn remove(config_file: String) -> anyhow::Result<()> {
    println!("Removing Config: {config_file}");

    let full_config_name = format!("./{config_file}");
    let mut files_to_remove: Vec<String> = Vec::new();

    for file in Path::new(&full_config_name).read_dir()? {
        files_to_remove.push(file?.file_name().into_string().unwrap());
    }

    println!("Removing: {files_to_remove:?}");

    loop {
        print!("Are you sure you want to remove [y, N]: ");
        std::io::stdout().flush()?;

        let mut user_line = String::new();
        std::io::stdin().read_line(&mut user_line)?;
        let user_line = user_line.trim();

        match user_line.to_lowercase().as_str() {
            "y" => break,
            "n" | "" => {
                println!("Canceled!");
                return Ok(());
            }

            _ => println!("Please use 'y', or 'n'!"),
        }
    }

    println!("Deleting Files...");
    std::fs::remove_dir_all(full_config_name)?;
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
            new(config_name)?;
        }
        Command::Remove { config_name } => remove(config_name)?,
        Command::Install { config_name } => {
            let config_name = config_name.unwrap_or(DEFAULT_CONFIG_NAME.into());
            install(config_name);
        }
        Command::List => {
            list();
        }
    }

    println!("Done");
    Ok(())
}
