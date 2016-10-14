
extern crate mount;
extern crate staticfile;
extern crate time;
extern crate lru_time_cache;

use std::string::String;
use std::path::Path;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use self::lru_time_cache::LruCache;
use ::hyper;
use std::io::Read;
use ::iron::prelude::*;
use ::iron::status;
use ::iron::{BeforeMiddleware, AfterMiddleware, typemap};
use self::mount::Mount;
use self::staticfile::Static;
use self::time::precise_time_ns;
use log;

struct ResponseTime;

impl typemap::Key for ResponseTime {
    type Value = u64;
}

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}


fn analytics(_: &mut Request) -> IronResult<Response> {
    info!("Calling analytics");
    Ok(Response::with((status::Ok, "Hello World")))
}

fn proxy(req: &mut Request, 
         address: &str, 
         port: u16) -> IronResult<Response> {

    let client = hyper::Client::new();
    let backend_url = format!("{}://{}:{}/{}",
                              req.url.scheme(),
                              address,
                              port,
                              req.url.path().join("/"));
    info!("proxying to {}", backend_url);
    match client.get(&backend_url).send() {
        Ok(mut resp) => {
            let mut body = String::new();
            match resp.read_to_string(&mut body) {
                Ok(_) => {
                    // Return response to client
                    Ok(Response::with((status::Ok, 
                                       body)))
                },
                Err(_) => {
                    error!("Failed to read body from backend");
                    Ok(Response::with((status::InternalServerError,
                                       "Could not read body")))
                }
            }
        },
        Err(err) => {
            // Error in backend-request to backend,
            // return generic error-code to client
            error!("Request to backend failed: {:?}", err);
            Ok(Response::with((status::InternalServerError,
                               format!("{}", err))))
        }
    }
}

pub fn run(listen_address: &str, listen_port: u16, 
           backend_address: &str, backend_port:u16) {


    // TODO: Cleanup dance with variables in closure to fix 
    //       lifetime-issues
    let cl_addr = backend_address.to_string();
    let cl_port = backend_port.clone();
    let custom_proxy = move |req: &mut Request| 
            -> IronResult<Response> {
        proxy(req, &cl_addr, cl_port)
    };

    let mut mount = Mount::new();
    mount.mount("/analytics/", analytics)
         .mount("/", custom_proxy);
    ;

    let mut chain = Chain::new(mount);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);

    info!("Starting botdetector listening to {}:{}, proxying to {}:{}",
          listen_address, listen_port, backend_address, backend_port);
    Iron::new(chain)
        .http(format!("{}:{}", listen_address, listen_port).as_str())
        .unwrap();
}
