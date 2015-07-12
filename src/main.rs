extern crate hyper;
extern crate rand;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;

use rustc_serialize::json;

use std::io::Write;
use std::io::Read;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::net::Fresh;
use rand::random;
use std::fs::File;

use getopts::Options;

/// Magic ball answer structure
#[derive(RustcDecodable, RustcEncodable, Debug)]
struct Answer {
    /// Answer text
    text: String,
    /// Answer probability
    probability: f64,
}

/// Load posible answer and return then as array
fn get_answers(file: String) -> Result<Vec<Answer>, &'static str> {
    info!("Loading answers from {}", file);
    let mut data = String::new();
    let f = File::open(file);
    if let Err(_) = f { 
        return Err("Error open file");
    }
    if let Err(_) = f.unwrap().read_to_string(&mut data) {
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    env_logger::init().unwrap();
    let mut opts = Options::new();
    opts.optopt("a", "answers", "answers file to read from", "FILE");
    let matches = opts.parse(&args[1..]).unwrap();

    let answers = get_answers(matches.opt_str("answers").unwrap()).unwrap();

    let handler = move |req: Request, res: Response<Fresh>| {
        let answer = answers.get(random::<usize>() % answers.len()).unwrap();
        send_answer(req, res, answer.text.as_bytes())
    };

    info!("Starting server on 0.0.0.0:3000");
    Server::http("0.0.0.0:3000").unwrap().handle(handler);
}