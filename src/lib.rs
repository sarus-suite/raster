use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use derivative::Derivative;

pub type SarusResult<T> = std::result::Result<T, SarusError>;

#[derive(Debug, Clone, Deserialize)]
pub struct SarusError {
    code: u64,
    msg: String,
}

impl std::fmt::Display for SarusError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error {:03}: {}", self.code, self.msg)
    }
}

impl Error for SarusError {}

#[allow(dead_code)]
#[derive(Derivative, Serialize, Deserialize)]
pub struct EDF {
   #[serde(default = "get_default_annotations")]
   annotations: HashMap<String,String>,
   base_environment: Option<BaseEnvironment>,
   #[serde(default = "get_default_devices")]
   devices: Vec<String>,
   #[serde(default = "get_default_entrypoint")]
   entrypoint: bool,
   #[serde(default = "get_default_env")]
   env: HashMap<String,String>,
   image: Option<String>,
   #[serde(default = "get_default_mounts")]
   mounts: Vec<String>,
   #[serde(default = "get_default_workdir")]
   workdir: String,
   #[serde(default = "get_default_writable")]
   writable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BaseEnvironment {
    TypeString(String),
    TypeVec(Vec<String>),
}

fn get_default_annotations() -> HashMap<String,String> {
    return HashMap::from([]);
}

fn get_default_devices() -> Vec<String> {
    return vec![];
}

fn get_default_entrypoint() -> bool {
    return false;
}

fn get_default_env() -> HashMap<String,String> {
    return HashMap::from([]);
}

fn get_default_writable() -> bool {
    return true;
}

fn get_default_mounts() -> Vec<String> {
    return vec![];
}

fn get_default_workdir() -> String {
    return String::from("");
}

fn load(file_path: &str) -> Result<String, Box<dyn Error>> {

  // SD-67022 - prevent reading wrong file
  let fp = Path::new(file_path);

  let fname = match fp.file_name().and_then(OsStr::to_str) {
      Some(name) => name,
      None => return Err(format!("Cannot extract file name from {file_path}").into()),
  };
  
  let ext = match fp.extension().and_then(OsStr::to_str) {
      Some(x) => x,
      None => return Err(format!("Cannot extract file extension from {file_path}").into()),
  };

  if ext != "toml" {
      return Err(format!("File name {fname} doesn't end with .toml").into());
  }

  if ! fp.exists() {
      return Err(format!("File {file_path} not found").into());
  }

  let outstr = std::fs::read_to_string(file_path)?;

  Ok(outstr)
}

pub fn validate(path: String) -> SarusResult<()> {

    let path_str = path.as_str();

    // Embedding schema file
    let schema_content = include_str!("schema/edf.json");

    let schema: serde_json::Value = match serde_json::from_str(&schema_content) {
        Ok(c) => c,
        Err(_) => {
            return Err(
                SarusError {
                    code: 0,
                    msg: String::from("Failed to parse schema file"),
                });
        }
    };

    let validator = match jsonschema::options().build(&schema) {
        Ok(v) => v,
        Err(error) => {
            return Err(
                SarusError {
                    code: 1,
                    msg: String::from(format!("Schema is invalid.\n{error}")),
                });
        }
    };
    
    let toml_content = match load(path_str) {
        Ok(c) => c,
        Err(e) => {
            return Err(
                SarusError {
                    code: 2,
                    msg: String::from(format!("{}", e)),
                });
        },
    };

    let toml_value = match toml::from_str(&toml_content) {
        Ok(v) => v,
        Err(e) => {
            return Err(
                SarusError {
                    code: 3,
                    msg: String::from(format!("{}", e)),
                });
        },
    };

    let mut has_errors = false;
    let mut errors = validator.iter_errors(&toml_value);
    let mut emsg = String::from("");

    if let Some(first) = errors.next() {
        has_errors = true;
        emsg = format!("Errors:\n1. {first}");
        for (i, error) in errors.enumerate() {
            emsg = String::from(format!("{emsg}\n{}. {}", (i + 2), error));
        }
    }

    if has_errors {
        return Err(
            SarusError {
                code: 4,
                msg: String::from(format!("{}", emsg)),
            });
    } else {
        return Ok(());
    }
}

pub fn render(path: String) -> SarusResult<EDF> {

    validate(path.clone())?;

    let path_str = path.as_str();
    
    let toml_content = match load(path_str) {
        Ok(c) => c,
        Err(e) => {
            return Err(
                SarusError {
                    code: 2,
                    msg: String::from(format!("{}", e)),
                });
        },
    };

    
    let toml_value = match toml::from_str(toml_content.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return Err(
                SarusError {
                    code: 3,
                    msg: String::from(format!("{}", e)),
                });
        },
    };

    let e: EDF = toml_value;

    Ok(e)
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_toml() {
        // Loop through all toml files containing "good" in their name
        let good_filepaths = std::fs::read_dir("src/toml").unwrap();
        for fr in good_filepaths {
            let fpath = fr.unwrap().path();
            let fname = fpath.file_name().unwrap().to_os_string().into_string().unwrap();
            if fname.contains("good") {
                let fstr = fpath.into_os_string().into_string().unwrap();
                let r = render(fstr);
                assert!(r.is_ok());
            }
        }
    }

    #[test]
    fn file_not_found() {
        let result = render(String::from("src/toml/not_found.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn not_a_toml_file() {
        let result = render(String::from("src/toml/test.txt"));
        assert!(result.is_err());
    }
}
