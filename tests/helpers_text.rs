<<<<<<< HEAD
use apicentric::simulator::template::helpers::text;
use handlebars::Handlebars;
=======
use handlebars::Handlebars;
use apicentric::simulator::template::helpers::text;
>>>>>>> origin/main
use serde_json::json;

#[test]
fn upper_and_contains_helpers() {
    let mut h = Handlebars::new();
    text::register(&mut h);
    let out = h
<<<<<<< HEAD
        .render_template(
            "{{#if (contains (upper word) \"HEL\")}}yes{{/if}}",
            &json!({"word": "hello"}),
        )
        .unwrap();
    assert_eq!(out, "yes");
}
=======
        .render_template("{{#if (contains (upper word) \"HEL\")}}yes{{/if}}", &json!({"word": "hello"}))
        .unwrap();
    assert_eq!(out, "yes");
}

>>>>>>> origin/main
