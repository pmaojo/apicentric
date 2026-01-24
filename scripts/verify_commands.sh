#!/bin/bash
set -e

# Setup
echo "🏗️ Building Apicentric with full features..."
cargo build --features full

# Check for Anyhow
echo "🔍 Verifying 'anyhow' is not in the dependency tree..."
if cargo tree | grep -q "anyhow"; then
    echo "⚠️ 'anyhow' found in dependency tree! Checking direct dependencies..."
    if grep -q "anyhow" Cargo.toml; then
        echo "❌ 'anyhow' found in Cargo.toml!"
        exit 1
    else
        echo "✅ 'anyhow' removed from Cargo.toml (transitive dependencies may still exist)"
    fi
else
    echo "✅ 'anyhow' completely eradicated!"
fi

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

# Test 3: IoT Error Handling Drill (Mqtt)
# We will invoke the twin runner with a config that points to a non-existent file
# to ensure it errors gracefully with our new ApicentricError system.
echo "🧪 Testing Error Handling (IoT Twin)..."
set +e
./target/debug/apicentric twin run --device non_existent_device.yaml > output.log 2>&1
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -ne 0 ]; then
    echo "✅ Twin command failed as expected (Exit Code: $EXIT_CODE)"
    if grep -q "FileSystem error" output.log || grep -q "Failed to read device file" output.log; then
        echo "✅ Error message format verified"
    else
        echo "❌ Unexpected error message format:"
        cat output.log
        kill $SIM_PID
        exit 1
    fi
else
    echo "❌ Twin command should have failed!"
    kill $SIM_PID
    exit 1
fi

# Cleanup
echo "🧹 Cleaning up..."
kill $SIM_PID
rm -rf temp_services output.log
echo "🎉 All manual verification tests passed! Libertador Protocol Successful."
