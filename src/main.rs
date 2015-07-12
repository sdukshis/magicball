extern crate hyper;
extern crate rand;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate env_logger;

use rustc_serialize::json;

use std::io::Write;
use std::io::Read;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::net::Fresh;
use rand::random;
use std::fs::File;

/// Magic ball answer structure
#[derive(RustcDecodable, RustcEncodable, Debug)]
struct Answer {
    /// Answer text
    text: String,
    /// Answer probability
    probability: f64,
}

/// Load posible answer and return then as array
fn get_answers() -> Result<Vec<Answer>, &'static str> {
    let mut data = String::new();
    let mut f = File::open("answers.json");
    if let Err(e) = f { 
        return Err("Error open file");
    }
    if let Err(e) = f.unwrap().read_to_string(&mut data) {
        return Err("Error read from file");
    }
    let answers: Vec<Answer> = json::decode(data.as_ref()).unwrap();
    Ok(answers)
}

fn send_answer(_: Request, res: Response<Fresh>, answer: &[u8]) {
    let mut res = res.start().unwrap();
    res.write_all(answer).unwrap();
    res.end().unwrap();
}

fn main() {
    env_logger::init().unwrap();
    info!("Loading answers");
    let answers = get_answers().unwrap();
    info!("Loaded {} answers", answers.len());
    for answer in answers.iter() {
        debug!("Answer: {:?}", answer);
    }
    let handler = move |req: Request, res: Response<Fresh>| {
        let answer = answers.get(random::<usize>() % answers.len()).unwrap();
        send_answer(req, res, answer.text.as_bytes())
    };

    info!("Starting server on 127.0.0.1:3000");
    Server::http("127.0.0.1:3000").unwrap().handle(handler);
}