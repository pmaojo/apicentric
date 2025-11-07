use std::process::Command;

use mockforge::simulator::config::ServiceDefinition;
use mockforge::simulator::react_query::to_react_query;
use tempfile::tempdir;

#[test]
fn generated_hooks_compile_and_call_endpoints() {
    let yaml = std::fs::read_to_string("tests/data/service_hooks.yaml").unwrap();
    let service: ServiceDefinition = serde_yaml::from_str(&yaml).unwrap();
    let ts = to_react_query(&service).unwrap();

    let dir = tempdir().unwrap();
    let hooks_path = dir.path().join("hooks.ts");
    std::fs::write(&hooks_path, ts).unwrap();

    // stub react-query module
    let stub_dir = dir.path().join("node_modules/@tanstack/react-query");
    std::fs::create_dir_all(&stub_dir).unwrap();
    std::fs::write(
        stub_dir.join("index.d.ts"),
        "export function useQuery(key: any, fn: any): any;\nexport function useMutation(fn: any): any;",
    )
    .unwrap();
    std::fs::write(
        stub_dir.join("index.js"),
        "exports.useQuery=(k,f)=>f();exports.useMutation=(f)=>({mutate:f});",
    )
    .unwrap();

    // compile TypeScript
    let root_dir = std::env::current_dir().unwrap();
    let tsc_path = root_dir.join("node_modules/.bin/tsc");
    let status = Command::new(tsc_path)
        .args([hooks_path.to_str().unwrap(), "--module", "commonjs"])
        .current_dir(dir.path())
        .status()
        .expect("tsc failed");
    assert!(status.success());

    // run hooks with fetch stub
    let script = r#"
const { usePetsQuery, usePostPetsMutation } = require('./hooks.js');
const calls = [];
global.fetch = (url, init) => { calls.push({url, method: init && init.method}); return Promise.resolve({ json: async () => ({}) }); };
usePetsQuery('http://example.com');
usePostPetsMutation('http://example.com').mutate({});
console.log(JSON.stringify(calls));
"#;
    let output = Command::new("node")
        .arg("-e")
        .arg(script)
        .current_dir(dir.path())
        .output()
        .expect("node run failed");
    assert!(output.status.success());
    let calls: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(calls[0]["url"], "http://example.com/api/pets");
    assert_eq!(calls[1]["url"], "http://example.com/api/pets");
    assert_eq!(calls[1]["method"], "POST");
}
