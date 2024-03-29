use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::{
    fs::{self ,OpenOptions},
    io::{Read, Write},
    path::Path,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CommandLine {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Creates a new Dotter Directory
    New {
        /// Optional configuration tag
        config_name: Option<String>,
    },
    /// Removes a Dotter Directory
    Remove {
        /// Configuration to remove
        config_name: String,
    },
    /// Installs a Dotter Directory
    Install {
        /// Optional configuration tag to install from
        config_name: Option<String>,
    },
    /// Lists all Dotter Directories
    List,
}

const DEFAULT_CONFIG_NAME: &str = "default";
const DEFAULT_CONFIG_CONTENTS: &str = r#"[doot]
name = "example"
authors = ["your name"]
version = "0.0.1"
topic = "My example config for example program!"

[config]
target = ["~/.config/my_config/config.txt"]
source = ["config.txt"]
ask = true
debug = true
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

fn user_boolean(question: &str, yes_no_bias: bool) -> anyhow::Result<bool> {
    loop {
        if yes_no_bias {
            print!("{question} [Y, n]: ");
        } else {
            print!("{question} [y, N]: ");
        }
        std::io::stdout().flush()?;

        let mut user_line = String::new();
        std::io::stdin().read_line(&mut user_line)?;
        let user_line = user_line.trim();

        match user_line.to_lowercase().as_str() {
            "y" => break Ok(true),
            "n" => break Ok(false),
            "" => break Ok(yes_no_bias),

            _ => println!("Please use 'y', or 'n'!"),
        }
    }
}

fn remove(config_file: String) -> anyhow::Result<()> {
    println!("Removing Config: {config_file}");

    let full_config_name = format!("./{config_file}");
    let mut files_to_remove: Vec<String> = Vec::new();

    for file in Path::new(&full_config_name).read_dir()? {
        files_to_remove.push(file?.file_name().into_string().unwrap());
    }

    println!("Removing: {files_to_remove:?}");

    let user_bool = user_boolean("Are you sure you want to remove these files", false)?;
    if !user_bool {
        println!("Canceled");
        return Ok(());
    }

    println!("Deleting Files...");
    std::fs::remove_dir_all(full_config_name)?;
    Ok(())
}

#[derive(Deserialize, Debug)]
struct DootConfig {
    doot: DootItems,
    config: Config,
}

#[derive(Deserialize, Debug)]
struct DootItems {
    name: String,
    topic: String,
    authors: Vec<String>,
    version: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    target: Vec<String>,
    source: Vec<String>,
    ask: Option<bool>,
    debug: Option<bool>,
}

fn install_config(config: DootConfig, parent_dir: String) -> anyhow::Result<()> {
    let Config {
        target,
        source,
        ask,
        debug,
    } = config.config;
    let ask = ask.unwrap_or(true);
    let debug = debug.unwrap_or(false);

    let DootItems {
        name,
        topic,
        authors,
        version,
    } = config.doot;
    println!(
        "Package:\n\tName:     {name}\n\tTopic:    {topic}\n\tAuthors:  {authors:?}\n\tVersion:  {version}"
    );

    let should_install = if ask {
        user_boolean("Are you sure you want to install?", true)?
    } else {
        true
    };

    if !should_install {
        println!("Skipped...");
        return Ok(());
    }

    if source.len() != target.len() {
        bail!(
            "There are {} sources, but found {} targets, source and targets must match!",
            source.len(),
            target.len()
        );
    }

    if source.len() == 0 {
        bail!("There must be at least one 'source' and 'target' pair!");
    }

    let user_home = std::env::var_os("HOME")
        .context("Could not find home dir, please set HOME enviroment var!")?
        .to_os_string()
        .into_string()
        .unwrap();

    for (source, target) in source.iter().zip(target.iter()) {
        let parent_dir = Path::new(&parent_dir);
        let source = parent_dir
            .join(Path::new(&source))
            .canonicalize()
            .context("Could not join source path")?
            .into_os_string()
            .into_string()
            .unwrap();

        let target = parent_dir
            .join(Path::new(&target.as_str().replace("~", &user_home)))
            .into_os_string()
            .into_string()
            .unwrap();

        if !debug {
            let mut config_source = OpenOptions::new()
                .read(true)
                .open(&source)
                .context(format!("Config's source '{source}' was not found!"))?;
            let mut config_dest = OpenOptions::new().write(true).create(true).open(&target)?;

            let mut reading_string = String::new();
            config_source.read_to_string(&mut reading_string)?;
            config_dest.write_all(reading_string.as_bytes())?;
        }
        if debug {
            println!("DEBUG: {source} -> {target}");
        } else {
            println!("COPY: {source} -> {target}");
        }
    }

    Ok(())
}

fn install(config_file: String) -> anyhow::Result<()> {
    if config_file.contains(".") {
        bail!("Invalid name '{config_file}'. Please use a doot directory name!");
    }

    let mut doots = Vec::new();
    for file in Path::new(&config_file).read_dir()? {
        let file = file?;
        if file.file_type()?.is_dir() {
            continue;
        }

        if !file
            .file_name()
            .as_os_str()
            .to_str()
            .unwrap()
            .ends_with(".toml")
        {
            continue;
        }

        doots.push(file.path().into_os_string().into_string().unwrap());
    }

    println!("Found toml files: {doots:?}");
    for doot_file in doots {
        let mut read_string = String::new();
        let mut file = OpenOptions::new().read(true).open(&doot_file)?;
        file.read_to_string(&mut read_string)?;

        let config = match toml::from_str::<DootConfig>(&read_string) {
            Ok(ok) => ok,
            Err(err) => {
                println!("Not valid doot file: '{doot_file}: Skipping... \n{err}");
                continue;
            }
        };
        //println!("Config: {config:#?}");
        let current_dir = std::env::current_dir()?
            .into_os_string()
            .into_string()
            .unwrap();
        install_config(config, format!("{current_dir}/{config_file}"))?;
    }
    Ok(())
}

fn list() -> anyhow::Result<()> {
    println!("Listing configs");
    let paths = fs::read_dir("./")?;
    for path in paths {
        let entry = path?;
        if entry.file_type()?.is_dir() {
            
            for entry in fs::read_dir(entry.path())? {
                let entry = entry?;
                let file_name = entry.file_name();
        
                if let Some(name) = file_name.to_str() {
                    if name.ends_with(".toml") {
                        println!("Found TOML file: {}", entry.path().display());
                    }
                }
            }
        }
    }
    println!("if you want to install a config type 'dotter install <config_name>'");
    // todo!()
    Ok(())
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
            install(config_name)?;
        }
        Command::List => {
            list()?;
        }
    }

    println!("Done");
    Ok(())
}
