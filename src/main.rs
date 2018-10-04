#![recursion_limit="128"]

#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;

extern crate tokio;
extern crate reqwest;
extern crate futures;

extern crate colored;

use std::env::args;

use reqwest::async::Client;
use futures::Future;
use colored::*;

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


fn get_crate_info(crate_name: &str) -> impl Future<Item=Crate, Error=Error> {
    #[derive(Deserialize)]
    struct CrateResponse {
        #[serde(rename="crate")]
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
        .and_then(|mut res| {
            res.json::<CrateResponse>()
        })
        .and_then(|crate_res| {
            Ok(crate_res.krate)
        })
        .map_err(|e| {
            debug!("error: {}", e);
            Error::CrateNotFound
        })
}

fn run_async() {

    for arg in args().skip(1) {
        tokio::spawn(get_crate_info(&arg)
            .and_then(|krate| {
                println!("{}: {} {}", krate.name.blue(), krate.max_version, "(latest)".green());
                Ok(())
            })
            .map_err(move |e| {
                debug!("error: {}", e);
                println!("{}", format!("the crate '{}' doesn't exist", arg).red());
            }));
    }
}

fn main() {
    env_logger::init();
    let fut = ::futures::lazy(|| Ok(run_async()));
    tokio::run(fut);
}
