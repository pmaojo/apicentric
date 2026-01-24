#!/bin/bash
./target/debug/apicentric mcp < mcp_pipe > mcp.log 2>&1 &
SERVER_PID=$!

sleep 2

cat test_mcp_tool.json > mcp_pipe

sleep 2

kill $SERVER_PID
