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
        let color: Color = match config.color.parse::<usize>() {
            Ok(n) if n <= 11 => Self::get_color(n),
            _ => Self::get_color(6),
        };
        Config {
            token: config.token,
            color,
        }
    }
    pub fn get_color(n: usize) -> Color {
        let colors = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
        ];

        let color: Color = colors[n];
        color
    }
}

pub fn match_color(color: Color) -> String {
    match color {
        Color::Red => "0".to_string(),
        Color::Green => "1".to_owned(),
        Color::Yellow => "2".to_owned(),
        Color::Blue => "3".to_owned(),
        Color::Magenta => "4".to_owned(),
        Color::Cyan => "5".to_owned(),
        Color::LightGreen => "7".to_owned(),
        Color::LightYellow => "8".to_owned(),
        Color::LightBlue => "9".to_owned(),
        Color::LightMagenta => "10".to_owned(),
        Color::LightCyan => "11".to_owned(),
        _ => "6".to_owned(),
    }
}

use std::env;

pub fn get_config() -> Config {
    let filename = "Config.toml";
    let mut args = env::args();
    args.next();

    match args.next() {
        Some(argument) => {
            let mut file = File::create(filename).unwrap();
            file.write_all(format!("token = '{}'\ncolor = '6'", argument).as_bytes())
                .unwrap();
        }
        None => {}
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
