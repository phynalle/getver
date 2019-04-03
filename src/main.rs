use std::env::args;
use std::thread;

use colored::Colorize;
use curl::{easy::Easy, Error as CurlError};
use failure::Fail;
use log::debug;
use serde_derive::Deserialize;
use serde_json::Error as SerdeError;
use version::version;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "the crate {} doesn't exist", _0)]
    CrateNotFound(String),
    #[fail(display = "{}", _0)]
    Io(CurlError),
    #[fail(display = "{}", _0)]
    Serde(SerdeError),
}

impl From<CurlError> for Error {
    fn from(e: CurlError) -> Error {
        Error::Io(e)
    }
}

impl From<SerdeError> for Error {
    fn from(e: SerdeError) -> Error {
        Error::Serde(e)
    }
}

#[derive(Deserialize)]
pub struct Crate {
    pub name: String,
    pub max_version: String,
}

fn get_crate_info(crate_name: String) -> Result<Crate, Error> {
    #[derive(Deserialize)]
    struct CrateResponse {
        #[serde(rename = "crate")]
        krate: Option<Crate>,
    };

    let mut buf = Vec::new();
    let mut handle = Easy::new();
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
    handle.url(&url)?;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            buf.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    let res: CrateResponse = serde_json::from_slice(&buf)?;
    res.krate.ok_or_else(|| Error::CrateNotFound(crate_name))
}

fn run() {
    let (tx, rx) = spmc::channel::<String>();
    let num_args = args().len();

    let mut threads = Vec::new();
    for _ in 0..num_args {
        let rx = rx.clone();
        let t = thread::spawn(move || {
            while let Ok(crate_name) = rx.recv() {
                match get_crate_info(crate_name) {
                    Ok(krate) => println!("{}: {}", krate.name.blue(), krate.max_version.yellow()),
                    Err(e) => {
                        debug!("error: {}", e);
                        println!("{}", e.to_string().red());
                    }
                }
            }
        });
        threads.push(t);
    }

    for arg in args().skip(1) {
        tx.send(arg).unwrap();
    }
    drop(tx);

    for t in threads {
        t.join().unwrap();
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

fn main() {
    parse_argument();
    run();
}
