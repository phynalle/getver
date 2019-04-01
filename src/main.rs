#![feature(async_await, await_macro, futures_api)]
#![recursion_limit = "128"]

use std::env::args;

use colored::Colorize;
use failure::Fail;
use futures::{compat::Future01CompatExt, future::join_all, FutureExt, TryFutureExt};
use log::debug;
use reqwest::r#async::Client;
use serde_derive::Deserialize;
use tokio::prelude::Future;
use tokio_threadpool::ThreadPool;
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
    let mut res = await!(Client::new().get(&url).send().compat())?;
    debug!("url = {}, status = {}", url, res.status());
    if res.status().as_u16() == 404 {
        return Err(Error::CrateNotFound(crate_name));
    }
    let crate_res: CrateResponse = await!(res.json().compat())?;
    Ok(crate_res.krate)
}

async fn run_async() {
    let mut not_founds = Vec::new();

    let crates = join_all(args().skip(1).map(get_crate_info));
    for krate in await!(crates) {
        match krate {
            Ok(krate) => println!("{}: {}", krate.name.blue(), krate.max_version.yellow()),
            Err(Error::CrateNotFound(name)) => not_founds.push(name),
            Err(e) => println!("{}", e.to_string().red()),
        };
    }

    if !not_founds.is_empty() {
        println!("{} {}", "Crate Not Found:".red(), not_founds.join(", "));
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

fn main() {
    parse_argument();
    let fut = run_async().unit_error().boxed().compat();
    let pool = ThreadPool::new();
    pool.spawn(fut);
    pool.shutdown_on_idle().wait().unwrap();
}
