extern crate hyper;
extern crate rand;

use std::io::Write;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::net::Fresh;
use rand::random;

/// Magic ball answer structure
struct Answer {
    /// Answer text
    text: String,
    /// Answer probability
    probability: f64,
}

/// Load posible answer and return then as array
fn get_answers() -> Vec<Answer> {
    vec![Answer{text: "Yes".to_string(), probability: 0.5},
         Answer{text: "No".to_string(), probability: 0.5}]
}

fn send_answer(_: Request, res: Response<Fresh>, answer: &[u8]) {
    let mut res = res.start().unwrap();
    res.write_all(answer).unwrap();
    res.end().unwrap();
}

fn main() {
    let answers = get_answers();
    let handler = move |req: Request, res: Response<Fresh>| {
        let answer = answers.get(random::<usize>() % answers.len()).unwrap();
        send_answer(req, res, answer.text.as_bytes())
    };

    Server::http("127.0.0.1:3000").unwrap().handle(handler);
}