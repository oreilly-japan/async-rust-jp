#!/usr/bin/env bash

SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"
cd $SCRIPTPATH
cd ..
cd connection && cargo build --release && cd ..
cd server && cargo build --release && cd ..
cp connection/target/release/connection ./connection_bin
cp server/target/release/server ./server_bin
