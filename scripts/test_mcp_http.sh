#!/bin/bash
cargo build --bin apicentric --features mcp

./target/debug/apicentric --config apicentric.json mcp --http &
SERVER_PID=$!

sleep 10

curl -X POST -H "Content-Type: application/json" -d @test_mcp_list_request.json http://127.0.0.1:8080/ > curl_output.json

sleep 5

kill $SERVER_PID
