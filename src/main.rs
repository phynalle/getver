use std::collections::HashSet;
use std::env::args;

use colored::Colorize;
use runtime::spawn;
use serde::Deserialize;
use thiserror::Error as ThisError;
use version::version;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("the crate '{0}' doesn't exist")]
    CrateNotFound(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Surf(#[from] surf::Exception),
}

#[derive(Deserialize)]
pub struct Crate {
    pub name: String,
    pub max_version: String,
}

async fn get_crate_info(crate_name: String) -> Result<Crate, Error> {
    #[derive(Deserialize)]
    struct CrateResponse {
        #[serde(rename = "crate")]
        krate: Crate,
    };

    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
    let mut res = surf::get(&url).await?;
    if res.status().is_success() {
        let CrateResponse { krate } = res.body_json().await?;
        Ok(krate)
    } else {
        Err(Error::CrateNotFound(crate_name))
    }
}

async fn fetch_version(crate_name: String) {
    match get_crate_info(crate_name).await {
        Ok(krate) => println!("{}: {}", krate.name.blue(), krate.max_version.yellow()),
        Err(e) => println!("{}", format!("{}", e).red()),
    }
}

async fn run(crate_names: HashSet<String>) {
    let futures: Vec<_> = crate_names
        .into_iter()
        .map(|name| spawn(fetch_version(name)))
        .collect();
    for fut in futures {
        fut.await;
    }
}

fn print_help_message() {
    let info = format!(
        r#"
{} {}

{}:
    getver [options] crate...
{}:
    -h, --help      Prints version information
{}:
    crate...        the name of crate
"#,
        "getver".blue(),
        version!(),
        "usage".green(),
        "options".green(),
        "arguments".green(),
    );
    println!("{}", info);
}

fn parse_argument() {
    if let Some(arg) = args().nth(2) {
        if arg.starts_with('-') {
            if arg == "-h" || arg == "--help" {
                print_help_message()
            } else {
                println!(
                    r#"{}: Found argument '{}' which wasn't expected

{}: getver [options] crate..."#,
                    "error".red().bold(),
                    arg.red(),
                    "usage".green(),
                );
            }
            ::std::process::exit(1)
        }
    }
}

#[runtime::main]
async fn main() {
    parse_argument();
    run(args().skip(1).collect()).await;
}
