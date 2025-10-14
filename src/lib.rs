use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use derivative::Derivative;

use crate::error::{SarusError, SarusResult};
use crate::mount::{SarusMounts, sarus_mounts_from_strings};

pub mod error;
pub mod mount;

/*
pub type SarusResult<T> = std::result::Result<T, SarusError>;

#[derive(Debug, Clone, Deserialize)]
pub struct SarusError {
    code: u64,
    file_path: Option<String>,
    msg: String,
}

impl std::fmt::Display for SarusError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fp = match &self.file_path {
            Some(p) => format!(" on {p}"),
            None => String::from(""), 
        };
        write!(f, "Error {:03}{}: {}", self.code, fp, self.msg)
    }
}

impl Error for SarusError {}
*/

#[allow(dead_code)]
#[derive(Derivative, Serialize, Deserialize, Clone)]
pub struct RawEDF {
   annotations: Option<HashMap<String,String>>,
   base_environment: Option<BaseEnvironment>,
   devices: Option<Vec<String>>,
   entrypoint: Option<bool>,
   env: Option<HashMap<String,String>>,
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
   annotations: HashMap<String,String>,
   #[serde(default = "get_default_devices")]
   devices: Vec<String>,
   #[serde(default = "get_default_entrypoint")]
   entrypoint: bool,
   #[serde(default = "get_default_env")]
   env: HashMap<String,String>,
   image: String,
   #[serde(default = "get_default_mounts")]
   mounts: SarusMounts,
   #[serde(default = "get_default_parallax_enable")]
   parallax_enable: bool,
   #[serde(default = "get_default_parallax_imagestore")]
   parallax_imagestore: String,
   #[serde(default = "get_default_parallax_mount_program")]
   parallax_mount_program: String,
   #[serde(default = "get_default_parallax_path")]
   parallax_path: String,
   #[serde(default = "get_default_perfmon")]
   perfmon: bool,
   #[serde(default = "get_default_podman_module")]
   podman_module: String,
   #[serde(default = "get_default_podman_path")]
   podman_path: String,
   #[serde(default = "get_default_podman_tmp_path")]
   podman_tmp_path: String,
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
// From pyxis code (still needed ???)
// escape source or target mount entry to build an fstab like entry as used by enroot
// from man 3 getmntent:
//     Since fields in the mtab and fstab files are separated by
//     whitespace, octal escapes are used to represent the characters
//     space (\040), tab (\011), newline (\012), and backslash (\\) in
//     those files when they occur in one of the four strings in a
//     mntent structure.  The routines addmntent() and getmntent() will
//     convert from string representation to escaped representation and
//     back.  When converting from escaped representation, the sequence
//     \134 is also converted to a backslash.
fn escape_mount(path: String) -> String {
    let mut epath = String::from("");
    for c1 in path.chars() {
        let c2 = match c1 {
            ' ' => format!("\\040"),
            '\t' => format!("\\011"),
            '\n' => format!("\\012"),
            '\\' => format!("\\\\"),
            _ => format!("{c1}"),
        };
        epath.push_str(c2.as_str());
    }
    epath
}

pub type SarusMounts = Vec<String>;

#[allow(dead_code)]
trait SarusMountsTrait {
    #[allow(unused_variables)]
    fn add(&mut self, mount: String) -> SarusResult<()> {
         Ok(())
    }
}

impl SarusMountsTrait for SarusMounts {
    fn add(&mut self, mount: String) -> SarusResult<()> {
        let mut a = mount.split(":");
        let asize = a.clone().count();
       
        if asize < 2 || asize > 3 {
            eprintln!("{mount}");
            return Err(
                SarusError {
                    code: 8,
                    file_path: None,
                    msg: format!("{} number of field, expected 2 or 3", asize),
                });
        };
        
        let mut s = a.next().unwrap();
        let t = a.next().unwrap();
        let mut f = "";
        if asize == 3 {
            f = a.next().unwrap();
        }
        let mut df = "";

        let mut ps: std::path::PathBuf = std::path::Path::new(s).into();
        let pt: std::path::PathBuf = std::path::Path::new(t).into();

        if f == "sqsh" {
            if ps.is_relative() {
                ps = match std::path::absolute(&ps) {
                        Err(_) => {
                            return Err(
                                SarusError {
                                    code: 9,
                                    file_path: None,
                                    msg: format!("cannot translate {} in an absolute path", ps.display()),
                                });
                            },
                        Ok(ok) => ok,    
                    } 
            } else if ps.is_absolute() {
                ()
            } else {    
                return Err(
                    SarusError {
                        code: 10,
                        file_path: None,
                        msg: format!("source of squashfs mount {} must be a relative path or an absolute path", s),
                    });
            }
            s = match ps.as_os_str().to_str() {
                Some(ok) => ok,
                None => { return Err(
                    SarusError {
                        code: 11,
                        file_path: None,
                        msg: format!("cannot translate {} into string", ps.display()),
                    })},
            };
        } else {
            if ps.is_relative() || ps.is_absolute() {
                df = "x-create=auto,rbind";
            } else {
                if s == "tmpfs" {
                    df = "x-create=dir";
                } else if s == "umount" {
                    df = "x-detach";
                } else {
                    return Err(
                        SarusError {
                            code: 12,
                            file_path: None,
                            msg: format!("mount source must be a relative path, an absolute path, \"tmpfs\" or \"umount\""),
                        });
                }
            }

        }
        
        if ! pt.is_relative() && ! pt.is_absolute() {
            return Err(
                SarusError {
                    code: 13,
                    file_path: None,
                    msg: format!("mount target must be a relative path or an absolute path"),
                });
        }

        let es = escape_mount(String::from(s));
        let et = escape_mount(String::from(t));

        let em;
        
        if f == "sqsh" {
            let metadata = match std::fs::metadata(s) {
                Ok(m) => m,
                Err(e) => {
                    return Err(
                        SarusError {
                            code: 14,
                            file_path: None,
                            msg: format!("could not stat source of squashfs mount ({s}): {e}"),
                        });
               },
            };
            if ! metadata.is_file() {
                return Err(
                    SarusError {
                        code: 14,
                        file_path: None,
                        msg: format!("source of squashfs mount ({s}) must be a regular file"),
                    });
            }
            em = format!("{es} {et}");
        } else {
            let flags;
            if f != "" {
                flags = format!("{df},{f}");
            } else {
                flags = format!("{df}");
            }
            em = format!("{es} {et} {flags}");
        }

        if ! self.contains(&em) {
                self.push(em.clone());
        }
        Ok(())
    }
}
*/
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
                        file_path: None,
                        msg: String::from("missing image specification"),
                        }),
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
                    file_path: None,
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
                    file_path: None,
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
                    file_path: Some(String::from(path_str)),
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
                    file_path: Some(String::from(path_str)),
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
                file_path: None,
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
                file_path: None,
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
                    file_path: Some(String::from(path_str)),
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
                    file_path: Some(String::from(edf_path)),
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
        
        match i.mounts {
            Some(mut a) => {
                match e.mounts {
                    Some(ref mut b) => {
                        b.append(&mut a);
                        /*
                        let mut bm = SarusMounts::new(b.clone());
                        bm.merge(SarusMounts::new(a))?;
                        */
                    },
                    None => {
                        e.mounts = Some(a);
                    },
                }
            },
            None => (),
        }
        
        /*
        match i.mounts {
            Some(a) => {
                match e.mounts {
                    Some(ref mut b) => {
                        let mut em = a.clone(); 
                        for m in b.iter() {
                            let _ = em.add(m.clone())?;
                        };
                        *b = em;
                    },
                    None => {
                        e.mounts = Some(a);
                    },
                }
            },
            None => {
                match e.mounts {
                    Some(ref mut b) => {
                        let mut em = SarusMounts::from([]); 
                        for m in b.iter() {
                            let _ = em.add(m.clone())?;
                        };
                        *b = em;
                    },
                    None => (),
                }
            },
        }
        */
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
