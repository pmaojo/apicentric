use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use apicentric::simulator::config::ServiceDefinition;
use apicentric::simulator::react_query::to_react_query;
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
    let local_tsc = root_dir.join("node_modules/.bin/tsc");
    let mut command = if local_tsc.exists() {
        Command::new(local_tsc)
    } else {
        Command::new(PathBuf::from("tsc"))
    };

    let status = command
        .args([hooks_path.to_str().unwrap(), "--module", "commonjs"])
        .current_dir(dir.path())
        .status();

    match status {
        Ok(status) => {
            if !status.success() {
                panic!("tsc exited with status {:?}", status.code());
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
<<<<<<< HEAD
            eprintln!(
                "Skipping react-query generation test: TypeScript compiler not available ({err})"
            );
=======
            eprintln!("Skipping react-query generation test: TypeScript compiler not available ({err})");
>>>>>>> origin/main
            return;
        }
        Err(err) => panic!("failed to run tsc: {err}"),
    }

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
