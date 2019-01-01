#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::env;

use futures::{Future, Stream};
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio::runtime::Runtime;
use hubcaps::{Credentials, Error, Github, HttpCache, Result};
use hubcaps::repositories::*;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut rt = Runtime::new()?;

    let host = "https://api.github.com";
    let agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let credentials = env::var("GITHUB_TOKEN").ok().map(Credentials::Token);
    let client = Client::builder().build(HttpsConnector::new(4).unwrap());
    let http_cache = HttpCache::in_home_dir();
    let github: Github<_> = Github::custom(host, agent, credentials, client, http_cache);

    rt.block_on(github.repos().iter(&RepoListOptions::builder().affiliation(vec![Affiliation::Owner]).build()).filter(|r| r.fork).for_each(|r| {
        println!("{}", r.full_name);
        Ok(())
    }))?;

    Ok(())
}
