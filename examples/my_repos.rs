use hubcaps::{Github, HttpCache, Result};
use hyper::Client;
use hyper::rt::Stream;
use hyper_tls::HttpsConnector;
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut rt = Runtime::new()?;

    let host = "https://api.github.com";
    let agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let credentials = None;
    let client = Client::builder().build(HttpsConnector::new(4).unwrap());
    let http_cache = HttpCache::in_home_dir();
    let github = Github::custom(host, agent, credentials, client, http_cache);

    rt.block_on(
        github
            .user_repos("dwijnand")
            .iter(&Default::default())
            .for_each(move |repo| {
                println!("{}", repo.name);
                Ok(())
            }),
    )?;

    Ok(())
}
