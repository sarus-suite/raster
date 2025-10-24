use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "/etc/sarus/edf.conf";

#[derive(Serialize, Deserialize, Clone)]
pub struct RawConfig {
    edf_system_search_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "get_default_edf_system_search_path")]
    pub edf_system_search_path: String,
}

fn get_default_edf_system_search_path() -> String {
    return String::from("/etc/edf");
}

impl From<RawConfig> for Config {
    fn from(r: RawConfig) -> Self {
        Config {
            edf_system_search_path: match r.edf_system_search_path {
                Some(s) => s,
                None => get_default_edf_system_search_path(),
            },
        }
    }
}

fn load_raw_config(filepath: String) -> RawConfig {
    let path_str = filepath.as_str();

    let empty = RawConfig {
        edf_system_search_path: None,
    };

    let toml_content = match std::fs::read_to_string(path_str) {
        Ok(c) => c,
        Err(_) => {
            return empty;
        }
    };

    let toml_value = match toml::from_str(&toml_content) {
        Ok(v) => v,
        Err(_) => {
            return empty;
        }
    };

    let r: RawConfig = toml_value;
    r
}

pub fn load_config() -> Config {
    let config_file_path = String::from(CONFIG_FILE);
    let r = load_raw_config(config_file_path);
    let c = Config::from(r);
    c
}
