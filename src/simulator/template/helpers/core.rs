use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde_json::Value;

pub fn register_core_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("find", Box::new(find_helper));
    handlebars.register_helper("find_by_field", Box::new(find_by_field_helper));
    handlebars.register_helper("find_by_multi_field", Box::new(find_by_multi_field_helper));
    handlebars.register_helper("filter", Box::new(filter_helper));
    handlebars.register_helper("json", Box::new(json_helper));
    handlebars.register_helper("merge", Box::new(merge_helper));
    handlebars.register_helper("select", Box::new(select_helper));
    handlebars.register_helper("default", Box::new(default_helper));
}

/// Helper for getting array length
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

            if params.len() >= 3 && (params.len() - 1) % 2 == 0 {
                // Find item that matches all field-value pairs
                'item_loop: for item in array {
                    if let Some(obj) = item.as_object() {
                        // Check all field-value pairs
                        for i in (1..params.len()).step_by(2) {
                            if let (Some(field_param), Some(value_param)) =
                                (params.get(i), params.get(i + 1))
                            {
                                if let Some(field) = field_param.value().as_str() {
                                    if let Some(item_value) = obj.get(field) {
                                        let search_value = value_param.value();

                                        // Compare values with type conversion
                                        let matches = item_value == search_value
                                            || (item_value.is_string()
                                                && search_value.is_string()
                                                && item_value.as_str() == search_value.as_str())
                                            || (item_value.is_number()
                                                && search_value.is_string()
                                                && item_value.to_string()
                                                    == search_value.as_str().unwrap_or(""));

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
