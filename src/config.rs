use std::{env, fs};
use std::io::Read;
use std::path::{Path, PathBuf};

use json;
use json_comments::StripComments;
use whoami;

// Gets the path the current debug configuration is located at
pub fn config_path() -> Option<String> {
    // If the CUR_FILE environment variable exists, use that
    // Otherwise, use the working directory
    let mut cur_file : String = env::current_dir().unwrap().to_string_lossy().to_string();
    if env::var("CUR_FILE").is_ok() {
        cur_file = env::var("CUR_FILE").unwrap();
    }
    let mut src = PathBuf::from(cur_file);
    while !src.is_dir() {
        src.pop();
    }
    // Look up through the heirarchy to find the config file
    loop {
        info!("Checking for file {}", src.join(".kak-dap.json").to_str().unwrap());
        let exists = std::path::Path::new(src.join(".kak-dap.json").to_str().unwrap()).exists();
        if exists {
            let root_dir = src.join(".kak-dap.json").to_str().unwrap().to_string();
            info!("Found config at {}", root_dir);
            return Some(root_dir);
        }
        if !src.pop() {
            break;
        }
    }
    return None
}

// Gets the JSON configuration from the configuration file.
pub fn get_config(config_path : &String) -> Option<json::JsonValue> {
    let data = fs::read_to_string(config_path).expect("Couldn't read configuration file");
    // Remove comments before processing
    let mut stripped = String::new();
    StripComments::new(data.as_bytes()).read_to_string(&mut stripped).unwrap();
    // Replace expandables
    let config_dir = Path::new(config_path).parent().unwrap().to_string_lossy().to_string();
    stripped = str::replace(&stripped, "${HOME}", env!("HOME"));
    stripped = str::replace(&stripped, "${USER}", &whoami::username());
    stripped = str::replace(&stripped, "${CUR_DIR}", &config_dir);
    stripped = str::replace(&stripped, "$$", "$");
    // Attempt to retrieve JSON configuration
    let parsed = json::parse(&stripped);
    if parsed.is_err() {
        return None
    }
    return Some(parsed.unwrap());
}
