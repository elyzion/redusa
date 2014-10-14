//! The is the main HTTP server for Redusa.

#![crate_name = "redusa"]
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate time;
extern crate http;
extern crate iron;
extern crate bodyparser;
extern crate router;
extern crate logger;
extern crate serialize;

use iron::{Chain, ChainBuilder, Iron, Request, Response, IronResult, Handler, IronError, Plugin};
use iron::status;
use bodyparser::{BodyParser};
use router::{Router, Params};
use logger::Logger;
use http::method::{Get, Post};
//use http::status;
use serialize::json;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;
use std::mem::transmute;
use std::ptr;

use level::LevelRepository;
use level::repository::Inited;

mod config;
mod level;
mod score;

#[deriving(Clone)]
struct RedusaServer;

fn main() {

    //TODO: Add chain middleware.
    //let mut auth_chain: Box<Chain + Send> = Chain::new();
    //auth_chain.link(FromFn::new(echo_to_term));
    
    let mut router = Router::new();
    // Get list of scores for a single level
    router.get(
        "/level/:level",
        get_level
    );
    // Add a score to a level
    router.post(
        "/level/:level",
        add_score
    );

    let (logger_before, logger_after) = Logger::new(None);
    //let mut server: Server = Iron::new();
    //server.chain.link(BodyParser::new());
    //server.chain.link(logger);
    //server.chain.link(router);
    //server.listen(Ipv4Addr(127, 0, 0, 1), 8001);
    let mut chain = ChainBuilder::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    //chain.link_before(ResponseTime);
    //chain.link_after(ResponseTime);
    Iron::new(chain).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}


fn get_level(req: &mut Request) -> IronResult<Response> {
    let evidence = level::repository::init();
    let score = level::repository::get(evidence).get_level_high_scores(1);
    match score { 
        Some(tree) => Ok(Response::with(status::Ok, "Hello world")),
        None => Ok(Response::with(status::Ok, "Hello world"))
    }
}

fn add_score(req: &mut Request) -> IronResult<Response> {
    req.get::<BodyParser>().map(|parsed| println!("Parsed Json:\n{}", parsed));
    let evidence = level::repository::init();
    level::repository::get(evidence).add_score(1, "1".to_string(), 1);
    Ok(Response::with(status::Ok, "Hello, world!"))
}




