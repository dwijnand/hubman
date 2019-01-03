#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::env;

use futures::{Future, Stream};
use hubcaps::branches::Branch;
use hubcaps::repositories::*;
use hubcaps::{Credentials, Error, Github, HttpCache, Result};
use hyper::Client;
use hyper_tls::HttpsConnector;

fn main() -> Result<()> {
    pretty_env_logger::init();

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

    my_forks.fold((), move |(), r| {
        print!("https://github.com/{}/branches: ", r.full_name);

        let credentials = env::var("GITHUB_TOKEN").ok().map(Credentials::Token);
        let client = Client::builder().build(HttpsConnector::new(4).unwrap());
        let http_cache = HttpCache::in_home_dir();
        let github: Github<_> = Github::custom(host, agent, credentials, client, http_cache);
        fn per_branch(b: Branch) {
            let protected = match b.protected {
                Some(true) => " (protected)",
                Some(false) => "",
                None => ""
            };
            print!("{}{}, ", b.name, protected);
        }
        github.repo(r.owner.login, r.name).branches().iter()
            .fold((), |(), b| {
                per_branch(b);
                futures::future::ok::<(), Error>(())
            })
            .and_then(|()| {
                println!();
                futures::future::ok::<(), Error>(())
            })
    }).run();

    Ok(())
}

trait AsyncRun {
    fn run(self);
}

impl<F> AsyncRun for F where
    F: Future<Item = (), Error = Error> + Send + 'static
{
    fn run(self) {
        tokio::run(self.map_err(|err| {
            eprintln!("Error: {}", err);
            ()
        }))
    }
}
