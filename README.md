BotDetector - Proof-of-concept
==============================

A simple proof-of-concept of detecting scrapers/bots in a reverse-proxy written in Rust.

By proxying incoming http-requests through a reverse proxy, the proxy can analyze incoming traffic and label connections according to certain rules. What BotDetector currently does is a simple filtering on request-frequency but it is easy to imagine more elaborate ways of filtering.

There are three gradings of an incoming request:

 * GoodActor: A good actor which is let through without modification.
 * SuspiciousActor: A suspicious request is let through to the backend but has the http-header `bot-probability` set to a value between 0 and 1, describing likelyhood of request being a bot.
 * BadActor: A bad actor is turned away without access to the requested resource.

A simple test
-------------

Execute supplied script
```
./deploy_test_autoreload.sh
```
This starts a simple dummy-server and sets up `BotDetector` as a reverse proxy for this dummy-server.

Then, run 
```
curl localhost:8080/
```
a few times in quick succession. The first requests will pass through while later requests should be denied:

```
$ curl localhost:8080/
<p>Hello World!</p>
$ curl localhost:8080/
<p>Hello World!</p>
$ curl localhost:8080/
<p>Hello World!</p>
$ curl localhost:8080/
<p>Hello World!</p>
$ curl localhost:8080/
<p>Go away silly bot<p>
$ curl localhost:8080/botdetector_analytics
{
    "num_good_reqs": 4,
    "num_susp_reqs": 1,
    "num_bad_reqs": 1
}
```

server.rs
---------
This file sets up the proxy itself using `Hyper.rs`.

detector.rs
-----------
This code is called from the proxy to analyze and label incoming requests. Currently simply labels as suspicious or bad actor depending on request-frequency.


Building
--------

![Build status](https://travis-ci.org/PureW/BotDetector.svg?branch=master)

```
cargo build --release
```
This produces `botdetector` in target/release/botdetector. See `botdetector -h` for further instructions.


Simple improvements
-------------------

Due to lack of labeled training data, this simple POC infers bad actors from small, unlabeled logs. A big improvement would be to have access to a larger repository of labeled data.

With bigger datasets comes possibility of fingerprinting individual clients using features other than just ip-address. Examples of such would be raw data such as HTTP-headers, proxy-chains, heuristics such as frequency of calls, indirect data acquired from client-side javascript-execution etc.
