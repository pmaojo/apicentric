use regex::Regex;

/// Handles preprocessing of template strings before rendering.
#[derive(Default)]
pub struct TemplatePreprocessor;

impl TemplatePreprocessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-process template to convert custom syntax to Handlebars helpers
    pub fn preprocess(&self, template: &str) -> String {
        // Normalize bucket helper names
        let bucket_regex = Regex::new(r"\{\{\s*bucket\.(set|get)").unwrap();
        let mut result = bucket_regex
            .replace_all(template, "{{bucket_$1")
            .to_string();

        // Handle simple cases first - just return fixtures as JSON
        let simple_fixture_regex = Regex::new(r"\{\{\s*fixtures\.(\w+)\s*\}\}").unwrap();
        result = simple_fixture_regex
            .replace_all(&result, "{{json fixtures.$1}}")
            .to_string();

        // Handle complex pipe operations with multiple pipes
        let complex_pipe_regex = Regex::new(r"\{\{\s*([^{}]+?)\s*\}\}").unwrap();

        result = complex_pipe_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let content = caps.get(1).unwrap().as_str().trim();

                // Skip if it's already processed (contains json, length, etc.)
                if content.starts_with("json ")
                    || content.starts_with("length ")
                    || content.starts_with("find_by_field ")
                    || content.starts_with("merge ")
                {
                    return format!("{{{{{}}}}}", content);
                }

                // Handle 'not (...)' expressions
                if content.starts_with("not (") && content.ends_with(")") {
                    let inner_content = &content[5..content.len() - 1];
                    if inner_content.contains(" | ") {
                        let inner_processed = Self::process_pipe_expression(inner_content);
                        return format!("{{{{not ({})}}}}", inner_processed);
                    } else {
                        return format!("{{{{not {}}}}}", inner_content);
                    }
                }

                // Parse pipe operations
                if content.contains(" | ") {
                    let processed = Self::process_pipe_expression(content);
                    return format!("{{{{{}}}}}", processed);
                }

                // Handle simple non-piped expressions
                Self::process_simple_expression(content)
            })
            .to_string();

        result
    }

    /// Process pipe expressions like "fixtures.users | find(id=params.id)"
    fn process_pipe_expression(content: &str) -> String {
        let parts: Vec<&str> = content.split(" | ").collect();
        if parts.len() >= 2 {
            let mut current_value = parts[0].trim().to_string();

            for pipe_part in &parts[1..] {
                let pipe_part = pipe_part.trim();

                if pipe_part.starts_with("find(") && pipe_part.ends_with(")") {
                    let args_content = &pipe_part[5..pipe_part.len() - 1];
                    let conditions: Vec<&str> = args_content.split(',').map(|s| s.trim()).collect();

                    if conditions.len() == 1 {
                        if let Some((key, val)) = conditions[0].split_once('=') {
                            current_value = format!(
                                "find_by_field {} \"{}\" {}",
                                current_value,
                                key.trim(),
                                val.trim()
                            );
                        }
                    } else {
                        let mut find_args = vec![current_value];
                        for condition in conditions {
                            if let Some((key, val)) = condition.split_once('=') {
                                find_args.push(format!("\"{}\"", key.trim()));
                                find_args.push(val.trim().to_string());
                            }
                        }
                        current_value = format!("find_by_multi_field {}", find_args.join(" "));
                    }
                } else if pipe_part.starts_with("select(") && pipe_part.ends_with(")") {
                    let args_content = &pipe_part[7..pipe_part.len() - 1];
                    let fields: Vec<&str> = args_content.split(',').map(|s| s.trim()).collect();
                    let field_args = fields.join(" ");
                    current_value = format!("select ({}) {}", current_value, field_args);
                } else if pipe_part == "length" {
                    current_value = format!("length {}", current_value);
                } else if pipe_part.starts_with("merge(") && pipe_part.ends_with(")") {
                    let args_content = &pipe_part[6..pipe_part.len() - 1];
                    current_value = format!("merge {} {}", current_value, args_content);
                } else if pipe_part.starts_with("default(") && pipe_part.ends_with(")") {
                    let args_content = &pipe_part[8..pipe_part.len() - 1];
                    current_value = format!("default {} {}", current_value, args_content);
                }
            }

            current_value
        } else {
            content.to_string()
        }
    }

    /// Process simple expressions without pipes
    fn process_simple_expression(content: &str) -> String {
        if content.starts_with("fixtures.") {
            format!("{{{{json {}}}}}", content)
        } else if content.starts_with("params.")
            || content.starts_with("request.")
            || content.starts_with("runtime.")
        {
            format!("{{{{{}}}}}", content)
        } else if content.contains("random_string(") {
            let random_regex = Regex::new(r"random_string\((\d+)\)").unwrap();
            let processed = random_regex.replace_all(content, "random_string $1");
            format!("{{{{{}}}}}", processed)
        } else if content == "now()" {
            format!("{{{{now}}}}")
        } else {
            format!("{{{{{}}}}}", content)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_fixture_shortcut() {
        let pre = TemplatePreprocessor::new();
        let result = pre.preprocess("Hello {{fixtures.name}}");
        assert_eq!(result, "Hello {{json fixtures.name}}");
    }

    #[test]
    fn processes_pipe_expression() {
        let pre = TemplatePreprocessor::new();
        let result = pre.preprocess("{{fixtures.users | length}}");
        assert_eq!(result, "{{length fixtures.users}}");
    }
}
