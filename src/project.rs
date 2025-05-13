use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A structure that represents a project with values from its `package.json` file.
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub values: Value,
}

impl Project {
    pub fn new(path: &PathBuf) -> std::io::Result<Project> {
        let mut raw_package_json = File::open(path)?;
        let mut contents = String::new();
        raw_package_json.read_to_string(&mut contents)?;
        let values: Value = serde_json::from_str(&contents)?;

        Ok(Project { values })
    }

    pub fn dependencies(&self) -> Option<&Map<String, Value>> {
        self.values["dependencies"].as_object()
    }

    pub fn dev_dependencies(&self) -> Option<&Map<String, Value>> {
        self.values["devDependencies"].as_object()
    }

    pub fn name(&self) -> &str {
        self.values["name"].as_str().unwrap()
    }

    pub fn version(&self) -> &str {
        self.values["version"].as_str().unwrap()
    }

    pub fn update_dependency_version(
        &mut self,
        name: &str,
        version: &String,
        range_prefix: Option<char>,
    ) {
        // If package name contains `/`, represent it as `~1` to be in line with the JSON pointer spec:
        // https://datatracker.ietf.org/doc/html/rfc6901#section-3
        //
        // JSON pointers are used to access and mutate `dependencies` and `devDependencies` in serialized `package.json`
        let package_json_pointer = name.replace('/', "~1");
        let latest_version = match range_prefix {
            Some(range_symbol) => range_symbol.to_string() + version,
            None => version.to_owned(),
        };

        if let Some(v) = self
            .values
            .pointer_mut(format!("/dependencies/{package_json_pointer}").as_str())
        {
            *v = latest_version.into();
        } else if let Some(v) = self
            .values
            .pointer_mut(format!("/devDependencies/{package_json_pointer}").as_str())
        {
            *v = latest_version.into();
        }
    }

    pub fn write_to_file(&self) -> std::io::Result<()> {
        let mut file = File::create("./package.json")?;
        let data = serde_json::to_string_pretty(&self.values)?;
        file.write_all(data.as_bytes())?;

        Ok(())
    }
}
