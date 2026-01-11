use apicentric::simulator::template::helpers::text;
use handlebars::Handlebars;
use serde_json::json;

#[test]
fn upper_and_contains_helpers() {
    let mut h = Handlebars::new();
    text::register(&mut h);
    let out = h
        .render_template(
            "{{#if (contains (upper word) \"HEL\")}}yes{{/if}}",
            &json!({"word": "hello"}),
        )
        .unwrap();
    assert_eq!(out, "yes");
}
