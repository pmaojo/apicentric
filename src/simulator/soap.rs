//! SOAP / XML support for the simulator.
//!
//! Converts XML request bodies into a `serde_json::Value` tree so that
//! Handlebars templates can access SOAP envelopes the same way they
//! access JSON: e.g. `{{request.body.Envelope.Body.MyOp.Param}}`.
//!
//! Element-name namespace prefixes are stripped (`ns2:Foo` → `Foo`).
//! Attributes are stored under `@name`. Mixed text content under `#text`.
//! Repeated children with the same name collapse to an array.

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use serde_json::{Map, Value};

/// Returns `true` if the given content-type header indicates XML / SOAP.
pub fn is_xml_content_type(content_type: &str) -> bool {
    let ct = content_type.to_ascii_lowercase();
    ct.contains("application/soap+xml")
        || ct.contains("text/xml")
        || ct.contains("application/xml")
}

/// Parses an XML document into a `serde_json::Value`.
///
/// The top-level value is an object containing exactly one key (the root
/// element name, namespace-stripped). This mirrors how `request.body`
/// looks for JSON bodies.
pub fn xml_to_value(xml: &str) -> Result<Value, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    // Skip prolog / comments / declarations until we hit the root element.
    let mut buf = Vec::new();
    let root_start: BytesStart<'static> = loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => break e.into_owned(),
            Ok(Event::Empty(e)) => {
                // Root is a self-closing element — just return it.
                let name = local_name(&e);
                let mut obj = Map::new();
                add_attributes(&e, &mut obj);
                let mut wrapper = Map::new();
                wrapper.insert(name, Value::Object(obj));
                return Ok(Value::Object(wrapper));
            }
            Ok(Event::Eof) => return Err("empty XML document".into()),
            Ok(_) => {
                buf.clear();
                continue;
            }
            Err(e) => return Err(format!("XML parse error: {e}")),
        }
    };
    buf.clear();

    let root_name = local_name(&root_start);
    let root_value = parse_element(&mut reader, &root_start)?;

    let mut wrapper = Map::new();
    wrapper.insert(root_name, root_value);
    Ok(Value::Object(wrapper))
}

fn local_name(start: &BytesStart) -> String {
    let full = std::str::from_utf8(start.name().as_ref())
        .unwrap_or("")
        .to_string();
    match full.split_once(':') {
        Some((_, local)) => local.to_string(),
        None => full,
    }
}

fn add_attributes(start: &BytesStart, obj: &mut Map<String, Value>) {
    for attr_res in start.attributes() {
        let Ok(attr) = attr_res else { continue };
        let key_full = std::str::from_utf8(attr.key.as_ref())
            .unwrap_or("")
            .to_string();
        let key_local = match key_full.split_once(':') {
            Some((_, local)) => local.to_string(),
            None => key_full,
        };
        let val = attr
            .unescape_value()
            .map(|c| c.into_owned())
            .unwrap_or_default();
        obj.insert(format!("@{key_local}"), Value::String(val));
    }
}

/// Parse the contents of an open element (already consumed `Start`) until
/// matching `End`. Returns the element's value as a JSON Value.
fn parse_element(
    reader: &mut Reader<&[u8]>,
    open: &BytesStart,
) -> Result<Value, String> {
    let mut obj = Map::new();
    add_attributes(open, &mut obj);

    let mut text_accum = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let child_name = local_name(&e);
                let child_value = parse_element(reader, &e)?;
                insert_child(&mut obj, child_name, child_value);
            }
            Ok(Event::Empty(e)) => {
                let child_name = local_name(&e);
                let mut child_obj = Map::new();
                add_attributes(&e, &mut child_obj);
                let child_value = if child_obj.is_empty() {
                    Value::Null
                } else {
                    Value::Object(child_obj)
                };
                insert_child(&mut obj, child_name, child_value);
            }
            Ok(Event::Text(t)) => {
                let txt = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                if !txt.trim().is_empty() {
                    text_accum.push_str(&txt);
                }
            }
            Ok(Event::CData(c)) => {
                let bytes = c.into_inner();
                if let Ok(s) = std::str::from_utf8(&bytes) {
                    text_accum.push_str(s);
                }
            }
            Ok(Event::End(_)) => {
                // Recursion guarantees this End corresponds to `open`,
                // because each nested element is consumed by its own
                // `parse_element` call which exits on its matching End.
                break;
            }
            Ok(Event::Eof) => {
                return Err(format!(
                    "unexpected EOF inside <{}>",
                    local_name(open)
                ))
            }
            Ok(_) => {}
            Err(e) => return Err(format!("XML parse error: {e}")),
        }
        buf.clear();
    }

    // Decide return shape:
    //   - text only          → string
    //   - children only      → object
    //   - text + children    → object with #text key
    //   - empty              → null
    if obj.is_empty() && text_accum.is_empty() {
        Ok(Value::Null)
    } else if obj.is_empty() {
        Ok(Value::String(text_accum))
    } else {
        if !text_accum.is_empty() {
            obj.insert("#text".to_string(), Value::String(text_accum));
        }
        Ok(Value::Object(obj))
    }
}

fn insert_child(obj: &mut Map<String, Value>, name: String, value: Value) {
    use serde_json::map::Entry;
    match obj.entry(name) {
        Entry::Vacant(v) => {
            v.insert(value);
        }
        Entry::Occupied(mut o) => {
            let existing = o.get_mut();
            match existing {
                Value::Array(arr) => arr.push(value),
                _ => {
                    let prev = std::mem::replace(existing, Value::Null);
                    *existing = Value::Array(vec![prev, value]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_simple_element() {
        let v = xml_to_value("<foo>bar</foo>").unwrap();
        assert_eq!(v, json!({"foo": "bar"}));
    }

    #[test]
    fn parses_nested_with_attributes() {
        let v = xml_to_value(
            r#"<root id="1"><child name="x">v</child></root>"#,
        )
        .unwrap();
        assert_eq!(
            v,
            json!({"root": {"@id": "1", "child": {"@name": "x", "#text": "v"}}})
        );
    }

    #[test]
    fn strips_namespace_prefixes() {
        let v = xml_to_value(
            r#"<soap:Envelope xmlns:soap="x"><soap:Body><Op>1</Op></soap:Body></soap:Envelope>"#,
        )
        .unwrap();
        assert_eq!(v, json!({"Envelope": {"Body": {"Op": "1"}}}));
    }

    #[test]
    fn repeated_children_become_array() {
        let v = xml_to_value("<list><item>a</item><item>b</item></list>").unwrap();
        assert_eq!(v, json!({"list": {"item": ["a", "b"]}}));
    }

    #[test]
    fn detects_xml_content_types() {
        assert!(is_xml_content_type("text/xml"));
        assert!(is_xml_content_type("application/xml; charset=utf-8"));
        assert!(is_xml_content_type("application/soap+xml; action=Foo"));
        assert!(!is_xml_content_type("application/json"));
    }
}
