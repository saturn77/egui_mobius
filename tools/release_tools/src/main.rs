use clap::{Parser, Subcommand};
use color_eyre::eyre::{eyre, Result};
use semver::Version;
use std::{fs, path::PathBuf, process::Command};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run pre-release checks
    Check,
    /// Publish crates to crates.io
    Publish,
    /// Bump version numbers
    Bump { version: String },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => run_checks(),
        Commands::Publish => publish_crates(),
        Commands::Bump { version } => bump_version(&version),
    }
}

fn run_checks() -> Result<()> {
    println!("Running release checks...");
    
    // Check git status
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    if !status.stdout.is_empty() {
        return Err(eyre!("Working directory not clean"));
    }

    // Check versions are consistent
    let main_version = get_version("crates/egui_mobius/Cargo.toml")?;
    for entry in WalkDir::new("crates").min_depth(1).max_depth(2).into_iter() {
        let entry = entry?;
        if entry.file_name() == "Cargo.toml" {
            let version = get_version(entry.path())?;
            if version != main_version {
                return Err(eyre!(
                    "Version mismatch in {}: {} != {}",
                    entry.path().display(),
                    version,
                    main_version
                ));
            }
        }
    }

    // Run tests
    Command::new("cargo")
        .args(["test", "--all"])
        .status()?;

    // Run clippy
    Command::new("cargo")
        .args(["clippy", "--all", "--", "-D", "warnings"])
        .status()?;

    println!("All checks passed!");
    Ok(())
}

fn publish_crates() -> Result<()> {
    run_checks()?;

    let crates = [
        "as_command_derive",
        "egui_mobius_macros",
        "egui_mobius_widgets",
        "egui_mobius",
    ];

    for crate_name in crates {
        println!("Publishing {}...", crate_name);
        Command::new("cargo")
            .current_dir(format!("crates/{}", crate_name))
            .args(["publish"])
            .status()?;
        
        // Wait for crates.io to update
        std::thread::sleep(std::time::Duration::from_secs(30));
    }

    Ok(())
}

fn bump_version(new_version: &str) -> Result<()> {
    Version::parse(new_version)?; // Validate version string

    for entry in WalkDir::new("crates").min_depth(1).max_depth(2).into_iter() {
        let entry = entry?;
        if entry.file_name() == "Cargo.toml" {
            let content = fs::read_to_string(entry.path())?;
            let mut doc = content.parse::<toml::Document>()?;
            
            if let Some(package) = doc.get_mut("package") {
                if let Some(version) = package.get_mut("version") {
                    *version = toml::Value::String(new_version.to_string());
                }
            }
            
            fs::write(entry.path(), doc.to_string())?;
        }
    }

    // Update README badge
    let readme_path = "README.md";
    let content = fs::read_to_string(readme_path)?;
    let updated = content.replace(
        &format!("version-{}-green", get_version("crates/egui_mobius/Cargo.toml")?),
        &format!("version-{}-green", new_version),
    );
    fs::write(readme_path, updated)?;

    // Add CHANGELOG entry
    let changelog_path = "CHANGELOG.md";
    let content = fs::read_to_string(changelog_path)?;
    let date = chrono::Local::now().format("%Y-%m-%d");
    let new_entry = format!("\n## [{}] - {}\n\n### Added\n\n### Changed\n\n### Fixed\n", new_version, date);
    let updated = content.replace("# Changelog\n", &format!("# Changelog\n{}", new_entry));
    fs::write(changelog_path, updated)?;

    Ok(())
}

fn get_version(path: impl AsRef<std::path::Path>) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let doc = content.parse::<toml::Document>()?;
    
    doc.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| eyre!("No version found in Cargo.toml"))
}
