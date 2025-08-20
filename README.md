# mb

![Build](https://github.com/YGGverse/mb/actions/workflows/build.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/mb/status.svg)](https://deps.rs/repo/github/YGGverse/mb)
[![crates.io](https://img.shields.io/crates/v/mb.svg)](https://crates.io/crates/mb)

Simple, js-less micro-blogging platform written in Rust.

It uses the [Rocket](https://rocket.rs) framework and the [redb](https://www.redb.org) database for serving messages.

## Demo

![mb v0.1.1 index page](https://github.com/user-attachments/assets/07b49ecd-fde7-460a-8702-34f30fb116ba)

## Install

1. `git clone https://github.com/YGGverse/mb.git && cd mb`
2. `cargo build --release`
3. `sudo install target/release/mb /usr/local/bin/mb`

## Usage

### systemd

``` /etc/systemd/system/mb.service
[Unit]
After=network.target
Wants=network.target

[Service]
Type=simple
User=mb
Group=mb
WorkingDirectory=/path/to/public-and-templates
ExecStart=/usr/local/bin/mb --token=strong_key
StandardOutput=file:///path/to/debug.log
StandardError=file:///path/to/error.log

[Install]
WantedBy=multi-user.target
```
* the `database` file will be created if it does not already exist at the given location
* the `token` value is the access key to create and delete your messages (the authentication feature has not yet been implemented)
* copy `templates` and `public` folders to `WorkingDirectory` destination (see [Rocket deployment](https://rocket.rs/guide/v0.5/deploying/#deploying) for details)

### nginx

``` default
server {
    listen 80;

    location / {
        # expires 15m;
        # add_header Cache-Control "public, max-age=900";
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```
