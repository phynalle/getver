use std::collections::HashSet;
use std::env::args;

use colored::Colorize;
use failure::Fail;
use futures::future::join_all;
use serde_derive::Deserialize;
use version::version;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "the crate '{}' doesn't exist", _0)]
    CrateNotFound(String),
    #[fail(display = "{}", _0)]
    Io(std::io::Error),
    #[fail(display = "{}", _0)]
    Json(serde_json::Error),
    #[fail(display = "{}", _0)]
    Surf(surf::Exception),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<surf::Exception> for Error {
    fn from(err: surf::Exception) -> Self {
        Error::Surf(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
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
    join_all(crate_names.into_iter().map(fetch_version)).await;
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
