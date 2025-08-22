use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};
use url::Url;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Access token to create and remove posts
    #[arg(long, short)]
    pub token: String,

    /// Path to the [redb](https://www.redb.org)
    /// * if the given path does not exist, a new database will be created
    #[arg(long, default_value_t = String::from("./mb.redb"))]
    pub database: String,

    /// Path to the public directory (which contains the CSS theme and other multimedia files)
    #[arg(long, default_value_t = String::from("./public"))]
    pub public: String,

    /// Server name
    #[arg(long, default_value_t = String::from("mb"))]
    pub title: String,

    /// Server description
    #[arg(long)]
    pub description: Option<String>,

    /// Canonical URL
    #[arg(long, short)]
    pub url: Option<Url>,

    /// Format timestamps (on the web view)
    ///
    /// * tip: escape with `%%d/%%m/%%Y %%H:%%M` in the CLI/bash argument
    #[arg(long, default_value_t = String::from("%d/%m/%Y %H:%M"))]
    pub time_format: String,

    /// Default listing limit
    #[arg(long, short, default_value_t = 20)]
    pub limit: usize,

    /// Default capacity (estimated torrents in the `public` directory)
    #[arg(long, default_value_t = 1000)]
    pub capacity: usize,

    /// Bind server on given host
    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
    pub host: IpAddr,

    /// Bind server on given port
    #[arg(long, default_value_t = 8000)]
    pub port: u16,

    /// Configure instance in the debug mode
    #[arg(long, default_value_t = false)]
    pub debug: bool,

    /// Limit post max length to `n` chars (JS-less, `maxlength` attribute)
    #[arg(long)]
    pub post_max_chars: Option<usize>,
}
