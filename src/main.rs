//! The is the main HTTP server for Redusa.

#![crate_name = "redusa"]
#![allow(unused_imports)]
#![allow(dead_code)]

//extern crate time;
//extern crate http;
extern crate iron;
extern crate bodyparser;
extern crate persistent;
extern crate router;
extern crate logger;
extern crate env_logger;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use iron::prelude::*;
use iron::status;
use router::{Router, Params};
use logger::Logger;
use persistent::Read;

use level::LevelRepository;
use level::repository::Inited;

mod config;
mod level;
mod score;

#[derive(Clone)]
struct RedusaServer;

fn main() {
    env_logger::init().unwrap();

    //TODO: Add chain middleware.
    //let mut auth_chain: Box<Chain + Send> = Chain::new();
    //auth_chain.link(FromFn::new(echo_to_term));

    // Init logger
    let (logger_before, logger_after) = Logger::new(None);
    
    let mut router = Router::new();
    // Get list of scores for a single level
    router.get("/level/:level", get_level, "get_level");
    // Add a score to a level
    router.post("/level/:level", add_score, "add_score");

    let mut chain = Chain::new(router);
    chain.link_before(Read::<bodyparser::MaxBodyLength>::one(config::MAX_BODY_LENGTH));
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    Iron::new(chain).http("127.0.0.1:3000").unwrap();
}


fn get_level(req: &mut Request) -> IronResult<Response> {
    let evidence = level::repository::init();
    let score = level::repository::get(evidence).get_level_high_scores(1);
    match score { 
        Some(tree) => Ok(Response::with("Hello world")),
        None => Ok(Response::with("Hello world"))
    }
}

fn add_score(req: &mut Request) -> IronResult<Response> {
    req.get::<bodyparser::Json>().map(|parsed| println!("Parsed Json:\n{:?}", parsed));
    let evidence = level::repository::init();
    level::repository::get(evidence).add_score(1, "1".to_string(), 1);
    Ok(Response::with("Hello, world!"))
}




