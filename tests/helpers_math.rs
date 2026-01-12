use apicentric::simulator::template::helpers::math;
use handlebars::Handlebars;
use serde_json::json;

#[test]
fn eq_helper_evaluates() {
    let mut h = Handlebars::new();
    math::register(&mut h);
    let out = h.render_template("{{eq 1 1}}", &json!({})).unwrap();
    assert_eq!(out, "true");
}

#[test]
fn length_helper_counts() {
    let mut h = Handlebars::new();
    math::register(&mut h);
    let out = h
        .render_template("{{length array}}", &json!({"array": [1,2,3]}))
        .unwrap();
    assert_eq!(out, "3");
}
