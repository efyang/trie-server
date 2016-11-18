extern crate hyper;
extern crate time;
extern crate rand;
#[macro_use]
extern crate lazy_static;

mod challenge;

use hyper::server::{Server, Handler, Request, Response};
use hyper::method::Method;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::cell::RefCell;
use std::io::Read;
use time::precise_time_s;
use std::sync::{Arc, Mutex};
use rand::{thread_rng, ThreadRng};
use challenge::Challenge;

const CHALLENGES_NEEDED: usize = 500;
const MAX_TIME_INTERVAL: f64 = 0.0001;

struct UserInfo {
    challenges_completed: usize,
    last_connected: f64,
    // whether in the dictionary or not
    last_challenge_solution: bool,
}

impl UserInfo {
    fn new(challenge_answer: bool) -> UserInfo {
        UserInfo {
            challenges_completed: 0,
            last_connected: precise_time_s(),
            last_challenge_solution: challenge_answer,
        }
    }

    fn challenges_completed(&self) -> usize {
        self.challenges_completed
    }

    fn update_after_correct(&mut self, new_challenge: &Challenge) {
        self.challenges_completed += 1;
        self.update_time();
        self.last_challenge_solution = new_challenge.answer;
    }

    fn update_time(&mut self) {
        self.last_connected = precise_time_s();
    }
}

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

struct ChallengeServer {
    current_users: Arc<Mutex<HashMap<SocketAddr, UserInfo>>>,
}

impl Handler for ChallengeServer {
    fn handle(&self, mut request: Request, response: Response) {
        let mut users = self.current_users.lock().unwrap();
        let user_addr = request.remote_addr;
        let user_exists = users.contains_key(&user_addr);
        if user_exists && request.method == Method::Post {
            match check_solution(&users[&user_addr], &mut request) {
                Some(correct) => {
                    if correct {
                        if users[&user_addr].challenges_completed() + 1 > CHALLENGES_NEEDED {
                            // they've finished, send the flag
                        } else {
                            let challenge = RNG.with(|rng| Challenge::generate(&mut *rng.borrow_mut()));
                            users.get_mut(&user_addr).unwrap().update_after_correct(&challenge);
                            // send back the next question
                            send_response(response, &format!("Check if the following word is in the dictionary: {}\n", challenge.question));
                        }
                    } else {
                        users.remove(&user_addr);
                        // tell user that they got it wrong
                        send_response(response, "Your response was incorrect\n");;
                    }
                }
                None => {
                    // tell user that their reponse failed in some way so got it wrong
                    users.remove(&user_addr);
                    send_response(response, "Your response failed in some way\n");
                }
            }
        } else if !user_exists && request.method == Method::Get {
            // new challenger: add them to users and set them up
            let challenge = RNG.with(|rng| Challenge::generate(&mut *rng.borrow_mut()));
            users.insert(user_addr, UserInfo::new(challenge.answer));
            send_response(response, &format!("Check if the following word is in the dictionary: {}\n", challenge.question));
        }
    }
}

fn send_response(response: Response, message: &str) {
    response.send(message.as_bytes()).unwrap();
}

// return if user got correct or not
fn check_solution(user: &UserInfo, request: &mut Request) -> Option<bool> {
    if precise_time_s() < user.last_connected + MAX_TIME_INTERVAL {
        parse_request(request).map(|b| user.last_challenge_solution == b)
    } else {
        // over time limit
        Some(false)
    }
}

fn parse_request(request: &mut Request) -> Option<bool> {
    let mut rstr = String::new();
    if request.read_to_string(&mut rstr).is_err() {
        return None;
    }
    rstr = rstr.trim().to_lowercase();
    if &rstr == "true" {
        Some(true)
    } else if &rstr == "false" {
        Some(false)
    } else {
        None
    }
}

impl ChallengeServer {
    fn new() -> ChallengeServer {
        ChallengeServer {
            current_users: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn main() {
    let challenge_server = ChallengeServer::new();
    let listen = Server::http("0.0.0.0:3000").unwrap().handle(challenge_server).unwrap();
    println!("Challenge server running on {:?}", listen.socket);
}
