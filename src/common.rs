use std::collections::HashMap;
use std::process::Command;

use crate::error::{SarusError, SarusResult};

pub fn expand_vars_string(
    input: String,
    env: &Option<HashMap<String, String>>,
) -> SarusResult<String> {
    match env {
        Some(h) => expand_vars_string_with_env(input, &h),
        None => expand_vars_string_without_env(input),
    }
}

fn expand_vars_string_with_env(
    input: String,
    env: &HashMap<String, String>,
) -> SarusResult<String> {
    /*
     * Running bash here, as a unprivileged user, I suppose.
     * Probably there is a security need for vetting:
     * * running user (?)
     * * input (?)
     * * env (?)
     */
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("echo -n \"{}\"", &input))
        .env_clear()
        .envs(env)
        .output();

    let stdout = match output {
        Ok(o) => o.stdout,
        Err(e) => {
            return Err(SarusError {
                code: 18,
                file_path: None,
                msg: String::from(format!("cannot expand string {input}, {e}")),
            });
        }
    };

    let out = match str::from_utf8(&stdout) {
        Ok(o) => o,
        Err(e) => {
            return Err(SarusError {
                code: 19,
                file_path: None,
                msg: String::from(format!("cannot expand string {input}, {e}")),
            });
        }
    };

    Ok(String::from(out))
}

fn expand_vars_string_without_env(s: String) -> SarusResult<String> {
    match shellexpand::env(&s) {
        Ok(ok) => return Ok(ok.to_string()),
        Err(e) => {
            return Err(SarusError {
                code: 17,
                file_path: None,
                msg: String::from(format!(
                    "cannot expand variable {}, {}",
                    e.var_name, e.cause
                )),
            });
        }
    };
}

pub fn expand_vars_hashmap(
    h: HashMap<String, String>,
    env: &Option<HashMap<String, String>>,
) -> SarusResult<HashMap<String, String>> {
    let mut newh = h.clone();
    for (k, v) in h {
        let ev = expand_vars_string(v.clone(), env)?;
        if ev != v {
            newh.insert(k, ev);
        }
    }
    return Ok(newh);
}

pub fn expand_vars_vec(
    v: Vec<String>,
    env: &Option<HashMap<String, String>>,
) -> SarusResult<Vec<String>> {
    let mut newv = vec![];
    for s in v {
        newv.push(expand_vars_string(s, env)?);
    }
    return Ok(newv);
}
