use serde::{Deserialize, Serialize};
use std::fs;
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "Apps")]
    pub app: Vec<App>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct App {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Ports")]
    pub ports: Vec<u16>,
    #[serde(rename = "Targets")]
    pub targets: Vec<String>,
}

pub fn get_config(path: &str) -> Config {
    let data = fs::read_to_string(path).expect("unabled to read from file");
    let config: Config = serde_json::from_str(&data).unwrap();
    return config;
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_config_file() {
        let config = get_config("./config.json");
        println!("{:?}", config);
        assert!(true)
    }
}
