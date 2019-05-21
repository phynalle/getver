#![feature(async_await, await_macro)]
#![recursion_limit = "128"]

use colored::Colorize;
use failure::Fail;
use futures::{compat::Future01CompatExt, future::join_all};
use reqwest::r#async::Client;
use serde_derive::Deserialize;
use std::env::args;
use version::version;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "the crate doesn't exist")]
    CrateNotFound(String),
    #[fail(display = "{}", _0)]
    Io(reqwest::Error),
}

impl From<::reqwest::Error> for Error {
    fn from(err: ::reqwest::Error) -> Self {
        Error::Io(err)
    }
}

#[derive(Deserialize)]
pub struct Crate {
    pub name: String,
    pub max_version: String,
}

async fn get_crate_info(crate_name: String) -> Result<Crate> {
    #[derive(Deserialize)]
    struct CrateResponse {
        #[serde(rename = "crate")]
        krate: Crate,
    };

    let url = format!("https://crates.io/api/v1/crates/{}", &crate_name);
    let mut res = Client::new().get(&url).send().compat().await?;
    if res.status().as_u16() == 404 {
        return Err(Error::CrateNotFound(crate_name));
    }
    let crate_res: CrateResponse = res.json().compat().await?;
    Ok(crate_res.krate)
}

async fn run_async() {
    let crates = join_all(args().skip(1).map(get_crate_info));
    for krate in crates.await {
        match krate {
            Ok(krate) => println!("{}: {}", krate.name.blue(), krate.max_version.yellow()),
            Err(Error::CrateNotFound(name)) => {
                println!("{}", format!("the crate '{}' doesn't exist", name).red())
            }
            Err(e) => println!("{}", e.to_string().red()),
        };
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
            std::process::exit(1)
        }
    }
}

#[runtime::main(runtime_tokio::Tokio)]
async fn main() {
    parse_argument();
    run_async().await
}
