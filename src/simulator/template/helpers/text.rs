use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use regex::Regex;
use serde_json::Value;

/// Register text-related helpers
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("upper", Box::new(upper_helper));
    handlebars.register_helper("lower", Box::new(lower_helper));
    handlebars.register_helper("contains", Box::new(contains_helper));
    handlebars.register_helper("starts_with", Box::new(starts_with_helper));
    handlebars.register_helper("ends_with", Box::new(ends_with_helper));
    handlebars.register_helper("matches", Box::new(regex_match_helper));
}

/// Helper for converting to uppercase
pub fn upper_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(s) = param.value().as_str() {
            out.write(&s.to_uppercase())?;
        }
    }
    Ok(())
}

/// Helper for converting to lowercase
pub fn lower_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(s) = param.value().as_str() {
            out.write(&s.to_lowercase())?;
        }
    }
    Ok(())
}

/// String/array contains
pub fn contains_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
<<<<<<< HEAD
    let hay = h
        .param(0)
        .map(|p| p.value())
        .cloned()
        .unwrap_or(Value::Null);
    let needle = h
        .param(1)
        .map(|p| p.value())
        .cloned()
        .unwrap_or(Value::Null);
=======
    let hay = h.param(0).map(|p| p.value()).cloned().unwrap_or(Value::Null);
    let needle = h.param(1).map(|p| p.value()).cloned().unwrap_or(Value::Null);
>>>>>>> origin/main
    let result = match (hay, needle) {
        (Value::String(s), Value::String(n)) => s.contains(&n),
        (Value::Array(arr), v) => arr.iter().any(|e| e == &v),
        _ => false,
    };
    out.write(if result { "true" } else { "false" })?;
    Ok(())
}

pub fn starts_with_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let s = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");
    let pfx = h.param(1).and_then(|p| p.value().as_str()).unwrap_or("");
    out.write(if s.starts_with(pfx) { "true" } else { "false" })?;
    Ok(())
}

pub fn ends_with_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let s = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");
    let sfx = h.param(1).and_then(|p| p.value().as_str()).unwrap_or("");
    out.write(if s.ends_with(sfx) { "true" } else { "false" })?;
    Ok(())
}

pub fn regex_match_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let s = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");
    let pat = h.param(1).and_then(|p| p.value().as_str()).unwrap_or("");
    let result = Regex::new(pat).map(|re| re.is_match(s)).unwrap_or(false);
    out.write(if result { "true" } else { "false" })?;
    Ok(())
}
<<<<<<< HEAD
=======

>>>>>>> origin/main
