

use std::string::String;
use std::path::Path;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use std::thread;
use std::sync::mpsc;
use std::sync::Mutex;
use std::io;
use std::io::Read;

use hyper;
use hyper::server::{Handler, Server, Request, Response};
use hyper::status::StatusCode;

use serde_json;

use ::detector;

#[derive(Serialize, Deserialize, Default)]
struct ReqStats {
    num_good_reqs: u32,
    num_susp_reqs: u32,
    num_bad_reqs: u32,
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

fn proxy(req: &mut Request,
         address: &str,
         port: u16,
         path: &str,
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

/// BaseHandler persists data between requests
/// 
/// At first planned to use `mpsc` to send `ReqStats`
/// lock-free to a separare aggregation-thread. 
/// Turns out it is not possible to do away with the Mutex in Hyper, 
/// see:
/// https://stackoverflow.com/questions/40060378/why-does-hyper-require-handler-to-implement-sync-instead-of-using-independent-ha
struct BaseHandler {
    backend_address: String,
    backend_port: u16,
    req_stats: Mutex<ReqStats>,
    detector: Mutex<detector::Detector>,
}

impl BaseHandler {

    fn route(&self, mut req: &mut Request) -> Resp {

        if self.check_bad_actor(&mut req) {
            // Early return for bad actors
            return Resp::new(StatusCode::Unauthorized,
                             "<p>Go away silly bot<p>\n");
        }

        match req.uri.clone() {
            hyper::uri::RequestUri::AbsolutePath(ref p) => {
                info!("BaseHandler::route received {}", p);
                if p.starts_with("/botdetector_analytics") {
                    self.analytics(&mut req)
                } else {
                    proxy(&mut req,
                          &self.backend_address,
                          self.backend_port,
                          &p,
                          )
                }
            },
            _ => {
                error!("Not implemented this method yet");
                Resp::new(StatusCode::NotImplemented, "")
            }
        }
    }

    /// Handle the details of checking ActorStatus and marking header
    fn check_bad_actor(&self, mut req: &mut Request) -> bool {

        // TODO: Probably want to call `new_event` when we know
        //       if the route is valid or not.
        let mut detector = self.detector.lock().unwrap();
        match detector.new_event(&req.remote_addr) {
            detector::ActorStatus::BadActor => {
                let mut s = self.req_stats.lock().unwrap();
                (*s).num_bad_reqs += 1u32;
                true
            },
            detector::ActorStatus::SuspiciousActor(p) => {
                // Add a header for backend to see suspicious actors
                req.headers.set_raw("bot-probability",
                                    vec!(format!("{}", p)
                                            .to_string()
                                            .into_bytes()));
                let mut s = self.req_stats.lock().unwrap();
                (*s).num_susp_reqs += 1u32;
                false
            },
            detector::ActorStatus::GoodActor => {
                let mut s = self.req_stats.lock().unwrap();
                (*s).num_good_reqs += 1u32;
                false
            }
        }
    }

    fn analytics(&self, _: &mut Request) -> Resp {
        info!("Calling analytics");
        match serde_json::to_string_pretty(&*self.req_stats.lock().unwrap()) {
            Ok(s) => {
                Resp::new(StatusCode::Ok, s)
            },
            Err(err) => {
                error!("Error in serializing stats: {:?}",
                       err);
                Resp::new(StatusCode::InternalServerError, 
                          "I am terribly sorry, something went wrong")
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

    //let (tx, rx) = mpsc::channel();

    let listen_to: SocketAddr = format!("{}:{}", 
                                        listen_address, 
                                        listen_port).parse()
        .expect("Could not parse socket address");

    info!("Starting botdetector listening to {}, proxying to {}:{}",
          listen_to, listen_address, listen_port);
    Server::http(listen_to).unwrap()
            .handle(BaseHandler {
                req_stats: Mutex::new(ReqStats::default()),
                backend_address: backend_address.to_string(),
                backend_port: backend_port,
                detector: Mutex::new(
                    detector::Detector::new(
                        detector::DetectorConf::new())),
            })
            .unwrap();
}

