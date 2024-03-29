use arg_parse::Args;
use error::Error;
use error::ErrorType;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;

mod arg_parse;
mod error;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Config {
    Workspace { workspace: MetadataSection },
    Package { package: MetadataSection },
}

#[derive(Deserialize, Debug)]
struct MetadataSection {
    metadata: Metadata,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    scripts: HashMap<String, String>,
}

impl Metadata {
    fn print_script_names(&self) {
        self.scripts
            .keys()
            .for_each(|script_name| println!("{}", script_name));
    }
}

fn main() -> Result<(), Error> {
    let metadata = parse_toml_file("Cargo.toml")?;

    let args = arg_parse::parse(env::args().collect()).or_else(|err| {
        metadata.print_script_names();
        Err(err)
    })?;

    match metadata.scripts.get(&args.script_name) {
        Some(script) => run_script(script.clone(), args),
        None => {
            metadata.print_script_names();
            Err(Error::new(
                ErrorType::InvalidScriptName,
                "script name is invalid",
            ))
        }
    }
}

fn parse_toml_file(file_path: &str) -> Result<Metadata, Error> {
    let mut f = File::open(file_path).unwrap_or_else(|_| panic!("{} file not found.", file_path));

    let mut toml = String::new();
    f.read_to_string(&mut toml).or_else(|_| {
        Err(Error::new(
            ErrorType::NoToml,
            format!("Failed to read {}", file_path),
        ))
    })?;

    let config: Config = toml::from_str(&toml).or_else(|_| {
        Err(Error::new(
            ErrorType::NoScriptInfo,
            "toml file does not contain package.metadata.scripts or workspace.metadata.scripts table"
        ))
    })?;

    match config {
        Config::Workspace { workspace } => Ok(workspace.metadata),
        Config::Package { package } => Ok(package.metadata),
    }
}

fn run_script(script: String, args: Args) -> Result<(), Error> {
    let mut shell = if cfg!(target_os = "windows") {
        let mut shell = Command::new("cmd");
        shell.arg("/C");

        shell
    } else {
        let mut shell = Command::new("sh");
        shell.arg("-c");

        shell
    };

    let mut modified_script = script.replace("$0", &args.binary_path);
    args.script_arguments
        .iter()
        .enumerate()
        .for_each(|(index, arg)| {
            let replace_target = "$".to_owned() + (index + 1).to_string().as_str();
            modified_script = modified_script.replace(&replace_target, arg)
        });

    let mut child = shell
        .arg(modified_script)
        .spawn()
        .expect("spawning script failed");

    let exit_status = child.wait().expect("script wasn't running");
    return Error::parse_exit_status(exit_status);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workspace_toml() {
        let result = parse_toml_file("test-files/workspace-cargo.toml").unwrap();
        assert!(result.scripts.contains_key("hello"));
        assert!(result.scripts.contains_key("goodbye"));
    }

    #[test]
    fn test_parse_package_toml() {
        let result = parse_toml_file("test-files/package-cargo.toml").unwrap();
        assert!(result.scripts.contains_key("hello"));
        assert!(result.scripts.contains_key("goodbye"));
    }

    #[test]
    fn test_parse_no_script_info() {
        let error = parse_toml_file("test-files/no-script-info-cargo.toml").unwrap_err();
        assert_eq!(error, Error::new(ErrorType::NoScriptInfo, ""));
    }
}
