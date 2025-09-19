use toml::Table;
use toml::Value;
use toml::map::Map;
use std::error::Error;
//use std::path::Path;
/*
use serde::Deserialize;


#[derive(Deserialize)]
pub struct EDF {
   base_environment: Option<BaseEnvironment>,
   image: Option<String>,
}

#[derive(Deserialize)]
pub enum BaseEnvironment {
    TypeString(String),
    TypeVec(Vec<String>),
}
*/

pub fn load(file_path: &str) -> Result<String, Box<dyn Error>> {
  println!("Trying to read {file_path}");

  let outstr = std::fs::read_to_string(file_path)?;

  println!("This is its content:\n\n{outstr}\n");

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

/*
pub fn edf_parse2(input: &str) -> Result<EDF, Box<dyn Error>> {
    let edf: EDF = toml::from_str(input)?;
    println!("base_environment: {}", edf.base_environment.ok_or("NULL")?);
    println!("image: {}", edf.image.ok_or("NULL")?);
    Ok(edf)
}
*/
pub fn edf_load(file_path: &str) -> Result<Map<String, Value>, Box<dyn Error>> {

    let _content = load(file_path)?;
    let _toml_table = toml_parse(&_content)?;
    let edf_map = edf_parse(_toml_table)?; 
    Ok(edf_map)
}

/*
pub fn edf_load2(file_path: &str) -> Result<EDF, Box<dyn Error>> {

    let _content = load(file_path)?;
    let edf = edf_parse2(&_content)?; 
    Ok(edf)
}
*/

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
