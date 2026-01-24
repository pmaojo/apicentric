use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use apicentric::simulator::config::ServiceDefinition;
use apicentric::simulator::react_query::generate_react_query_hooks;
use apicentric::simulator::react_view::generate_react_view;
use tempfile::tempdir;

#[test]
fn generated_view_compiles_and_composes_with_hooks() {
    let yaml = std::fs::read_to_string("tests/data/service_hooks.yaml").unwrap();
    let service: ServiceDefinition = serde_yaml::from_str(&yaml).unwrap();
    let hooks_ts = generate_react_query_hooks(&service).unwrap();
    let view_tsx = generate_react_view(&service).unwrap();

    let dir = tempdir().unwrap();
    let hooks_path = dir.path().join("hooks.ts");
    let view_path = dir.path().join("view.tsx");
    std::fs::write(&hooks_path, hooks_ts).unwrap();
    std::fs::write(&view_path, view_tsx).unwrap();

    // stub tanstack/react-query
    let rq_dir = dir.path().join("node_modules/@tanstack/react-query");
    std::fs::create_dir_all(&rq_dir).unwrap();
    std::fs::write(
        rq_dir.join("index.d.ts"),
        "export function useQuery(key: any, fn: any): any;\nexport function useMutation(fn: any): any;",
    )
    .unwrap();
    std::fs::write(
        rq_dir.join("index.js"),
        "exports.useQuery=(k,f)=>f();exports.useMutation=(f)=>({mutate:f});",
    )
    .unwrap();

    // stub react
    let react_dir = dir.path().join("node_modules/react");
    std::fs::create_dir_all(&react_dir).unwrap();
    std::fs::write(
        react_dir.join("index.d.ts"),
        "export function useState<T>(i:T): [T,(v:T)=>void];\nexport declare const React: {createElement:any; useState: typeof useState};\nexport default React;",
    )
    .unwrap();
    std::fs::write(
        react_dir.join("index.js"),
        "function useState(i){return [i, function(){}];}\nfunction createElement(type, props, ...children){return {type, props:{...props, children}};}\nmodule.exports={useState, createElement, default:{createElement,useState}};",
    )
    .unwrap();

    // compile
    let root_dir = std::env::current_dir().unwrap();
    let local_tsc = root_dir.join("node_modules/.bin/tsc");
    let mut command = if local_tsc.exists() {
        Command::new(local_tsc)
    } else {
        Command::new(PathBuf::from("tsc"))
    };

    let status = command
        .args([
            hooks_path.to_str().unwrap(),
            view_path.to_str().unwrap(),
            "--jsx",
            "react",
            "--module",
            "commonjs",
        ])
        .current_dir(dir.path())
        .status();

    match status {
        Ok(status) => {
            if !status.success() {
                panic!("tsc exited with status {:?}", status.code());
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            eprintln!(
                "Skipping react view generation test: TypeScript compiler not available ({err})"
            );
            return;
        }
        Err(err) => panic!("failed to run tsc: {err}"),
    }

    // run component to ensure hooks execute
    let script = r#"
const { ServiceView } = require('./view.js');
const calls = [];
global.fetch = (url, init) => { calls.push({url, method: init && init.method}); return Promise.resolve({ json: async () => ({}) }); };
const tree = ServiceView({ baseUrl: 'http://example.com' });
const children = Array.isArray(tree.props.children) ? tree.props.children : [tree.props.children];
const form = children.find(c => c.type === 'form');
form.props.onSubmit({ preventDefault(){} });
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
