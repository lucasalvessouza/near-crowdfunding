#!/bin/sh

rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
near create-account subtest21.olucassouza3.testnet --masterAccount olucassouza3.testnet --initialBalance 3
near deploy --wasmFile ./target/wasm32-unknown-unknown/release/hello_near.wasm --accountId subtest21.olucassouza3.testnet
near call subtest21.olucassouza3.testnet create_project '{"target": 100000, "deadline": 1715537920, "name": "Project 1", "description": "Description project 1"}' --accountId olucassouza3.testnet