use serde::{Deserialize, Serialize};
use serde_json::{
    map::{Iter, Keys},
    Map, Value,
};
use std::{
    fs::File,
    io::{Read, Write},
    iter::Chain,
};

/// A structure that represents a `package.json` file, which contains information about dependencies of a project.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    pub values: Value,
}

impl PackageJson {
    pub fn new() -> std::io::Result<PackageJson> {
        let mut raw_package_json = File::open("./package.json")?;
        let mut contents = String::new();
        raw_package_json.read_to_string(&mut contents)?;
        let values: Value = serde_json::from_str(&contents)?;

        Ok(PackageJson { values })
    }

    pub fn dependencies(&self) -> &Map<String, Value> {
        self.values["dependencies"].as_object().unwrap()
    }

    pub fn dev_dependencies(&self) -> &Map<String, Value> {
        self.values["devDependencies"].as_object().unwrap()
    }

    pub fn all_dependency_keys_iter(&self) -> Chain<Keys, Keys> {
        self.dependencies()
            .keys()
            .chain(self.dev_dependencies().keys())
    }

    pub fn all_dependencies_iter(&self) -> Chain<Iter, Iter> {
        self.dependencies()
            .iter()
            .chain(self.dev_dependencies().iter())
    }

    pub fn update_dependency_version(&mut self, report: Report) {
        // If package name contains `/`, represent it as `~1` to be in line with the JSON pointer spec:
        // https://datatracker.ietf.org/doc/html/rfc6901#section-3
        //
        // JSON pointers are used to access and mutate `dependencies` and `devDependencies` in `package.json`
        let package_json_pointer = report.package_name.replace('/', "~1");
        let latest_version = match report.range_symbol {
            Some(range_symbol) => range_symbol.to_string() + &report.latest_version,
            None => report.latest_version,
        };

        if let Some(v) = self
            .values
            .pointer_mut(format!("/dependencies/{}", package_json_pointer).as_str())
        {
            *v = latest_version.into();
        } else if let Some(v) = self
            .values
            .pointer_mut(format!("/devDependencies/{}", package_json_pointer).as_str())
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

pub struct Report {
    pub package_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub range_symbol: Option<char>,
}

impl Report {
    pub fn new(
        package_name: String,
        current_version: String,
        latest_version: String,
        range_symbol: Option<char>,
    ) -> Self {
        Report {
            package_name,
            current_version,
            latest_version,
            range_symbol,
        }
    }
}
