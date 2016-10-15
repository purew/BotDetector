
extern crate mount;
extern crate staticfile;
extern crate time;
extern crate lru_time_cache;

use std::string::String;
use std::path::Path;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use std::thread;
use std::sync::mpsc;
use std::sync::Mutex;
use std::io;
use std::io::Read;

use self::lru_time_cache::LruCache;
use hyper;
use hyper::server::{Handler, Server, Request, Response};
use hyper::status::StatusCode;
use self::time::precise_time_ns;
use log;

struct ReqStats {
}

#[derive(debug)]
struct Resp {
    status: StatusCode,
    body: Vec<u8>,
}

impl Resp {

    fn new<S: Into<Vec<u8>>>(status: StatusCode, body: S) -> Resp {
        Resp {
            status: status,
            body: body.into(),
        }
    }
}

fn analytics(_: &mut Request) -> Resp {
    info!("Calling analytics");
    Resp::new(StatusCode::Ok, "Hello World")
}

fn proxy(req: &mut Request,
         address: &str,
         port: u16,
         path: &str,
         tx_stats: &Mutex<mpsc::Sender<ReqStats>>,
         ) -> Resp {

    let backend_url = format!("http://{}:{}{}",
                              address,
                              port,
                              path);
                              
    info!("proxying to {}", backend_url);
    let client = hyper::Client::new();
    match client.get(&backend_url).send() {
        Ok(mut resp) => {
            let mut body = String::new();
            match resp.read_to_string(&mut body) {
                Ok(_) => {
                    // Return response to client
                    Resp::new(StatusCode::Ok, body)
                },
                Err(_) => {
                    error!("Failed to read body from backend");
                    Resp::new(StatusCode::InternalServerError,
                              "Could not read body")
                }
            }
        },
        Err(err) => {
            // Error in backend-request to backend,
            // return generic error-code to client
            error!("Request to backend failed: {:?}", err);
            Resp::new(StatusCode::InternalServerError,
                      format!("{}", err))
        }
    }
}

struct BaseHandler {
    sender: Mutex<mpsc::Sender<ReqStats>>,
    backend_address: String,
    backend_port: u16,
}

impl BaseHandler {

    fn route(&self, mut req: &mut Request) -> Resp {
        match req.uri.clone() {
            hyper::uri::RequestUri::AbsolutePath(ref p) => {
                if p.starts_with("/analytics") {
                    analytics(&mut req)
                } else {
                    proxy(&mut req,
                          &self.backend_address,
                          self.backend_port,
                          &p,
                          &self.sender)
                }
            },
            _ => {
                error!("Not implemented this method yet");
                Resp::new(StatusCode::NotImplemented, "")
            }
        }
    }
}

impl Handler for BaseHandler {
    fn handle(&self, mut req: Request, mut resp: Response) {
        let r = self.route(&mut req);
        {
            *resp.status_mut() = r.status;
        }
        io::copy(&mut r.body.as_slice(), 
                 &mut resp.start().unwrap()).unwrap();
    }
}

pub fn run(listen_address: &str, listen_port: u16, 
           backend_address: &str, backend_port:u16) {

    let (tx, rx) = mpsc::channel();

    let listen_to: SocketAddr = format!("{}:{}", 
                                        listen_address, 
                                        listen_port).parse()
        .expect("Could not parse socket address");

    info!("Starting botdetector listening to {}, proxying to {}:{}",
          listen_to, listen_address, listen_port);
    Server::http(listen_to).unwrap()
            .handle(BaseHandler {
                sender: Mutex::new(tx),
                backend_address: backend_address.to_string(),
                backend_port: backend_port,
            })
            .unwrap();
}

