use chrono::DateTime;
use handlebars::Handlebars;
use mockforge::simulator::template::helpers::faker;
use serde_json::json;

#[test]
fn now_generates_timestamp() {
    let mut h = Handlebars::new();
    faker::register(&mut h);
    let out = h.render_template("{{now}}", &json!({})).unwrap();
    assert!(DateTime::parse_from_rfc3339(&out).is_ok());
}

#[test]
fn random_string_length() {
    let mut h = Handlebars::new();
    faker::register(&mut h);
    let out = h.render_template("{{random_string 5}}", &json!({})).unwrap();
    assert_eq!(out.len(), 5);
}

