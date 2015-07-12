extern crate hyper;
extern crate rand;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate mockstream;

use rustc_serialize::json;

use std::io::Write;
use std::io::Read;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::net::Fresh;

use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

use std::fs::File;

use getopts::Options;

use mockstream::MockStream;

/// Magic ball answer structure
#[derive(RustcDecodable, RustcEncodable, Debug, PartialEq)]
struct Answer {
    /// Answer text
    text: String,
    /// Answer weight
    weight: f64,
}

/// Load posible answer and return then as array
fn get_answers<R: std::io::Read>(r: &mut R) -> Result<Vec<Answer>, &'static str> {
    let mut data = String::new();
    if let Err(_) = r.read_to_string(&mut data) {
        return Err("Error read from file");
    }
    let answers: Vec<Answer> = json::decode(data.as_ref()).unwrap();
    info!("Loaded {} answers", answers.len());
    for answer in answers.iter() {
        debug!("Answer: {:?}", answer);
    }
    Ok(answers)
}

fn send_answer(req: Request, res: Response<Fresh>, answer: &[u8]) {
    debug!("{:?} {:?}", req.method, req.uri);
    let mut res = res.start().unwrap();
    res.write_all(answer).unwrap();
    res.end().unwrap();
}

/// Sum weight of all answers
fn total_weight(answers: &[Answer]) -> f64 {
    answers.iter().fold(0 as f64,
                        |acc, answer| acc + answer.weight)
}

/// Randomly selects one of the answers based on their weights
fn pick(answers: &[Answer]) -> Option<&Answer> {
    if answers.len() == 0 {
        return None;
    }
    let mut rnd = rand::thread_rng();
    let val = Range::new(0.0, total_weight(answers.as_ref())).
                ind_sample(&mut rnd);
    let mut sum = 0.0;
    answers.iter().find(|answer| {
        sum += answer.weight;
        sum > val
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    env_logger::init().unwrap();
    let mut opts = Options::new();
    opts.optopt("a", "answers", "answers file to read from", "FILE");
    let matches = opts.parse(&args[1..]).unwrap();
    let answers_file = matches.opt_str("answers").unwrap();
    let answers = get_answers(&mut File::open(answers_file).unwrap()).unwrap();
    let handler = move |req: Request, res: Response<Fresh>| {
        send_answer(req, res, pick(answers.as_ref()).
                                    unwrap().
                                    text.
                                    as_bytes());
    };

    info!("Starting server on 0.0.0.0:3000");
    Server::http("0.0.0.0:3000").unwrap().handle(handler);
}

#[test]
fn test_total_weight() {
    assert_eq!(total_weight([].as_ref()),
               0.0);
    assert_eq!(total_weight([Answer{text: "A".to_string(), weight: 1.0},
                             Answer{text: "B".to_string(), weight: 2.0}].as_ref()),
               3.0);
}

#[test]
fn test_pick() {
    assert!(pick([].as_ref()).is_none());
    assert!(pick([Answer{text: "A".to_string(), weight: 1.0},
                  Answer{text: "B".to_string(), weight: 2.0}].as_ref()).is_some());
}

#[test]
fn test_get_answers() {
    let mut mock = MockStream::new();
    mock.push_bytes_to_read(r#"[{"text": "A", "weight": 1}]"#.as_ref());
    assert_eq!(vec![Answer{text: "A".to_string(), weight: 1.0}],
               get_answers(&mut mock).unwrap());
}