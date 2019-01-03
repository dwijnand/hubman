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
use hyper::client::connect::Connect;
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio::runtime::Runtime;

fn github() -> Github<impl Clone + Connect + 'static> {
    let host = "https://api.github.com";
    let agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let credentials = env::var("GITHUB_TOKEN").ok().map(Credentials::Token);
    let client = Client::builder().build(HttpsConnector::new(4).unwrap());
    let http_cache = HttpCache::in_home_dir();
    Github::custom(host, agent, credentials, client, http_cache)
}

fn ok<T>(x: T) -> impl Future<Item = T, Error = Error> {
    future::ok::<T, Error>(x)
}

fn box_ok<T: Send + 'static>(x: T) -> Box<Future<Item = T, Error = Error> + Send> {
    Box::new(ok(x))
}


fn main() -> Result<()> {
    pretty_env_logger::init();
    let ref mut rt = Runtime::new()?;

    let my_forks = github()
        .repos()
        .iter(
            &RepoListOptions::builder()
                .affiliation(vec![Affiliation::Owner])
                .build(),
        )
        .filter(|r| r.fork);

    let my_forks_with_branches = my_forks.and_then(move |r| {
        let branches = github()
            .repo(r.owner.login.deref(), r.name.deref())
            .branches()
            .iter();
        branches.collect().map(move |branches| (r, branches))
    });

    my_forks_with_branches.and_then(move |(r, bs)| {
        fn is_protected(b: &Branch) -> bool {
            b.protected == Some(true)
        }
        fn per_branch(b: &Branch) {
            let protected = if is_protected(b) { " (protected)" } else { "" };
            print!("{}{}, ", b.name, protected);
        };
        if bs.len() == 1 && {
            let b = &bs[0];
            !is_protected(&b) && ["master", "z"].iter().any(|n| n == &b.name)
        } {
            println!("DELETING https://github.com/{}", r.full_name);
            github().repo(r.owner.login.deref(), r.name.deref()).delete()
        } else {
            print!("https://github.com/{}/branches: {} ", r.full_name, bs.len());
            bs.iter().for_each(|b| per_branch(b));
            println!();
            box_ok(())
        }
    }).run(rt)
}

trait AsyncRun {
    fn run(self, rt: &mut Runtime) -> Result<()>;
}
trait AsyncRunStream {
    fn run(self, rt: &mut Runtime) -> Result<()>;
}

impl<F> AsyncRun for F
where
    F: Future<Item = (), Error = Error> + Send + 'static,
{
    fn run(self, rt: &mut Runtime) -> Result<()> {
        rt.block_on(self)
    }
}

impl<S> AsyncRunStream for S
where
    S: Stream<Item = (), Error = Error> + Send + 'static,
{
    fn run(self, rt: &mut Runtime) -> Result<()> {
        self.for_each(|()| Ok(())).run(rt)
    }
}
