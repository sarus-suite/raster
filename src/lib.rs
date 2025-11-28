use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use toml::Value;
use toml::map::Map;

use crate::common::{expand_vars_hashmap, expand_vars_vec};
use crate::config::load_config;
use crate::error::{SarusError, SarusResult};
use crate::mount::{SarusMounts, sarus_mounts_from_strings};

pub mod common;
pub mod config;
pub mod error;
pub mod mount;

pub use crate::common::expand_vars_string;

#[allow(dead_code)]
#[derive(Derivative, Serialize, Deserialize, Clone)]
pub struct RawEDF {
    annotations: Option<Annotations>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Annotations {
    TypeMap(Map<String, Value>),
    TypeHashMap(HashMap<String, String>),
}

impl RawEDF {
    // Give priority to self's fields.
    fn merge(&mut self, i: RawEDF) {
        match i.annotations {
            Some(a) => match self.annotations {
                Some(b) => {
                    let mut a1 = annotations_as_hashmap(a);
                    let b1 = annotations_as_hashmap(b);
                    a1.extend(b1.clone());
                    //a1 = expand_vars_hashmap(a1, env)?;
                    self.annotations = Some(Annotations::TypeHashMap(a1));
                }
                None => {
                    let mut a1 = annotations_as_hashmap(a);
                    //a1 = expand_vars_hashmap(a1, env)?;
                    self.annotations = Some(Annotations::TypeHashMap(a1));
                }
            },
            None => (),
        }
        match i.devices {
            Some(mut a) => match self.devices {
                Some(b) => {
                    a.extend(b);
                    self.devices = Some(a);
                }
                None => {
                    self.devices = Some(a);
                }
            },
            None => (),
        }
        if i.entrypoint.is_some() {
            if self.entrypoint.is_none() {
                self.entrypoint = i.entrypoint;
            }
        }
        match i.env {
            Some(mut a) => match self.env {
                Some(ref mut b) => {
                    a.extend(b.clone());
                    self.env = Some(a);
                }
                None => {
                    self.env = Some(a);
                }
            },
            None => (),
        }
        if i.imagself.is_some() {
            if self.imagself.is_none() {
                self.image = i.image;
            }
        }
        match i.mounts {
            Some(mut a) => match self.mounts {
                Some(ref mut b) => {
                    a.append(b);
                    self.mounts = Some(a);
                }
                None => {
                    self.mounts = Some(a);
                }
            },
            None => (),
        }
        if i.parallax_enablself.is_some() {
            if self.parallax_enablself.is_none() {
                self.parallax_enable = i.parallax_enable;
            }
        }
        if i.parallax_imagestorself.is_some() {
            if self.parallax_imagestorself.is_none() {
                self.parallax_imagestore = i.parallax_imagestore;
            }
        }
        if i.parallax_mount_program.is_some() {
            if self.parallax_mount_program.is_none() {
                self.parallax_mount_program = i.parallax_mount_program;
            }
        }
        if i.parallax_path.is_some() {
            if self.parallax_path.is_none() {
                self.parallax_path = i.parallax_path;
            }
        }
        if i.perfmon.is_some() {
            if self.perfmon.is_none() {
                self.perfmon = i.perfmon;
            }
        }
        if i.podman_modulself.is_some() {
            if self.podman_modulself.is_none() {
                self.podman_module = i.podman_module;
            }
        }
        if i.podman_path.is_some() {
            if self.podman_path.is_none() {
                self.podman_path = i.podman_path;
            }
        }
        if i.podman_tmp_path.is_some() {
            if self.podman_tmp_path.is_none() {
                self.podman_tmp_path = i.podman_tmp_path;
            }
        }
        if i.workdir.is_some() {
            if self.workdir.is_none() {
                self.workdir = i.workdir;
            }
        }
        if i.writablself.is_some() {
            if self.writablself.is_none() {
                self.writable = i.writable;
            }
        }
    }
}

fn annotations_as_hashmap(a: Annotations) -> HashMap<String, String> {
    let r = match a {
        Annotations::TypeHashMap(h) => h,
        Annotations::TypeMap(m) => map2hashmap(m),
    };
    r
}

fn map2hashmap(m: Map<String, Value>) -> HashMap<String, String> {
    let mut r = HashMap::from([]);
    for i in m.iter() {
        let (k, v) = i;
        if v.is_table() {
            let t = v.as_table().unwrap();
            let h = map2hashmap(t.clone());
            for j in h.iter() {
                let (jk, jv) = j;
                let new_k = format!("{k}.{jk}");
                r.insert(new_k, jv.clone());
            }
        } else {
            if v.is_str() {
                r.insert(k.to_string(), v.as_str().unwrap().to_string());
            } else {
                r.insert(k.to_string(), v.to_string());
            }
        }
    }
    r
}

fn get_default_annotations() -> HashMap<String, String> {
    return HashMap::from([]);
}

fn get_default_devices() -> Vec<String> {
    return vec![];
}

fn get_default_entrypoint() -> bool {
    return true;
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

/*
impl TryFrom<RawEDF, uenv: &Option<HashMap<String,String>>> for EDF {
    type Error = SarusError;

    fn try_from(r: RawEDF, uenv: &Option<HashMap<String,String>>) -> SarusResult<Self> {
*/
fn edf_from_raw(r: RawEDF, uenv: &Option<HashMap<String, String>>) -> SarusResult<EDF> {
    Ok(EDF {
        annotations: match r.annotations {
            Some(s) => annotations_as_hashmap(s),
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
            Some(s) => sarus_mounts_from_strings(s, uenv)?,
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
    //    }
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

pub fn get_search_paths() -> Vec<String> {
    let mut search_paths = vec![];

    let user_sp = get_user_search_paths();
    search_paths.extend(user_sp);

    let sys_sp = get_sys_search_paths();
    search_paths.extend(sys_sp);

    search_paths
}

pub fn get_sys_search_paths() -> Vec<String> {
    let mut search_paths = vec![];
    let config = load_config();
    let sys_search_path = config.edf_system_search_path;

    if sys_search_path != "" {
        let paths = sys_search_path.split(":");
        for p in paths {
            search_paths.push(String::from(p));
        }
    }

    search_paths
}

pub fn get_user_search_paths() -> Vec<String> {
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

    search_paths
}

fn resolve_env_path(
    env: String,
    sp: &Vec<String>,
    uenv: &Option<HashMap<String, String>>,
) -> SarusResult<String> {
    let mut retopt = None;
    let mut file_path;

    let ee = expand_vars_string(env, uenv)?;

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

fn render_inner_loop(
    name: String,
    oedf: Option<&RawEDF>,
    sp: &Vec<String>,
    env: &Option<HashMap<String, String>>,
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

    let edf_path = resolve_env_path(name.clone(), sp, env)?;
    validate(edf_path.clone())?;

    // Create current raw EDF
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

    let mut cur_redf: RawEDF = toml_value;

    // Merge base EDFs 
    if cur_redf.base_environment.is_some() {
        let mut base_redf = RawEDF{};

        let be = cur_redf.base_environment.clone().unwrap();
        let ba = match be {
            BaseEnvironment::TypeString(s) => vec![s],
            BaseEnvironment::TypeVec(a) => a,
        };

        for b in ba.iter() {
            let _base_redf= render_inner_loop(
                b.to_string(),
                oedf,
                &sp,
                env,
                count,
                max,
            )?;
            base_redf.merge(_base_redf);
        }
        cur_redf.base_environment = None;

        cur_redf.merge(base_redf);
    }

    // Expand variables in the fields
    if cur_redf.devices.is_some() {
        cur_redf.devices = Some(expand_vars_vec(cur_redf.devices.unwrap(), env)?);

        // Remove duplicates from devices
        let dev = cur_redf.devices.clone().unwrap();
        let dev_set: HashSet<_> = dev.into_iter().collect();
        let dev_unique_vec: Vec<_> = dev_set.into_iter().collect();
        cur_redf.devices = Some(dev_unique_vec);
    }
    if cur_redf.env.is_some() {
        cur_redf.env = Some(expand_vars_hashmap(cur_redf.env.unwrap(), env)?);
    }
    if cur_redf.annotations.is_some() {
        let a = cur_redf.annotations.unwrap();
        let mut h = annotations_as_hashmap(a);
        h = expand_vars_hashmap(h, env)?;
        cur_redf.annotations = Some(Annotations::TypeHashMap(h));
    }
    if cur_redf.engincur_redf.is_some() {
        cur_redf.engine = Some(expand_vars_string(cur_redf.engincur_redf.unwrap(), env)?);
    }
    if cur_redf.parallax_imagestorcur_redf.is_some() {
        cur_redf.parallax_imagestore = Some(expand_vars_string(cur_redf.parallax_imagestorcur_redf.unwrap(), env)?);
    }
    if cur_redf.parallax_path.is_some() {
        cur_redf.parallax_path = Some(expand_vars_string(cur_redf.parallax_path.unwrap(), env)?);
    }
    if cur_redf.parallax_mount_program.is_some() {
        cur_redf.parallax_mount_program =
            Some(expand_vars_string(cur_redf.parallax_mount_program.unwrap(), env)?);
    }
    if cur_redf.podman_modulcur_redf.is_some() {
        cur_redf.podman_module = Some(expand_vars_string(cur_redf.podman_modulcur_redf.unwrap(), env)?);
    }
    if cur_redf.podman_path.is_some() {
        cur_redf.podman_path = Some(expand_vars_string(cur_redf.podman_path.unwrap(), env)?);
    }
    if cur_redf.podman_tmp_path.is_some() {
        cur_redf.podman_tmp_path = Some(expand_vars_string(cur_redf.podman_tmp_path.unwrap(), env)?);
    }
    if cur_redf.workdir.is_some() {
        cur_redf.workdir = Some(expand_vars_string(cur_redf.workdir.unwrap(), env)?);
    }

    return Ok(cur_redf);
}

pub fn render_from_search_paths(
    path: String,
    search_paths: Vec<String>,
    env: &Option<HashMap<String, String>>,
) -> SarusResult<EDF> {
    let sp = search_paths;
    let max_levels = 10;
    let loop_count = 0;
    let raw = render_inner_loop(path, None, &sp, env, loop_count, max_levels)?;
    //let e = EDF::try_from(raw, env)?;
    let e = edf_from_raw(raw, env)?;
    Ok(e)
}

pub fn render(path: String) -> SarusResult<EDF> {
    let sp = get_search_paths();
    render_from_search_paths(path, sp, &None)
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
