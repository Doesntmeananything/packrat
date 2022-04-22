use futures::{stream, StreamExt};
use node_semver::Version;
use registry::Metadata;
use reqwest::Client;

use crate::package::{PackageJson, Report};

mod package;
mod registry;
mod text;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut padding: usize = 0;
    println!("{:padding$}> Running Packrat to update NPM packages:", "");

    let mut package_json = PackageJson::new()?;

    let client = Client::new();

    padding = 2;
    println!("{:padding$}> Searching NPM for updates...", "");

    let metadata = stream::iter(package_json.all_dependency_keys_iter())
        .map(|package_name| {
            let client = &client;
            registry::fetch_metadata(client, package_name)
        })
        .buffer_unordered(8)
        .collect::<Vec<Result<Metadata, reqwest::Error>>>()
        .await;

    let mut patch_updates: Vec<Report> = vec![];
    let mut patch_dev_index: usize = 0;

    let mut minor_updates: Vec<Report> = vec![];
    let mut minor_dev_index: usize = 0;

    let mut major_updates: Vec<Report> = vec![];
    let mut major_dev_index: usize = 0;

    let mut pre_v1_updates: Vec<Report> = vec![];
    let mut pre_v1_dev_index: usize = 0;

    let dev_start_index = package_json.dependencies().len();

    package_json.all_dependencies_iter().enumerate().for_each(
        |(index, (package_name, version))| {
            for package in &metadata {
                match package {
                    Ok(package) => {
                        if package_name != &package.name {
                            continue;
                        }

                        let raw_current_version = &version.as_str().unwrap();
                        let range_symbol =
                            if raw_current_version.starts_with(|p: char| !p.is_numeric()) {
                                raw_current_version.chars().next()
                            } else {
                                None
                            };
                        let current_version = &raw_current_version.replace('^', "");
                        let latest_version = &package.dist_tags.latest;

                        if current_version == latest_version {
                            continue;
                        }

                        let parsed_current_version: Version = current_version.parse().unwrap();
                        let parsed_latest_version: Version = latest_version.parse().unwrap();

                        let report = Report::new(
                            package_name.to_string(),
                            current_version.to_string(),
                            latest_version.to_string(),
                            range_symbol,
                        );

                        if index == dev_start_index {
                            major_dev_index = major_updates.len();
                            minor_dev_index = minor_updates.len();
                            patch_dev_index = patch_updates.len();
                            pre_v1_dev_index = pre_v1_updates.len();
                        }

                        if parsed_latest_version.major == 0 {
                            pre_v1_updates.push(report);
                            continue;
                        }

                        if parsed_latest_version.major > parsed_current_version.major {
                            major_updates.push(report);
                            continue;
                        }

                        if parsed_latest_version.minor > parsed_current_version.minor {
                            minor_updates.push(report);
                            continue;
                        }

                        if parsed_latest_version.patch > parsed_current_version.patch {
                            patch_updates.push(report);
                            continue;
                        }
                    }
                    Err(error) => {
                        eprintln!("Unable to get up-to-date package information: {}", error);
                        break;
                    }
                }
            }
        },
    );

    if !patch_updates.is_empty() {
        padding = 4;
        text::print_section_name(text::VersionSection::Patch, padding);

        padding = 6;
        for (index, report) in patch_updates.into_iter().enumerate() {
            if index == patch_dev_index {
                text::print_dev_subheading(padding);
            }

            let current_version = report.current_version.parse::<Version>().unwrap();
            let latest_version = report.latest_version.parse::<Version>().unwrap();

            text::print_report_line(
                text::VersionSection::Patch,
                padding,
                &report.package_name,
                current_version,
                latest_version,
            );

            package_json.update_dependency_version(report);
        }
    }

    if !minor_updates.is_empty() {
        padding = 4;
        text::print_section_name(text::VersionSection::Minor, padding);

        padding = 6;
        for (index, report) in minor_updates.into_iter().enumerate() {
            if index == minor_dev_index {
                text::print_dev_subheading(padding);
            }

            let current_version = report.current_version.parse::<Version>().unwrap();
            let latest_version = report.latest_version.parse::<Version>().unwrap();

            text::print_report_line(
                text::VersionSection::Minor,
                padding,
                &report.package_name,
                current_version,
                latest_version,
            );

            package_json.update_dependency_version(report);
        }
    }

    if !major_updates.is_empty() {
        padding = 4;
        text::print_section_name(text::VersionSection::Major, padding);

        padding = 6;
        for (index, report) in major_updates.into_iter().enumerate() {
            if index == major_dev_index {
                text::print_dev_subheading(padding);
            }

            let current_version = report.current_version.parse::<Version>().unwrap();
            let latest_version = report.latest_version.parse::<Version>().unwrap();

            text::print_report_line(
                text::VersionSection::Major,
                padding,
                &report.package_name,
                current_version,
                latest_version,
            );

            package_json.update_dependency_version(report);
        }
    }

    if !pre_v1_updates.is_empty() {
        padding = 4;
        text::print_section_name(text::VersionSection::PreV1, padding);

        padding = 6;
        for (index, report) in pre_v1_updates.into_iter().enumerate() {
            if index == pre_v1_dev_index {
                text::print_dev_subheading(padding);
            }

            let current_version = report.current_version.parse::<Version>().unwrap();
            let latest_version = report.latest_version.parse::<Version>().unwrap();

            text::print_report_line(
                text::VersionSection::PreV1,
                padding,
                &report.package_name,
                current_version,
                latest_version,
            );

            package_json.update_dependency_version(report);
        }
    }

    text::print_update_prompt();

    package_json
        .write_to_file()
        .expect("Unable to update package.json");

    Ok(())
}
