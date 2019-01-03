#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::env;
use std::ops::Deref;

use futures::prelude::*;
use futures::{future, stream};
use hubcaps::branches::Branch;
use hubcaps::repositories::*;
use hubcaps::{Credentials, Error, Github, HttpCache, Result};
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let ref mut rt = Runtime::new()?;

    let host = "https://api.github.com";
    let agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let credentials = env::var("GITHUB_TOKEN").ok().map(Credentials::Token);
    let client = Client::builder().build(HttpsConnector::new(4).unwrap());
    let http_cache = HttpCache::in_home_dir();
    let github: Github<_> = Github::custom(host, agent, credentials, client, http_cache);

    let my_forks = github
        .repos()
        .iter(
            &RepoListOptions::builder()
                .affiliation(vec![Affiliation::Owner])
                .build(),
        )
        .filter(|r| r.fork);

    let my_forks_with_branches = my_forks.and_then(move |r| {
        let credentials = env::var("GITHUB_TOKEN").ok().map(Credentials::Token);
        let client = Client::builder().build(HttpsConnector::new(4).unwrap());
        let http_cache = HttpCache::in_home_dir();
        let github: Github<_> = Github::custom(host, agent, credentials, client, http_cache);
        let branches = github.repo(r.owner.login.deref(), r.name.deref()).branches().iter();
        branches.collect().map(move |branches| (r, branches))
    });

    my_forks_with_branches.for_each(|(r, bs)| {
        print!("https://github.com/{}/branches: ", r.full_name);
        fn per_branch(b: &Branch) {
            let protected = match b.protected {
                Some(true) => " (protected)",
                Some(false) => "",
                None => ""
            };
            print!("{}{}, ", b.name, protected);
        };
        bs.iter().for_each(|b| per_branch(b));
        println!();
        Ok(())
    }).run(rt)
}

trait AsyncRun {
    fn run(self, rt: &mut Runtime) -> Result<()>;
}

impl<F> AsyncRun for F where
    F: Future<Item = (), Error = Error> + Send + 'static
{
    fn run(self, rt: &mut Runtime) -> Result<()> {
        rt.block_on(self)
    }
}
