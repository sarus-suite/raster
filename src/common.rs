use std::collections::HashMap;

use crate::error::{SarusError, SarusResult};

pub fn expand_vars_string(s: String) -> SarusResult<String> {
    match shellexpand::env(&s) {
        Ok(ok) => return Ok(ok.to_string()),
        Err(e) => {
            return Err(SarusError {
                code: 16,
                file_path: None,
                msg: String::from(format!(
                    "cannot expand variable {}, {}",
                    e.var_name, e.cause
                )),
            });
        }
    };
}

pub fn expand_vars_hashmap(h: HashMap<String, String>) -> SarusResult<HashMap<String, String>> {
    let mut newh = h.clone();
    for (k, v) in h {
        let ev = expand_vars_string(v.clone())?;
        if ev != v {
            newh.insert(k, ev);
        }
    }
    return Ok(newh);
}

pub fn expand_vars_vec(v: Vec<String>) -> SarusResult<Vec<String>> {
    let mut newv = vec![];
    for s in v {
        newv.push(expand_vars_string(s)?);
    }
    return Ok(newv);
}
