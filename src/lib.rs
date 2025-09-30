use toml::Table;
use toml::Value;
use toml::map::Map;
use std::error::Error;
use std::path::Path;
use std::ffi::OsStr;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct EDF {
   _base_environment: Option<BaseEnvironment>,
   _image: Option<String>,
}

#[derive(Deserialize)]
pub enum BaseEnvironment {
    TypeString(String),
    TypeVec(Vec<String>),
}

pub fn load(file_path: &str) -> Result<String, Box<dyn Error>> {

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

  //println!("This is its content:\n\n{outstr}\n");

  Ok(outstr)
}

pub fn toml_parse(content: &str) -> Result<Map<String, Value>, Box<dyn Error>> {

    let table = content.parse::<Table>()?;
    Ok(table)
}

pub fn edf_parse(toml: Map<String, Value>) -> Result<Map<String, Value>, Box<dyn Error>> {
    let _allowed_fields = vec![
                              "annotations",
                              "base_environment",
                              "entrypoint",
                              "env",
                              "image",
                              "mounts",
                              "workdir",
                              "writable"
                              ];
    Ok(toml)
}


pub fn edf_parse2(input: &str) -> Result<EDF, Box<dyn Error>> {
    let edf: EDF = toml::from_str(input)?;
    //println!("base_environment: {}", edf.base_environment.ok_or("NULL")?);
    //println!("image: {}", edf.image.ok_or("NULL")?);
    Ok(edf)
}

pub fn edf_load(file_path: &str) -> Result<Map<String, Value>, Box<dyn Error>> {

    let _content = load(file_path)?;
    let _toml_table = toml_parse(&_content)?;
    let edf_map = edf_parse(_toml_table)?; 
    Ok(edf_map)
}

pub fn edf_load2(file_path: &str) -> Result<EDF, Box<dyn Error>> {

    let _content = load(file_path)?;
    let edf = edf_parse2(&_content)?; 
    Ok(edf)
}

pub fn validate(path: String) -> bool {

    let path_str = path.as_str();

    // Embedding schema file
    let schema_content = include_str!("schema/edf.json");

    // From: https://docs.rs/crate/jsonschema-for-toml/0.1.0/source/src/main.rs
    // Embedded schema not needed at runtime
    /*
    let schema_path_str = "./src/schema/edf.json";

    let schema_content = match std::fs::read_to_string(&schema_path_str) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Failed to read schema file: {}", schema_path_str);
            return false;
        }
    };

    let schema: serde_json::Value = match serde_json::from_str(&schema_content) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Failed to parse schema file: {}", schema_path_str);
            return false;
        }
    };
    */

    let schema: serde_json::Value = match serde_json::from_str(&schema_content) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Failed to parse schema file");
            return false;
        }
    };

    // Create validator
    let validator = match jsonschema::options().build(&schema) {
        Ok(v) => v,
        Err(error) => {
            eprintln!("Schema is invalid. Error: {error}");
            return false;
        }
    };
    
    let toml_content = match load(path_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return false;
        },
    };

    let toml_value = match toml::from_str(&toml_content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return false;
        },
    };

    let mut has_errors = false;
    let mut errors = validator.iter_errors(&toml_value);

    if let Some(first) = errors.next() {
        has_errors = true;
        println!("{path_str} is an INVALID EDF file.");
        eprintln!("\nErrors:");
        eprintln!("1. {first}");
        for (i, error) in errors.enumerate() {
            eprintln!("{}. {error}", i + 2);
        }
    } else {
        println!("{path_str} is a valid EDF file");
    }

    if has_errors {
        return false;
    } else {
        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_toml() {
        let table = edf_load("src/toml/test.toml").unwrap();
        let value = table["foo"].as_str().unwrap();
        let expected = "bar";
        assert_eq!(value,expected);
        /*
        let result = edf_load2("src/toml/test.toml");
        println!("{}",result.is_err());
        assert!(result.is_err());
        */
    }

    #[test]
    fn file_not_found() {
        let result = edf_load("src/toml/not_found.toml");
        assert!(result.is_err());
    }

    #[test]
    fn not_a_toml_file() {
        let result = edf_load("src/toml/test.txt");
        assert!(result.is_err());
    }
}
