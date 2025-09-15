use chrono::{DateTime, Utc};
use fake::{
    faker::{internet::en::FreeEmail, lorem::en::Sentence, name::en::Name},
    Fake,
};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use uuid::Uuid;

/// Register faker-related helpers
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("now", Box::new(now_helper));
    handlebars.register_helper("random", Box::new(random_helper));
    handlebars.register_helper("random_string", Box::new(random_string_helper));
    handlebars.register_helper("faker", Box::new(faker_helper));
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
    let key = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

    let value = match key {
        "internet.email" => FreeEmail().fake::<String>(),
        "person.name" => Name().fake::<String>(),
        "lorem.sentence" => Sentence(3..6).fake::<String>(),
        _ => String::new(),
    };

    out.write(&value)?;
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
    }

    out.write(&result)?;
    Ok(())
}
