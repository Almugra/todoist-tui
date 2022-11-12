use std::fs::{self, File};
use std::io::{stdin, stdout, Read, Write};

pub struct Config {
    pub token: String,
}

impl Config {
    pub fn get_token() -> Config {
        let mut file = match File::open("config.txt") {
            Ok(s) => s,
            Err(_) => get_config_file(),
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(s) => s,
            Err(error) => panic!("Problem reading the file: {:?}", error),
        };
        Config {
            token: contents.trim().to_string(),
        }
    }
}

pub fn get_config_file() -> File {
    match File::create("config.txt") {
        Ok(f) => f,
        Err(e) => panic!("Problem creating the file: {}", e),
    };
    print!("Please enter your todois api token: ");
    let _ = stdout().flush();

    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");
    match fs::write("config.txt", input) {
        Ok(w) => w,
        Err(e) => panic!("Problem writing the file: {}", e),
    };
    let file = match File::open("config.txt") {
        Ok(f) => f,
        Err(e) => panic!("Problem opening the file: {}", e),
    };
    file
}
