use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;

use crate::common::{expand_vars_hashmap, expand_vars_string, expand_vars_vec};
use crate::config::load_config;
use crate::error::{SarusError, SarusResult};
use crate::mount::{SarusMounts, sarus_mounts_from_strings};

pub mod common;
pub mod config;
pub mod error;
pub mod mount;

#[allow(dead_code)]
#[derive(Derivative, Serialize, Deserialize, Clone)]
pub struct RawEDF {
    annotations: Option<HashMap<String, String>>,
    base_environment: Option<BaseEnvironment>,
    devices: Option<Vec<String>>,
    entrypoint: Option<bool>,
    env: Option<HashMap<String, String>>,
    engine: Option<String>,
    image: Option<String>,
    mounts: Option<Vec<String>>,
    parallax_enable: Option<bool>,
    parallax_imagestore: Option<String>,
    parallax_path: Option<String>,
    parallax_mount_program: Option<String>,
    perfmon: Option<bool>,
    podman_module: Option<String>,
    podman_path: Option<String>,
    podman_tmp_path: Option<String>,
    workdir: Option<String>,
    writable: Option<bool>,
}

#[allow(dead_code)]
#[derive(Derivative, Serialize, Deserialize, Clone)]
pub struct EDF {
    #[serde(default = "get_default_annotations")]
    pub annotations: HashMap<String, String>,
    #[serde(default = "get_default_devices")]
    pub devices: Vec<String>,
    #[serde(default = "get_default_entrypoint")]
    pub entrypoint: bool,
    #[serde(default = "get_default_env")]
    pub env: HashMap<String, String>,
    pub image: String,
    #[serde(default = "get_default_mounts")]
    pub mounts: SarusMounts,
    #[serde(default = "get_default_parallax_enable")]
    pub parallax_enable: bool,
    #[serde(default = "get_default_parallax_imagestore")]
    pub parallax_imagestore: String,
    #[serde(default = "get_default_parallax_mount_program")]
    pub parallax_mount_program: String,
    #[serde(default = "get_default_parallax_path")]
    pub parallax_path: String,
    #[serde(default = "get_default_perfmon")]
    pub perfmon: bool,
    #[serde(default = "get_default_podman_module")]
    pub podman_module: String,
    #[serde(default = "get_default_podman_path")]
    pub podman_path: String,
    #[serde(default = "get_default_podman_tmp_path")]
    pub podman_tmp_path: String,
    #[serde(default = "get_default_workdir")]
    pub workdir: String,
    #[serde(default = "get_default_writable")]
    pub writable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum BaseEnvironment {
    TypeString(String),
    TypeVec(Vec<String>),
}

fn get_default_annotations() -> HashMap<String, String> {
    return HashMap::from([]);
}

fn get_default_devices() -> Vec<String> {
    return vec![];
}

fn get_default_entrypoint() -> bool {
    return false;
}

fn get_default_env() -> HashMap<String, String> {
    return HashMap::from([]);
}

fn get_default_mounts() -> SarusMounts {
    return vec![];
}

fn get_default_parallax_enable() -> bool {
    return true;
}

fn get_default_parallax_imagestore() -> String {
    return String::from("");
}

fn get_default_parallax_mount_program() -> String {
    return String::from("");
}

fn get_default_perfmon() -> bool {
    return false;
}

fn get_default_podman_module() -> String {
    return String::from("hpc");
}

fn get_default_podman_path() -> String {
    return String::from("podman");
}

fn get_default_podman_tmp_path() -> String {
    return String::from("/dev/shm");
}

fn get_default_parallax_path() -> String {
    return String::from("parallax");
}

fn get_default_workdir() -> String {
    return String::from("");
}

fn get_default_writable() -> bool {
    return true;
}

impl TryFrom<RawEDF> for EDF {
    type Error = SarusError;

    fn try_from(r: RawEDF) -> SarusResult<Self> {
        Ok(EDF {
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
                None => {
                    return Err(SarusError {
                        code: 7,
                        file_path: None,
                        msg: String::from("missing image specification"),
                    });
                }
            },
            mounts: match r.mounts {
                Some(s) => sarus_mounts_from_strings(s)?,
                None => get_default_mounts(),
            },
            parallax_enable: match r.parallax_enable {
                Some(s) => s,
                None => get_default_parallax_enable(),
            },
            parallax_imagestore: match r.parallax_imagestore {
                Some(s) => s,
                None => get_default_parallax_imagestore(),
            },
            parallax_mount_program: match r.parallax_mount_program {
                Some(s) => s,
                None => get_default_parallax_mount_program(),
            },
            parallax_path: match r.parallax_path {
                Some(s) => s,
                None => get_default_parallax_path(),
            },
            perfmon: match r.perfmon {
                Some(s) => s,
                None => get_default_perfmon(),
            },
            podman_module: match r.podman_module {
                Some(s) => s,
                None => get_default_podman_module(),
            },
            podman_path: match r.podman_path {
                Some(s) => s,
                None => get_default_podman_path(),
            },
            podman_tmp_path: match r.podman_tmp_path {
                Some(s) => s,
                None => get_default_podman_tmp_path(),
            },
            workdir: match r.workdir {
                Some(s) => s,
                None => get_default_workdir(),
            },
            writable: match r.writable {
                Some(s) => s,
                None => get_default_writable(),
            },
        })
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

    if !fp.exists() {
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
            return Err(SarusError {
                code: 0,
                file_path: None,
                msg: String::from("Failed to parse schema file"),
            });
        }
    };

    let validator = match jsonschema::options().build(&schema) {
        Ok(v) => v,
        Err(error) => {
            return Err(SarusError {
                code: 1,
                file_path: None,
                msg: String::from(format!("Schema is invalid.\n{error}")),
            });
        }
    };

    let toml_content = match load(path_str) {
        Ok(c) => c,
        Err(e) => {
            return Err(SarusError {
                code: 2,
                file_path: Some(String::from(path_str)),
                msg: String::from(format!("{}", e)),
            });
        }
    };

    let toml_value = match toml::from_str(&toml_content) {
        Ok(v) => v,
        Err(e) => {
            return Err(SarusError {
                code: 15,
                file_path: Some(String::from(path_str)),
                msg: String::from(format!("{}", e)),
            });
        }
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
        return Err(SarusError {
            code: 4,
            file_path: Some(String::from(path_str)),
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
        }
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

    let ee = expand_vars_string(env)?;

    // it doesn't look like a file_path
    if ![".", "/"].iter().any(|s| ee.starts_with(*s)) && !ee.ends_with(".toml") {
        for s in sp.iter() {
            file_path = format!("{s}/{ee}.toml");
            if std::path::Path::new(&file_path).is_file() {
                match std::fs::File::open(&file_path) {
                    Ok(_) => {
                        retopt = Some(file_path.clone());
                        break;
                    }
                    Err(_) => continue,
                };
            }
        }
    } else {
        file_path = ee.clone();
        if std::path::Path::new(&file_path).is_file() {
            match std::fs::File::open(&file_path) {
                Ok(_) => {
                    retopt = Some(file_path.clone());
                }
                Err(_) => {}
            }
        }
    }

    match retopt {
        Some(s) => return Ok(s),
        None => {
            let paths = sp
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            return Err(SarusError {
                code: 6,
                file_path: None,
                msg: String::from(format!("environment \"{ee}\" not found at {paths}")),
            });
        }
    }
}
/*
fn expand_vars_string(s: String) -> SarusResult<String> {
    match shellexpand::env(&s) {
        Ok(ok) => return Ok(ok.to_string()),
        Err(e) => return Err(SarusError {
            code: 16,
            file_path: None,
            msg: String::from(format!("cannot expand variable {}, {}", e.var_name, e.cause)),
        }),
    };
}

fn expand_vars_hashmap(h: HashMap<String, String>) -> SarusResult<HashMap<String, String>> {
    let mut newh = h.clone();
    for (k,v) in h {
        let ev = expand_vars_string(v.clone())?;
        if ev != v {
            newh.insert(k,ev);
        }
    }
    return Ok(newh);
}

fn expand_vars_vec(v: Vec<String>) -> SarusResult<Vec<String>> {
    let mut newv = vec![];
    for s in v {
        newv.push(expand_vars_string(s)?);
    }
    return Ok(newv);
}
*/

fn render_inner_loop(
    name: String,
    oedf: Option<&RawEDF>,
    sp: &Vec<String>,
    mut count: u64,
    max: u64,
) -> SarusResult<RawEDF> {
    count += 1;
    if count > max {
        return Err(SarusError {
            code: 5,
            file_path: None,
            msg: String::from(format!(
                "base_environment rendering has more than {max} levels"
            )),
        });
    }

    let edf_path = resolve_env_path(name.clone(), sp)?;

    validate(edf_path.clone())?;

    let path_str = edf_path.as_str();

    let toml_content = match load(path_str) {
        Ok(c) => c,
        Err(e) => {
            return Err(SarusError {
                code: 2,
                file_path: Some(String::from(path_str)),
                msg: String::from(format!("{}", e)),
            });
        }
    };

    let toml_value = match toml::from_str(toml_content.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return Err(SarusError {
                code: 3,
                file_path: Some(String::from(edf_path)),
                msg: String::from(format!("{}", e)),
            });
        }
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

        match i.annotations {
            Some(mut a) => match e.annotations {
                Some(ref mut b) => {
                    a.extend(b.clone());
                    e.annotations = Some(a);
                }
                None => {
                    e.annotations = Some(a);
                }
            },
            None => (),
        }
        match i.devices {
            Some(mut a) => match e.devices {
                Some(b) => {
                    a.extend(b);
                    e.devices = Some(a);
                }
                None => {
                    e.devices = Some(a);
                }
            },
            None => (),
        }
        if i.entrypoint.is_some() {
            if e.entrypoint.is_none() {
                e.entrypoint = i.entrypoint;
            }
        }
        match i.env {
            Some(mut a) => match e.env {
                Some(ref mut b) => {
                    a.extend(b.clone());
                    e.env = Some(a);
                }
                None => {
                    e.env = Some(a);
                }
            },
            None => (),
        }
        if i.image.is_some() {
            if e.image.is_none() {
                e.image = i.image;
            }
        }
        match i.mounts {
            Some(mut a) => match e.mounts {
                Some(ref mut b) => {
                    a.append(b);
                    e.mounts = Some(a);
                }
                None => {
                    e.mounts = Some(a);
                }
            },
            None => (),
        }
        if i.parallax_enable.is_some() {
            if e.parallax_enable.is_none() {
                e.parallax_enable = i.parallax_enable;
            }
        }
        if i.parallax_imagestore.is_some() {
            if e.parallax_imagestore.is_none() {
                e.parallax_imagestore = i.parallax_imagestore;
            }
        }
        if i.parallax_mount_program.is_some() {
            if e.parallax_mount_program.is_none() {
                e.parallax_mount_program = i.parallax_mount_program;
            }
        }
        if i.parallax_path.is_some() {
            if e.parallax_path.is_none() {
                e.parallax_path = i.parallax_path;
            }
        }
        if i.perfmon.is_some() {
            if e.perfmon.is_none() {
                e.perfmon = i.perfmon;
            }
        }
        if i.podman_module.is_some() {
            if e.podman_module.is_none() {
                e.podman_module = i.podman_module;
            }
        }
        if i.podman_path.is_some() {
            if e.podman_path.is_none() {
                e.podman_path = i.podman_path;
            }
        }
        if i.podman_tmp_path.is_some() {
            if e.podman_tmp_path.is_none() {
                e.podman_tmp_path = i.podman_tmp_path;
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

    if e.devices.is_some() {
        // Expand variables
        e.devices = Some(expand_vars_vec(e.devices.unwrap())?);

        //Remove duplicates from devices
        let dev = e.devices.clone().unwrap();
        let dev_set: HashSet<_> = dev.into_iter().collect();
        let dev_unique_vec: Vec<_> = dev_set.into_iter().collect();
        e.devices = Some(dev_unique_vec);
    }
    if e.env.is_some() {
        e.env = Some(expand_vars_hashmap(e.env.unwrap())?);
    }
    if e.annotations.is_some() {
        e.annotations = Some(expand_vars_hashmap(e.annotations.unwrap())?);
    }
    if e.engine.is_some() {
        e.engine = Some(expand_vars_string(e.engine.unwrap())?);
    }
    if e.parallax_imagestore.is_some() {
        e.parallax_imagestore = Some(expand_vars_string(e.parallax_imagestore.unwrap())?);
    }
    if e.parallax_path.is_some() {
        e.parallax_path = Some(expand_vars_string(e.parallax_path.unwrap())?);
    }
    if e.parallax_mount_program.is_some() {
        e.parallax_mount_program = Some(expand_vars_string(e.parallax_mount_program.unwrap())?);
    }
    if e.podman_module.is_some() {
        e.podman_module = Some(expand_vars_string(e.podman_module.unwrap())?);
    }
    if e.podman_path.is_some() {
        e.podman_path = Some(expand_vars_string(e.podman_path.unwrap())?);
    }
    if e.podman_tmp_path.is_some() {
        e.podman_tmp_path = Some(expand_vars_string(e.podman_tmp_path.unwrap())?);
    }
    if e.workdir.is_some() {
        e.workdir = Some(expand_vars_string(e.workdir.unwrap())?);
    }

    return Ok(e);
}

pub fn render(path: String) -> SarusResult<EDF> {
    let config = load_config();
    let sp = get_search_paths(config.edf_system_search_path);
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
            let fname = fpath
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();
            if fname.contains("good") {
                let fstr = fpath.into_os_string().into_string().unwrap();
                let r = render(fstr, String::from("/etc/edf"));
                assert!(r.is_ok());
            }
        }
    }

    #[test]
    fn file_not_found() {
        let result = render(
            String::from("src/toml/not_found.toml"),
            String::from("/etc/edf"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn not_a_toml_file() {
        let result = render(String::from("src/toml/test.txt"), String::from("/etc/edf"));
        assert!(result.is_err());
    }
}
