use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde_json::Value;

/// Register math and logic helpers
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("length", Box::new(length_helper));
    handlebars.register_helper("not", Box::new(not_helper));
    handlebars.register_helper("and", Box::new(and_helper));
    handlebars.register_helper("or", Box::new(or_helper));
    handlebars.register_helper("eq", Box::new(eq_helper));
    handlebars.register_helper("ne", Box::new(ne_helper));
    handlebars.register_helper("gt", Box::new(gt_helper));
    handlebars.register_helper("gte", Box::new(gte_helper));
    handlebars.register_helper("lt", Box::new(lt_helper));
    handlebars.register_helper("lte", Box::new(lte_helper));
    handlebars.register_helper("exists", Box::new(exists_helper));
}

fn is_truthy(value: &Value) -> bool {
    if value.is_null() {
        return false;
    }
    if let Some(b) = value.as_bool() {
        return b;
    }
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        return !(trimmed.is_empty() || trimmed == "null" || trimmed == "false");
    }
    if let Some(n) = value.as_f64() {
        return n != 0.0;
    }
    if value.is_array() {
        return !value.as_array().unwrap().is_empty();
    }
    if value.is_object() {
        return !value.as_object().unwrap().is_empty();
    }
    true
}

/// Helper for getting array length
pub fn length_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        if let Some(array) = param.value().as_array() {
            out.write(&array.len().to_string())?;
        } else {
            out.write("0")?;
        }
    } else {
        out.write("0")?;
    }
    Ok(())
}

/// Helper for logical NOT operation
pub fn not_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value();
        out.write(if is_truthy(value) { "false" } else { "true" })?;
    } else {
        out.write("true")?;
    }
    Ok(())
}

/// Helper for logical AND with variadic params
pub fn and_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    for p in h.params() {
        if !is_truthy(p.value()) {
            out.write("false")?;
            return Ok(());
        }
    }
    out.write("true")?;
    Ok(())
}

/// Helper for logical OR with variadic params
pub fn or_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    for p in h.params() {
        if is_truthy(p.value()) {
            out.write("true")?;
            return Ok(());
        }
    }
    out.write("false")?;
    Ok(())
}

pub fn eq_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h
        .param(0)
        .map(|p| p.value())
        .cloned()
        .unwrap_or(Value::Null);
    let b = h
        .param(1)
        .map(|p| p.value())
        .cloned()
        .unwrap_or(Value::Null);
    out.write(if a == b { "true" } else { "false" })?;
    Ok(())
}

pub fn ne_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h
        .param(0)
        .map(|p| p.value())
        .cloned()
        .unwrap_or(Value::Null);
    let b = h
        .param(1)
        .map(|p| p.value())
        .cloned()
        .unwrap_or(Value::Null);
    out.write(if a != b { "true" } else { "false" })?;
    Ok(())
}

pub fn gt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|p| p.value().as_f64()).unwrap_or(0.0);
    let b = h.param(1).and_then(|p| p.value().as_f64()).unwrap_or(0.0);
    out.write(if a > b { "true" } else { "false" })?;
    Ok(())
}

pub fn gte_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|p| p.value().as_f64()).unwrap_or(0.0);
    let b = h.param(1).and_then(|p| p.value().as_f64()).unwrap_or(0.0);
    out.write(if a >= b { "true" } else { "false" })?;
    Ok(())
}

pub fn lt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|p| p.value().as_f64()).unwrap_or(0.0);
    let b = h.param(1).and_then(|p| p.value().as_f64()).unwrap_or(0.0);
    out.write(if a < b { "true" } else { "false" })?;
    Ok(())
}

pub fn lte_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|p| p.value().as_f64()).unwrap_or(0.0);
    let b = h.param(1).and_then(|p| p.value().as_f64()).unwrap_or(0.0);
    out.write(if a <= b { "true" } else { "false" })?;
    Ok(())
}

/// Existence check (not null/empty)
pub fn exists_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        out.write(if is_truthy(param.value()) {
            "true"
        } else {
            "false"
        })?;
    } else {
        out.write("false")?;
    }
    Ok(())
}
