use std::fs;
use std::io::Write;
use std::{fs::File, process::exit};

use serde_derive::{Deserialize, Serialize};
use tui::style::Color;

#[derive(Deserialize, Serialize, Debug)]
pub struct RawConfig {
    pub token: String,
    pub color: String,
}

pub struct Config {
    pub token: String,
    pub color: Color,
}

impl Config {
    pub fn new(config: RawConfig) -> Config {
        let colors = config.color.replace(' ', "");
        let colors: Vec<&str> = colors.split(',').collect();
        let color = Color::Rgb(
            colors[0].parse().unwrap(), 
            colors[1].parse().unwrap(), 
            colors[2].parse().unwrap(), 
            );

        Config {
            token: config.token,
            color,
        }
    }
}

use std::env;


pub fn get_config() -> Config {
    let filename = "Config.toml";
    let mut args = env::args();
    args.next();

    if let Some(argument) = args.next() {
        let mut file = File::create(filename).unwrap();
        file.write_all(format!("token = '{}'\n
                                color = '210, 39, 48'", argument)
                       .as_bytes())
                       .unwrap();
    }

    let contents = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Unable to read file: {}", filename);
            exit(1);
        }
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(config) => Config::new(config),
        Err(_) => {
            eprintln!("Unable to load data from '{}'", filename);
            eprintln!("Make sure to put your API token inside the config.toml");
            exit(1);
        }
    };

    config
}
