use serde::Deserialize;
use std::fs;
use std::process::Command;
use std::string::String;
use toml;

use chroma::utils::str::{quote, to_macro_name};

/// A versioned package (refer to example in `/examples/hello-chroma`)
#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
    edition: String,
}

/// Binary file type (i.e., an executable target)
#[derive(Deserialize, Debug)]
struct Bin {
    name: String,
    path: String,
}

/// Type representing a configuration file with information about a package
#[derive(Deserialize, Debug)]
struct Config {
    package: Package,
    bin: Vec<Bin>,
}

/// Get macro definitions from config file
fn get_definitions(cfg: &Config) -> Vec<(String, String)> {
    let name = to_macro_name(cfg.package.name.as_str());

    let tag = |x| name.clone() + x;

    let version = &cfg.package.version;
    let mut definitions = vec![
        (name.clone(), "1".into()),
        (tag("_VERSION"), quote(&cfg.package.version)),
    ];

    let arr = ["_MAJOR", "_MINOR", "_PATCH"];
    for (i, item) in version.split('.').take(3).enumerate() {
        definitions.push((tag(arr[i]), item.into()))
    }
    definitions
}

/// Incorporate a list of definitions (see `get_definitions`) into the arguments used to build the
/// target file(s)
fn append_defs(args: &mut Vec<String>, defs: &Vec<(String, String)>) {
    for (name, body) in defs {
        args.push(format!("-D{name}={body}"))
    }
}

fn main() -> chroma::Result<()> {
    chroma::utils::fs::find_project_root()?.cd_to_root()?;

    // Load config from toml file (modeled after Cargo config files; see
    // `/examples/hello-chroma/chroma.toml` for an example)
    let contents = fs::read_to_string("chroma.toml")?;

    let config: Config = toml::from_str(&contents)?;

    let definitions = get_definitions(&config);
    println!("{:#?}", get_definitions(&config));

    println!("{:#?}", config);

    // Create directory to store build target(s)
    std::fs::create_dir_all("build").expect("Couldn't create build/ directory");
    for app in &config.bin {
        let mut args = vec![];
        append_defs(&mut args, &definitions);
        args.push("-o".into());
        args.push("build/".to_string() + app.name.as_str());
        // Include relevant standard library for language standard designated in config.toml
        args.push(format!("-std={}", config.package.edition));
        args.push(app.path.clone());
        // Execute the actual compilation command and catch any errors generated
        if !Command::new("g++").args(args).spawn()?.wait()?.success() {
            eprintln!("Error when compiling {}", app.name);
            std::process::exit(1)
        }
    }

    Ok(())
}
