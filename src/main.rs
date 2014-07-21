//! The is the main HTTP server for Redusa.

#![crate_name = "redusa#0.1"]
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate time;
extern crate http;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;

use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{Star, AbsoluteUri, AbsolutePath, Authority};
use http::status::{BadRequest, MethodNotAllowed};
use http::method::{Get, Head, Post, Put, Delete, Trace, Options, Connect, Patch};
use http::headers::content_type::MediaType;

mod config;
mod level;
mod score;


#[deriving(Clone)]
struct RedusaServer;

impl Server for RedusaServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 } }
    }

    fn handle_request(&self, r: Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());

        //Check
        match (&r.method, &r.request_uri) {
            (&Connect, _) => {
                w.status = MethodNotAllowed;
                return
            },
            (_, &Authority(_)) => {
                w.status = BadRequest;
                return
            },
            (&Options, &Star) => {
                w.headers.allow = Some(vec!(Get, Post, Put, Delete));
                w.headers.content_length = Some(0);
                return;
            },            
            (&Options, &AbsoluteUri(_)) | (&Options, &AbsolutePath(_)) => {
            },
            (_, &AbsoluteUri(_)) | (_, &AbsolutePath(_)) => {
            },
            (_, &Star) => {
            }
        }
        //TODO: QueryFilter
        //TODO: BodyFilter

        //Default header
        w.headers.content_type = Some(MediaType {
            type_: String::from_str("text"),
            subtype: String::from_str("plain"),
            parameters: vec!((String::from_str("charset"), String::from_str("UTF-8")))
        });

        w.write(b"Hello, World!\n").unwrap();

        match r.request_uri {
            Star | Authority(_) => {
                w.status = BadRequest;
                // Actually, valid for the CONNECT method.
            },
            AbsoluteUri(ref url) => {
                println!("absoluteURI, {}", url);
                //path =
            },
            AbsolutePath(ref url) => {
                println!("absolute path, {}", url);
                //w.status = a
            },
        }

    }
}

///TODO: Iron based implementation
fn main() {
    RedusaServer.serve_forever();
}

struct HttpExchange<'a> {
    request: &'a Request,
    reponse: String,
}


