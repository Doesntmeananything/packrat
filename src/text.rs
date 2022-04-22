use std::{
    io::{self, BufRead, Write},
    process,
};

use ansi_term::{Color, Style};
use node_semver::Version;

#[derive(PartialEq)]
pub enum VersionSection {
    Patch,
    Minor,
    Major,
    PreV1,
}

pub fn print_section_name(section_type: VersionSection, padding: usize) {
    let heading = match section_type {
        VersionSection::Patch => Color::Green.bold().underline().paint("Patch updates"),
        VersionSection::Minor => Color::Yellow.bold().underline().paint("Minor updates"),
        VersionSection::Major => Color::Red.bold().underline().paint("Major updates"),
        VersionSection::PreV1 => Color::Purple.bold().underline().paint("0.x.x updates"),
    };
    let description = match section_type {
        VersionSection::Patch => Color::Green.paint("Backward compatible bug fixes."),
        VersionSection::Minor => Color::Yellow.paint("New backward compatible features."),
        VersionSection::Major => Color::Red.paint("Changes that break backward compatibility."),
        VersionSection::PreV1 => Color::Purple.paint("Initial development, unstable public API."),
    };

    // Add empty line separator between sections
    if section_type != VersionSection::Patch {
        println!();
    }

    println!("{:padding$}{} {}", "", heading, description)
}

pub fn print_dev_subheading(padding: usize) {
    println!(
        "{:padding$}{}",
        "",
        Color::Cyan.paint("🔧 Dev dependencies")
    )
}

pub fn print_report_line(
    section_type: VersionSection,
    padding: usize,
    name: &String,
    current_version: Version,
    latest_version: Version,
) {
    let current_version_style = Style::new().dimmed().bold();
    let latest_version_style = match section_type {
        VersionSection::Patch => Color::Green.bold(),
        VersionSection::Minor => Color::Yellow.bold(),
        VersionSection::Major => Color::Red.bold(),
        VersionSection::PreV1 => Color::Purple.bold(),
    };

    let pretty_current = match section_type {
        VersionSection::Patch => format!(
            "{}.{}.{}",
            current_version.major,
            current_version.minor,
            current_version_style.paint(current_version.patch.to_string())
        ),
        VersionSection::Minor => format!(
            "{}.{}.{}",
            current_version.major,
            current_version_style.paint(current_version.minor.to_string()),
            current_version_style.paint(current_version.patch.to_string())
        ),
        VersionSection::Major => format!(
            "{}.{}.{}",
            current_version_style.paint(current_version.major.to_string()),
            current_version_style.paint(current_version.minor.to_string()),
            current_version_style.paint(current_version.patch.to_string())
        ),
        VersionSection::PreV1 => format!(
            "{}.{}.{}",
            current_version.major,
            current_version_style.paint(current_version.minor.to_string()),
            current_version_style.paint(current_version.patch.to_string())
        ),
    };

    let pretty_latest = match section_type {
        VersionSection::Patch => format!(
            "{}.{}.{}",
            latest_version.major,
            latest_version.minor,
            latest_version_style.paint(latest_version.patch.to_string())
        ),
        VersionSection::Minor => format!(
            "{}.{}.{}",
            latest_version.major,
            latest_version_style.paint(latest_version.minor.to_string()),
            latest_version_style.paint(latest_version.patch.to_string())
        ),
        VersionSection::Major => format!(
            "{}.{}.{}",
            latest_version_style.paint(latest_version.major.to_string()),
            latest_version_style.paint(latest_version.minor.to_string()),
            latest_version_style.paint(latest_version.patch.to_string())
        ),
        VersionSection::PreV1 => format!(
            "{}.{}.{}",
            latest_version.major,
            latest_version_style.paint(latest_version.minor.to_string()),
            latest_version_style.paint(latest_version.patch.to_string())
        ),
    };

    println!(
        "{:padding$}{} {} -> {}",
        "", name, pretty_current, pretty_latest
    )
}

pub fn print_update_prompt() {
    print!("\nUpdate package.json with these package versions? [y/n] ");
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line.unwrap().as_str() {
            "y" | "Y" => break,
            "n" | "N" => {
                println!("\nCancelled update.");
                process::exit(1);
            }
            _ => {
                print!(
                    "\nAnswer 'y' or 'n'. Update package.json with these package versions? [y/n] "
                );
                io::stdout().flush().unwrap();
            }
        }
    }
}
