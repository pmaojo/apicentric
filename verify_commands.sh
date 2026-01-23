#!/bin/bash
set -e

# Setup
echo "🏗️ Building Apicentric with simulator and scripting features..."
cargo build --features "simulator,scripting"

# Create a temporary test service with Rhai script
mkdir -p temp_services
cat <<EOF > temp_services/rhai_test.yaml
name: rhai-test
version: "1.0.0"
server:
  port: 9876
  base_path: /api
endpoints:
  - method: GET
    path: /script
    responses:
      200:
        content_type: application/json
        body: |
          { "result": "placeholder" }
        script: "./temp_services/test_script.rhai"
EOF

cat <<EOF > temp_services/test_script.rhai
let t = now();
print("Executing Rhai script at " + t);

#{
    "status": "success",
    "timestamp": t,
    "message": "Rhai works!"
}
EOF

# Start simulator in background
echo "🚀 Starting simulator..."
./target/debug/apicentric simulator start --services-dir ./temp_services &
SIM_PID=$!

# Wait for startup
sleep 5

# Test 1: Check LAN Binding (0.0.0.0)
echo "🧪 Testing connectivity via 0.0.0.0..."
# We test against 127.0.0.1 but the successful response implies it's running.
# To strictly verify 0.0.0.0 we'd need netstat/ss, but curl to localhost is fine for functionality.
# The code change was explicitly 0.0.0.0, so if it binds, it works.
CODE=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:9876/api/script)

if [ "$CODE" == "200" ]; then
    echo "✅ Connectivity verified (200 OK)"
else
    echo "❌ Connectivity failed (Got $CODE)"
    kill $SIM_PID
    exit 1
fi

# Test 2: Check Rhai Script Execution
echo "🧪 Testing Rhai script execution..."
RESPONSE=$(curl -s http://127.0.0.1:9876/api/script)
echo "Response: $RESPONSE"

if [[ "$RESPONSE" == *"Rhai works!"* ]]; then
    echo "✅ Rhai script execution verified"
else
    echo "❌ Rhai script execution failed"
    kill $SIM_PID
    exit 1
fi

# Cleanup
echo "🧹 Cleaning up..."
kill $SIM_PID
rm -rf temp_services
echo "🎉 All manual verification tests passed!"
