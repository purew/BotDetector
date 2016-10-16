#!/bin/bash
# Build, deploy reverse-proxy with dummy-server, autoreload on changes

set -e
set -u

BIN=botdetector
BIN_ARGS="deploy"
PORT_BACKEND=9000
PID=/tmp/deploy_script.pid
WATCHES=src

# Kill children when this script exits
trap "kill -- -0" EXIT

function get_pid {
    if [ -f $PID ]
    then
        cat $PID
    fi
}
# Clean up any earlier runs
OLD_PID=$(get_pid)
if [ "$OLD_PID" != "" ] && [ -e /proc/$OLD_PID ]
then
    echo "Killing older deploy-script with pid $OLD_PID"
    kill $OLD_PID
else
    echo "No earlier deploy-script running"
fi
echo "$$" > $PID

# Now begin the actual build/deploy/listen-loop
echo "Starting dummy-server and save pid"
cd data/html/ && python -m http.server $PORT_BACKEND --bind 127.0.0.1 &
echo "Launched dummy http-server at localhost:$PORT_BACKEND"

function relaunch {
    cargo build
    # Kill old proxy if running
    if [ "$(pgrep $BIN)" != "" ]; then
        echo "Killing old botdetector"
        kill $(pgrep $BIN)
    fi
    echo "Launching new botdetector"
    RUST_BACKTRACE=1 RUST_LOG=$BIN target/debug/$BIN $BIN_ARGS &
}

relaunch

# Setup filewatches on src
while (true); do 
    inotifywait -rq  -e modify "$WATCHES" && relaunch
    sleep 1
done
