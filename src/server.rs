
extern crate iron;
extern crate mount;
extern crate staticfile;
extern crate time;
extern crate lru_time_cache;


use std::path::Path;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use self::lru_time_cache::LruCache;
use self::iron::prelude::*;
use self::iron::{BeforeMiddleware, AfterMiddleware, typemap};
use self::mount::Mount;
use self::staticfile::Static;
use self::time::precise_time_ns;

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
    Ok(Response::with((iron::status::Ok, "Hello World")))
}

pub fn run(address: &str, port: u16) {

    let mut mount = Mount::new();
    mount.mount("/static/", Static::new(Path::new("static/")));

    let mut chain = Chain::new(mount);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    Iron::new(chain)
        .http(SocketAddr::new(IpAddr::from_str(address).unwrap(), port))
        .unwrap();
}
