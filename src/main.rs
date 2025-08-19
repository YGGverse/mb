#[macro_use]
extern crate rocket;

mod config;
mod db;
mod feed;

use config::Config;
use db::{Db, Order, Sort};
use feed::Feed;
use plurify::Plurify;
use rocket::{
    State,
    form::Form,
    http::Status,
    response::{Redirect, content::RawXml},
    serde::Serialize,
};
use rocket_dyn_templates::{Template, context};

#[get("/?<search>&<page>&<token>")]
fn index(
    search: Option<&str>,
    page: Option<usize>,
    token: Option<&str>,
    db: &State<Db>,
    config: &State<Config>,
) -> Result<Template, Status> {
    if token.is_some_and(|t| t != config.token) {
        warn!("Invalid access token! Access denied.");
        return Err(Status::Forbidden);
    }
    let posts = db
        .posts(
            search,
            Some((Sort::Time, Order::Desc)),
            page.map(|p| if p > 0 { p - 1 } else { p } * config.limit),
            Some(config.limit),
        )
        .map_err(|e| {
            error!("DB read error: `{e}`");
            Status::InternalServerError
        })?;

    Ok(Template::render(
        "index",
        context! {
            meta_title: {
                let mut t = String::new();
                if let Some(q) = search && !q.is_empty() {
                    t.push_str(q);
                    t.push_str(S);
                    t.push_str("Search");
                    t.push_str(S)
                }
                if let Some(p) = page && p > 1 {
                    t.push_str(&format!("Page {p}"));
                    t.push_str(S)
                }
                t.push_str(&config.title);
                if let Some(ref description) = config.description
                        && page.is_none_or(|p| p == 1) && search.is_none_or(|q| q.is_empty()) {
                    t.push_str(S);
                    t.push_str(description)
                }
                t
            },
            title: &config.title,
            description: config.description.as_deref(),
            back: page.map(|p| uri!(index(search, if p > 2 { Some(p - 1) } else { None }, token))),
            next: if page.unwrap_or(1) * config.limit >= posts.total { None }
                    else { Some(uri!(index(search, Some(page.map_or(2, |p| p + 1)), token))) },
            pagination_totals: if posts.total > 0 { Some(format!(
                "Page {} / {} ({} {} total)",
                page.unwrap_or(1),
                (posts.total as f64 / config.limit as f64).ceil(),
                posts.total,
                posts.total.plurify(&["post", "posts", "posts"])
            )) } else { None },
            posts: posts.posts.into_iter().map(|p| Post {
                id: p.id,
                time: p.time().format(&config.time_format).to_string(),
                message: p.message,
                href: Href {
                    delete: token.map(|t| uri!(delete(p.id, t)).to_string()),
                    post: uri!(post(p.id, token)).to_string()
                }
            }).collect::<Vec<Post>>(),
            home: uri!(index(None::<&str>, None::<usize>, token)),
            version: env!("CARGO_PKG_VERSION"),
            search,
            token
        },
    ))
}

#[derive(FromForm)]
struct Submit {
    message: String,
    token: String,
}
#[post("/submit", data = "<input>")]
fn submit(input: Form<Submit>, db: &State<Db>, config: &State<Config>) -> Result<Redirect, Status> {
    if input.token != config.token {
        warn!("Invalid access token! Access denied.");
        return Err(Status::Forbidden);
    }
    let i = input.into_inner();
    db.submit(i.message).map_err(|e| {
        error!("DB write error: `{e}`");
        Status::InternalServerError
    })?;
    Ok(Redirect::to(uri!(index(
        None::<&str>,
        None::<usize>,
        Some(i.token),
    ))))
}

#[get("/delete?<id>&<token>")]
fn delete(
    id: i64,
    token: String,
    db: &State<Db>,
    config: &State<Config>,
) -> Result<Redirect, Status> {
    if token != config.token {
        warn!("Invalid access token! Access denied.");
        return Err(Status::Forbidden);
    }
    db.delete(id).map_err(|e| {
        error!("DB write error: `{e}`");
        Status::InternalServerError
    })?;
    Ok(Redirect::to(uri!(index(
        None::<&str>,
        None::<usize>,
        Some(token),
    ))))
}

#[get("/<id>?<token>")]
fn post(
    id: i64,
    token: Option<&str>,
    db: &State<Db>,
    config: &State<Config>,
) -> Result<Template, Status> {
    if token.is_some_and(|t| t != config.token) {
        warn!("Invalid access token! Access denied.");
        return Err(Status::Forbidden);
    }
    match db.post(id).map_err(|e| {
        error!("DB read error: `{e}`");
        Status::InternalServerError
    })? {
        Some(post) => {
            let time = post.time().format(&config.time_format).to_string();
            Ok(Template::render(
                "post",
                context! {
                    meta_title: format!("{time}{S}{}", &config.title),
                    title: &config.title,
                    description: config.description.as_deref(),
                    back: None::<&str>,
                    next: None::<&str>,
                    pagination_totals: None::<&str>,
                    post: Post {
                        id: post.id,
                        message: post.message,
                        href: Href {
                            delete: token.map(|t| uri!(delete(post.id, t)).to_string()),
                            post: uri!(post(post.id, token)).to_string()
                        },
                        time
                    },
                    home: uri!(index(None::<&str>, None::<usize>, token)),
                    version: env!("CARGO_PKG_VERSION"),
                    search: None::<&str>
                },
            ))
        }
        None => Err(Status::NotFound),
    }
}

#[get("/rss")]
fn rss(db: &State<Db>, config: &State<Config>) -> Result<RawXml<String>, Status> {
    let mut f = Feed::new(
        &config.title,
        config.description.as_deref(),
        config.url.clone(),
        1024, // @TODO
    );
    for p in db
        .posts(
            None,
            Some((Sort::Time, Order::Desc)),
            None,
            Some(config.limit),
        )
        .map_err(|e| {
            error!("DB read error: `{e}`");
            Status::InternalServerError
        })?
        .posts
    {
        let time = p.time();
        f.push(
            p.id,
            time,
            time.format(&config.time_format).to_string(),
            &p.message,
        )
    }
    Ok(RawXml(f.commit()))
}

#[launch]
fn rocket() -> _ {
    use clap::Parser;
    let config = config::Config::parse();
    if config.url.is_none() {
        warn!("Canonical URL option is required for the RSS feed by the specification!") // @TODO
    }
    rocket::build()
        .attach(Template::fairing())
        .configure(rocket::Config {
            port: config.port,
            address: config.host,
            ..if config.debug {
                rocket::Config::debug_default()
            } else {
                rocket::Config::release_default()
            }
        })
        .mount("/", rocket::fs::FileServer::from(&config.public))
        .mount("/", routes![index, post, submit, delete, rss])
        .manage(Db::init(&config.database).unwrap())
        .manage(config)
}

/// Meta title separator
const S: &str = " • ";

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Href {
    /// Reference to post delete action
    /// * optional as dependent of access permissions
    delete: Option<String>,
    /// Reference to post details page
    post: String,
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Post {
    id: i64,
    message: String,
    /// Time created
    /// * edit time should be implemented as the separated history table
    time: String,
    href: Href,
}
