use chrono::{DateTime, Utc};
use fake::{faker::{internet::en::FreeEmail, lorem::en::Sentence, name::en::Name}, Fake};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use regex::Regex;
use serde_json::Value;
use uuid::Uuid;

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

/// Helper for generating current timestamp
pub fn now_helper(
    _h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.to_rfc3339();
    out.write(&timestamp)?;
    Ok(())
}

/// Helper for generating random values
pub fn random_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("uuid");

    let result = match param {
        "uuid" => Uuid::new_v4().to_string(),
        "number" => {
            let min = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(0);
            let max = h.param(2).and_then(|v| v.value().as_u64()).unwrap_or(100);
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().hash(&mut hasher);
            let hash = hasher.finish();
            let range = if max > min { max - min } else { 1 };
            (min + (hash % range)).to_string()
        }
        _ => "".to_string(),
    };

    out.write(&result)?;
    Ok(())
}

/// Helper for generating realistic sample data using the `fake` crate
pub fn faker_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let key = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("");

    let value = match key {
        "internet.email" => FreeEmail().fake::<String>(),
        "person.name" => Name().fake::<String>(),
        "lorem.sentence" => Sentence(3..6).fake::<String>(),
        _ => String::new(),
    };

    out.write(&value)?;
    Ok(())
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

/// Helper for finding items in arrays
pub fn find_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(array_param) = h.param(0) {
        if let Some(array) = array_param.value().as_array() {
            if let Some(key_param) = h.param(1) {
                if let Some(key) = key_param.value().as_str() {
                    if let Some(value_param) = h.param(2) {
                        // Find item where item[key] == value
                        for item in array {
                            if let Some(obj) = item.as_object() {
                                if let Some(item_value) = obj.get(key) {
                                    if item_value == value_param.value() {
                                        out.write(
                                            &serde_json::to_string(item).unwrap_or_default(),
                                        )?;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out.write("null")?;
    Ok(())
}

/// Helper for filtering arrays
pub fn filter_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(array_param) = h.param(0) {
        if let Some(array) = array_param.value().as_array() {
            if let Some(key_param) = h.param(1) {
                if let Some(key) = key_param.value().as_str() {
                    if let Some(value_param) = h.param(2) {
                        // Filter items where item[key] == value
                        let filtered: Vec<&Value> = array
                            .iter()
                            .filter(|item| {
                                if let Some(obj) = item.as_object() {
                                    if let Some(item_value) = obj.get(key) {
                                        return item_value == value_param.value();
                                    }
                                }
                                false
                            })
                            .collect();

                        out.write(&serde_json::to_string(&filtered).unwrap_or_default())?;
                        return Ok(());
                    }
                }
            }
        }
    }
    out.write("[]")?;
    Ok(())
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

/// Helper for JSON serialization
pub fn json_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value();
        if let Some(s) = value.as_str() {
            out.write(s)?;
        } else {
            let json_str = serde_json::to_string(value).unwrap_or_default();
            out.write(&json_str)?;
        }
    }
    Ok(())
}

/// Helper for finding items by field
pub fn find_by_field_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(array_param) = h.param(0) {
        if let Some(array) = array_param.value().as_array() {
            if let Some(field_param) = h.param(1) {
                if let Some(field) = field_param.value().as_str() {
                    if let Some(value_param) = h.param(2) {
                        // Find item where item[field] == value
                        for item in array {
                            if let Some(obj) = item.as_object() {
                                if let Some(item_value) = obj.get(field) {
                                    // Convert both values to compare properly
                                    let search_value = value_param.value();
                                    if item_value == search_value
                                        || (item_value.is_string()
                                            && search_value.is_string()
                                            && item_value.as_str() == search_value.as_str())
                                        || (item_value.is_number()
                                            && search_value.is_string()
                                            && item_value.to_string()
                                                == search_value.as_str().unwrap_or(""))
                                    {
                                        out.write(
                                            &serde_json::to_string(item).unwrap_or_default(),
                                        )?;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out.write("null")?;
    Ok(())
}

/// Helper for merging objects
pub fn merge_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(base_param) = h.param(0) {
        let mut result = base_param.value().clone();

        // Merge with additional parameters
        for i in 1..h.params().len() {
            if let Some(merge_param) = h.param(i) {
                if let (Some(base_obj), Some(merge_obj)) =
                    (result.as_object_mut(), merge_param.value().as_object())
                {
                    for (key, value) in merge_obj {
                        base_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        out.write(&serde_json::to_string(&result).unwrap_or_default())?;
    }
    Ok(())
}

/// Helper for selecting specific fields from objects
pub fn select_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(obj_param) = h.param(0) {
        let obj_value = obj_param.value();

        // Handle if the object is a JSON string that needs to be parsed
        let obj = if obj_value.is_string() {
            if let Ok(parsed) = serde_json::from_str::<Value>(obj_value.as_str().unwrap_or("{}")) {
                parsed
            } else {
                obj_value.clone()
            }
        } else {
            obj_value.clone()
        };

        if let Some(obj_map) = obj.as_object() {
            let mut result = serde_json::Map::new();

            // Select specified fields
            for i in 1..h.params().len() {
                if let Some(field_param) = h.param(i) {
                    if let Some(field_name) = field_param.value().as_str() {
                        // Remove quotes if present
                        let field_name = field_name.trim_matches('"');
                        if let Some(field_value) = obj_map.get(field_name) {
                            result.insert(field_name.to_string(), field_value.clone());
                        }
                    }
                }
            }

            out.write(&serde_json::to_string(&result).unwrap_or_default())?;
        } else {
            out.write("{}")?;
        }
    }
    Ok(())
}

/// Helper for providing default values
pub fn default_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(value_param) = h.param(0) {
        let value = value_param.value();
        if value.is_null() || (value.is_string() && value.as_str().unwrap_or("").is_empty()) {
            // Use default value if provided
            if let Some(default_param) = h.param(1) {
                out.write(&serde_json::to_string(default_param.value()).unwrap_or_default())?;
            } else {
                out.write("null")?;
            }
        } else {
            out.write(&serde_json::to_string(value).unwrap_or_default())?;
        }
    } else {
        // No value provided, use default if available
        if let Some(default_param) = h.param(1) {
            out.write(&serde_json::to_string(default_param.value()).unwrap_or_default())?;
        } else {
            out.write("null")?;
        }
    }
    Ok(())
}

/// Helper for finding items by multiple fields
pub fn find_by_multi_field_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(array_param) = h.param(0) {
        if let Some(array) = array_param.value().as_array() {
            // Parse field-value pairs: array field1 value1 field2 value2 ...
            let params: Vec<_> = h.params().into_iter().collect();
            println!("🔍 find_by_multi_field called with {} params", params.len());
            
            if params.len() >= 3 && (params.len() - 1) % 2 == 0 {
                // Find item that matches all field-value pairs
                'item_loop: for item in array {
                    if let Some(obj) = item.as_object() {
                        println!("🔍 Checking item: {:?}", obj);
                        // Check all field-value pairs
                        for i in (1..params.len()).step_by(2) {
                            if let (Some(field_param), Some(value_param)) =
                                (params.get(i), params.get(i + 1))
                            {
                                if let Some(field) = field_param.value().as_str() {
                                    if let Some(item_value) = obj.get(field) {
                                        let search_value = value_param.value();
                                        println!("🔍 Comparing field '{}': item_value = {:?}, search_value = {:?}", 
                                                field, item_value, search_value);

                                        // Compare values with type conversion
                                        let matches = item_value == search_value
                                            || (item_value.is_string()
                                                && search_value.is_string()
                                                && item_value.as_str() == search_value.as_str())
                                            || (item_value.is_number()
                                                && search_value.is_string()
                                                && item_value.to_string()
                                                    == search_value.as_str().unwrap_or(""));

                                        println!("🔍 Field '{}' matches: {}", field, matches);
                                        if !matches {
                                            continue 'item_loop; // This item doesn't match, try next
                                        }
                                    } else {
                                        continue 'item_loop; // Field not found, try next item
                                    }
                                }
                            }
                        }

                        // If we get here, all conditions matched
                        out.write(&serde_json::to_string(item).unwrap_or_default())?;
                        return Ok(());
                    }
                }
            }
        }
    }
    out.write("null")?;
    Ok(())
}

/// Helper for generating random strings
pub fn random_string_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let length = h.param(0).and_then(|v| v.value().as_u64()).unwrap_or(10) as usize;

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    let hash = hasher.finish();

    // Generate random string using hash as seed
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut result = String::new();
    let mut current_hash = hash;

    for _ in 0..length {
        let index = (current_hash % chars.len() as u64) as usize;
        result.push(chars.chars().nth(index).unwrap_or('A'));
        current_hash = current_hash.wrapping_mul(1103515245).wrapping_add(12345);
        // Simple LCG
    }

    out.write(&result)?;
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

/// Equality check (==)
pub fn eq_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).map(|p| p.value()).cloned().unwrap_or(Value::Null);
    let b = h.param(1).map(|p| p.value()).cloned().unwrap_or(Value::Null);
    out.write(if a == b { "true" } else { "false" })?;
    Ok(())
}

/// Inequality check (!=)
pub fn ne_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).map(|p| p.value()).cloned().unwrap_or(Value::Null);
    let b = h.param(1).map(|p| p.value()).cloned().unwrap_or(Value::Null);
    out.write(if a != b { "true" } else { "false" })?;
    Ok(())
}

/// Greater than
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

/// String/array contains
pub fn contains_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let hay = h.param(0).map(|p| p.value()).cloned().unwrap_or(Value::Null);
    let needle = h.param(1).map(|p| p.value()).cloned().unwrap_or(Value::Null);
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

/// Existence check (not null/empty)
pub fn exists_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        out.write(if is_truthy(param.value()) { "true" } else { "false" })?;
    } else {
        out.write("false")?;
    }
    Ok(())
}
