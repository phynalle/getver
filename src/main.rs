use std::env::args;

use colored::Colorize;
use failure::Fail;
use futures::Future;
use log::debug;
use reqwest::r#async::Client;
use serde_derive::Deserialize;
use version::version;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "the crate doesn't exist")]
    CrateNotFound,
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

fn get_crate_info(crate_name: &str) -> impl Future<Item = Crate, Error = Error> {
    #[derive(Deserialize)]
    struct CrateResponse {
        #[serde(rename = "crate")]
        krate: Crate,
    };

    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
    Client::new()
        .get(&url)
        .send()
        .and_then(move |res| {
            debug!("url = {}, status = {}", url, res.status());
            res.error_for_status()
        })
        .and_then(|mut res| res.json::<CrateResponse>())
        .and_then(|crate_res| Ok(crate_res.krate))
        .map_err(|e| {
            debug!("error: {}", e);
            Error::CrateNotFound
        })
}

fn run_async() {
    for arg in args().skip(1) {
        tokio::spawn(
            get_crate_info(&arg)
                .and_then(|krate| {
                    println!("{}: {}", krate.name.blue(), krate.max_version.yellow(),);
                    Ok(())
                })
                .map_err(move |e| {
                    debug!("error: {}", e);
                    println!("{}", format!("the crate '{}' doesn't exist", arg).red());
                }),
        );
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
    if let Some(arg) = args().skip(1).next() {
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

fn main() {
    env_logger::init();
    parse_argument();
    tokio::run(::futures::lazy(|| Ok(run_async())));
}
