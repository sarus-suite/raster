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
#[derive(Derivative, Serialize, Deserialize, Clone)]
pub struct RawEDF {
   annotations: Option<HashMap<String,String>>,
   base_environment: Option<BaseEnvironment>,
   devices: Option<Vec<String>>,
   entrypoint: Option<bool>,
   env: Option<HashMap<String,String>>,
   image: Option<String>,
   mounts: Option<Vec<String>>,
   workdir: Option<String>,
   writable: Option<bool>,
}

#[allow(dead_code)]
#[derive(Derivative, Serialize, Deserialize, Clone)]
pub struct EDF {
   #[serde(default = "get_default_annotations")]
   annotations: HashMap<String,String>,
   #[serde(default = "get_default_devices")]
   devices: Vec<String>,
   #[serde(default = "get_default_entrypoint")]
   entrypoint: bool,
   #[serde(default = "get_default_env")]
   env: HashMap<String,String>,
   image: String,
   #[serde(default = "get_default_mounts")]
   mounts: Vec<String>,
   #[serde(default = "get_default_workdir")]
   workdir: String,
   #[serde(default = "get_default_writable")]
   writable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl TryFrom<RawEDF> for EDF {
    type Error = SarusError;

    fn try_from(r: RawEDF) -> SarusResult<Self> {
        Ok(
            EDF {
                annotations: match r.annotations {
                    Some(s) => s,
                    None => get_default_annotations(),
                },
                devices: match r.devices {
                    Some(s) => s,
                    None => get_default_devices(),
                },
                entrypoint: match r.entrypoint {
                    Some(s) => s,
                    None => get_default_entrypoint(),
                },
                env: match r.env {
                    Some(s) => s,
                    None => get_default_env(),
                },
                image: match r.image {
                    Some(s) => s,
                    None => return Err(
                        SarusError {
                        code: 7,
                        msg: String::from("missing image specification"),
                        }),
                },
                mounts: match r.mounts {
                    Some(s) => s,
                    None => get_default_mounts(),
                },
                workdir: match r.workdir {
                    Some(s) => s,
                    None => get_default_workdir(),
                },
                writable: match r.writable {
                    Some(s) => s,
                    None => get_default_writable(),
                },
            }
        )
    }
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

fn get_search_paths(sys_search_path: String) -> Vec<String> {
    let mut search_paths = vec![];

    // $EDF_PATH or $HOME/.edf or ""
    let edf_path = match std::env::var("EDF_PATH") {
        Ok(p) => p,
        Err(_) => {
            let home_path = match std::env::var("HOME") {
                Ok(h) => h,
                Err(_) => String::from(""),
            };
            if home_path != "" {
                format!("{home_path}/.edf")
            } else {
                String::from("")
            }

        },
    };
    if edf_path != "" {
      search_paths.push(edf_path);
    }

    // add sys_search_path
    if sys_search_path != "" {
      search_paths.push(sys_search_path);
    }

    search_paths
}

fn resolve_env_path(env: String, sp: &Vec<String>) -> SarusResult<String> {

    let mut retopt = None;
    let mut file_path;

    // it doesn't look like a file_path
    if ! [".", "/"].iter().any(|s| env.starts_with(*s)) &&
        ! env.ends_with(".toml") {
            for s in sp.iter() {
                file_path = format!("{s}/{env}.toml");
                if std::path::Path::new(&file_path).is_file() {
                    match std::fs::File::open(&file_path) {
                        Ok(_) => {
                            retopt = Some(file_path.clone());
                            break;
                        },
                        Err(_) => continue,
                    };
                }
            }
    } else {
        file_path = env.clone();
        if std::path::Path::new(&file_path).is_file() {
            match std::fs::File::open(&file_path) {
                Ok(_) => {
                    retopt = Some(file_path.clone());
                },
                Err(_) => {},
            }
        }
    }

    match retopt {
        Some(s) => return Ok(s),
        None => {
            let paths = sp.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");
            return Err(
            SarusError {
                code: 6,
                msg: String::from(format!("environment \"{env}\" not found at {paths}")),
            });
        },
    }
}

fn render_inner_loop(name: String, oedf: Option<&RawEDF>, sp: &Vec<String>, mut count: u64, max: u64) -> SarusResult<RawEDF> {

    count += 1;
    if count > max {
        return Err(
            SarusError {
                code: 5,
                msg: String::from(format!("base_environment rendering has more than {max} levels")),
            });
    }

    let edf_path = resolve_env_path(name.clone(), sp)?;

    validate(edf_path.clone())?;

    let path_str = edf_path.as_str();

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

    let mut e: RawEDF = toml_value;
    let mut ei = None;

    if e.base_environment.is_some() {
        let be = e.base_environment.clone().unwrap();
    
        let ba = match be {
            BaseEnvironment::TypeString(s) => vec![s],
            BaseEnvironment::TypeVec(a) => a,
        };

        for b in ba.iter() {
            ei = Some(render_inner_loop(b.to_string(), oedf, &sp, count, max)?);
        }
        e.base_environment = None;
    }

    if ei.is_some() {
        let i = ei.unwrap();

        if i.annotations.is_some() {
            if e.annotations.is_none() {
                e.annotations = i.annotations;
            }
        }
        if i.devices.is_some() {
            if e.devices.is_none() {
                e.devices = i.devices;
            }
        }
        if i.entrypoint.is_some() {
            if e.entrypoint.is_none() {
                e.entrypoint = i.entrypoint;
            }
        }
        if i.env.is_some() {
            if e.env.is_none() {
                e.env = i.env;
            }
        }
        if i.image.is_some() {
            if e.image.is_none() {
                e.image = i.image;
            }
        }
        if i.mounts.is_some() {
            if e.mounts.is_none() {
                e.mounts = i.mounts;
            }
        }
        if i.workdir.is_some() {
            if e.workdir.is_none() {
                e.workdir = i.workdir;
            }
        }
        if i.writable.is_some() {
            if e.writable.is_none() {
                e.writable = i.writable;
            }
        }
    }

    return Ok(e)
}

pub fn render(path: String, sys_search_path: String) -> SarusResult<EDF> {

    let sp = get_search_paths(sys_search_path);
    let max_levels = 10;
    let loop_count = 0;
    let raw = render_inner_loop(path, None, &sp, loop_count, max_levels)?;
    let e = EDF::try_from(raw)?;
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
                let r = render(fstr,String::from("/etc/edf"));
                assert!(r.is_ok());
            }
        }
    }

    #[test]
    fn file_not_found() {
        let result = render(String::from("src/toml/not_found.toml"),String::from("/etc/edf"));
        assert!(result.is_err());
    }

    #[test]
    fn not_a_toml_file() {
        let result = render(String::from("src/toml/test.txt"),String::from("/etc/edf"));
        assert!(result.is_err());
    }
}
