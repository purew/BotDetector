BotDetector - Proof-of-concept
==============================


![Build status](https://travis-ci.org/PureW/BotDetector.svg?branch=master)

A simple proof-of-concept of detecting scrapers/bots in a reverse-proxy written in Rust.

BotDetector has two modes, *train* and *deploy*:

Mode: train
-----------

BotDetector parses a log-file from nginx to build (simple) rules on correct and bad traffic.

Mode: deploy
------------

BotDetector acts as a reverse-proxy and analyses incoming traffic before hitting actual servers and can either drop verified bad actors or mark potential bad actors in a HTTP-header.


Building
--------

```
cargo build --release
```

You can then train the botdetector using a small supplied logfile
```
gunzip -k data/train/logs/nginx.small.log
target/release/botdetector train data/train/logs/nginx.small.log
```

And finally start the reverse proxy with:
```
target/release/botdetector deploy -a localhost -p 8080
```


This produces `botdetector` in target/release/botdetector. See `botdetector -h`, `botdetector train -h`, `botdetector deploy -h` for further instructions.


Simple improvements
-------------------

Due to lack of labeled training data, this simple POC infers bad actors from small, unlabeled logs. A big improvement would be to have access to a larger repository of labeled data.

With bigger datasets comes possibility of fingerprinting individual clients using features other than just ip-address. Examples of such would be raw data such as HTTP-headers, proxy-chains, heuristics such as frequency of calls, indirect data acquired from client-side javascript-execution etc.
