
extern crate lru_cache;

use std::cmp;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::{HashMap, VecDeque};
use std::vec::Vec;
use std::net::{SocketAddr, IpAddr};

use self::lru_cache::LruCache;


/// Track this many clients
const NUM_CLIENTS_TRACK: usize = 10000;
/// Track this many events per client
const NUM_CLIENT_EVENTS: usize = 10;

pub const SUSP_PLACEHOLDER: f32 = 0.5;

#[derive(PartialEq, Debug)]
pub enum ActorStatus {
    GoodActor,
    SuspiciousActor(f32),
    BadActor,
}

#[derive(Clone)]
pub struct Event {
    ts: SystemTime,
}

impl Event {
    fn new() -> Event {
        Event { ts: SystemTime::now() }
    }
}


pub struct IPData {
    events: VecDeque<Event>,
    evt_freq: f32,
}

impl IPData {
    fn new() -> IPData {
        IPData {
            events: VecDeque::with_capacity(NUM_CLIENT_EVENTS),
            evt_freq: 0.0,
        }
    }

    /// Register event for this ip and calculate key-values
    fn new_event(&mut self, evt: Event) {

        // Only keep newest events
        if self.events.len() >= NUM_CLIENT_EVENTS {
            self.events.pop_front().unwrap();
        }
        debug!("num_evts {}", self.events.len());
        self.events.push_back(evt);
        debug!("num_evts {}", self.events.len());

        if self.events.len() <= 1 {
            return;
        }
        // Just go with a simple frequency for now
        match self.events
            .back()
            .unwrap()
            .ts
            .duration_since(self.events
                .front()
                .unwrap()
                .ts
                .clone()) {
            Ok(tdiff) => {
                let secs = cmp::max(10, tdiff.as_secs());
                debug!("new_event: secs={}, events.len()={}",
                       secs, self.events.len());
                self.evt_freq = self.events.len() as f32 / secs as f32;
            }
            Err(err) => {
                error!("Error while calculating tdiff: {:?}", err);
            }
        }
    }
}

pub struct Detector {
    ip_map: LruCache<IpAddr, IPData>,
    conf: DetectorConf,
}


/// Configuration of Detector
#[derive(Debug)]
pub struct DetectorConf {
    bad_evt_freq: f32,
    susp_evt_freq: f32,
}

impl DetectorConf {
    pub fn new() -> DetectorConf {
        DetectorConf {
            bad_evt_freq: 30.0 / 60.0,
            susp_evt_freq: 20.0 / 60.0,
        }
    }

    /// Run analysis on an IPData to judge its `ActorStatus`
    fn analyze_actor(&self, d: &IPData) -> ActorStatus {
        debug!("analyze_actor: evt_freq={}", d.evt_freq);
        if d.evt_freq >= self.bad_evt_freq {
            ActorStatus::BadActor
        } else if d.evt_freq >= self.susp_evt_freq {
            ActorStatus::SuspiciousActor(SUSP_PLACEHOLDER)
        } else {
            ActorStatus::GoodActor
        }
    }
}

impl Detector {
    pub fn new(conf: DetectorConf) -> Detector {
        info!("Detector initialized with conf {:?}", conf);
        Detector {
            ip_map: LruCache::new(NUM_CLIENTS_TRACK),
            conf: conf,
        }
    }

    /// Register new event in `Detector`, return level of suspicion.
    ///
    /// TODO: Detection could use a lot more information than just the
    ///       SocketAddr but keep it simple for now.
    pub fn new_event(&mut self, addr: &SocketAddr) -> ActorStatus {

        let ip = addr.ip();

        if !self.ip_map.contains_key(&ip) {
            debug!("Adding map-entry for {}", addr.ip());
            self.ip_map.insert(ip, IPData::new());
        }
        let mut d = self.ip_map.get_mut(&ip).unwrap();
        d.new_event(Event::new());

        self.conf.analyze_actor(d)
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_analyze_actor() {

        let num_good_evts = 4;
        let num_susp_evts: usize = 6;
        let num_bad_evts: usize = 9;

        let conf = DetectorConf {
            bad_evt_freq: num_bad_evts as f32 / 60.0,
            susp_evt_freq: num_susp_evts as f32 / 60.0,
        };

        let mut d = IPData::new();
        d.new_event(Event { ts: UNIX_EPOCH });

        for (num_evts, ans) in vec!(
                (num_good_evts, ActorStatus::GoodActor),
                (num_susp_evts, ActorStatus::SuspiciousActor(SUSP_PLACEHOLDER)),
                (num_bad_evts, ActorStatus::BadActor),
            ) {
            for i in d.events.len()..num_evts {
                println!("Adding evt {}", i);
                d.new_event(Event { ts: UNIX_EPOCH + Duration::new(60, 0) });
            }
            println!("Has {} evts", d.events.len());
            assert_eq!(conf.analyze_actor(&d), ans);
        }
    }
}
