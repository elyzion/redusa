//! The is the main HTTP server for Redusa.

#![crate_name = "redusa"]
#![allow(unused_imports)]
#![allow(dead_code)]

//extern crate time;
//extern crate http;
extern crate iron;
extern crate iron_test;
extern crate bodyparser;
extern crate persistent;
extern crate router;
extern crate logger;
extern crate env_logger;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate mime;

use iron::prelude::*;
use iron::status;
use iron::mime::*;

use router::{Router, Params};
use logger::Logger;
use persistent::Read;

use std::str::FromStr;

use level::LevelRepository;
use level::repository::Inited;

mod config;
mod level;
mod score;

fn main() {
    env_logger::init().unwrap();

    // Init logger
    let (logger_before, logger_after) = Logger::new(None);
    
    let mut chain = Chain::new(app_router());
    chain.link_before(Read::<bodyparser::MaxBodyLength>::one(config::MAX_BODY_LENGTH));
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    Iron::new(chain).http("127.0.0.1:3000").unwrap();
}

fn app_router() -> Router {
    let mut router = Router::new();
    // Get list of scores for a single level
    router.get("/level/:level", get_level, "get_level");
    // Add a score to a level
    router.post("/level/:level", add_score, "add_score");
    router
}

fn get_level(req: &mut Request) -> IronResult<Response> {
    let evidence = level::repository::init();
    let level = req.extensions.get::<Router>().unwrap().find("level").unwrap_or("/");
    let score = level::repository::get(evidence).get_level_high_scores(u64::from_str(level).unwrap());
    let js = serde_json::to_string(&score).unwrap();
    match score { 
        Some(tree) => Ok(Response::with((mime!(Application/Json), status::Ok, js))),
        None => Ok(Response::with((mime!(Application/Json), status::Ok, "{}")))
    }
}

fn add_score(req: &mut Request) -> IronResult<Response> {
    let body = req.get::<bodyparser::Struct<PostPoints>>().unwrap().unwrap();
    let level = req.extensions.get::<Router>().unwrap().find("level").unwrap_or("/");
    // TODO: rewrite repository handling and init to modern rust..
    let evidence = level::repository::init();
    // level, user, points
    level::repository::get(evidence).add_score(u64::from_str(level).unwrap(), body.user_id, body.points);
    Ok(Response::with(status::Accepted))
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PostPoints {
    user_id:    String,
    points:     u64
}

#[cfg(test)]
mod test {
    extern crate serde;
    extern crate serde_json;

    use std::collections::{BTreeSet,BTreeMap};
    use std::str::from_utf8;
    use iron::*;
    use iron::status;
    use iron::headers::ContentType;
    use iron::prelude::*;
    use iron_test::{request, response};
    use mime::Mime;
    use score::Score;
    use super::{PostPoints, get_level, add_score, app_router};

    #[test]
    fn test_get_empty_level() {
        let response = request::get("http://localhost:3000/level/1",
                                    Headers::new(),
                                    &app_router()).unwrap();
        let result_body = response::extract_body_to_bytes(response);

        assert_eq!(result_body, b"{}");
    }
    
    #[test]
    fn test_post_score() {
        let mut headers = Headers::new();
        headers.set(ContentType(mime!(Application/Json)));
        let response = request::post("http://localhost:3000/level/1",
                                    headers,
                                    "{ \"user_id\": \"3\", \"points\": 30 }",                    
                                    &app_router()).unwrap();
        assert_eq!(response.status.as_ref().unwrap(), &status::Accepted);

        let result_body = response::extract_body_to_bytes(response);
        assert_eq!(result_body, b"");
    }

    #[test]
    fn test_get_and_post() {
        {
            let response = request::get("http://localhost:3000/level/1",
                                        Headers::new(),
                                        &app_router()).unwrap();
            let result_body = response::extract_body_to_bytes(response);

            assert_eq!(result_body, b"{}");
        }
        {
            let mut headers = Headers::new();
            headers.set(ContentType(mime!(Application/Json)));
            let response = request::post("http://localhost:3000/level/1",
                                        headers,
                                        "{ \"user_id\": \"3\", \"points\": 30 }",                    
                                        &app_router()).unwrap();
            assert_eq!(response.status.as_ref().unwrap(), &status::Accepted);

            let result_body = response::extract_body_to_bytes(response);
            assert_eq!(result_body, b"");
        }
        {
            let response = request::get("http://localhost:3000/level/1",
                                        Headers::new(),
                                        &app_router()).unwrap();
            let result_body = response::extract_body_to_bytes(response);
            let r: BTreeSet<Score> = serde_json::from_str(&(from_utf8(&result_body).unwrap())).unwrap();
            println!("{:?}",  r);
            assert_eq!(r.len(), 1);
            let item = r.iter().last().unwrap();
            assert_eq!(item.score, 30);
            assert_eq!(item.user_id, "3");
        }
    }
}