use crate::simulator::service::state::DataBucket;
use handlebars::{Context, Handlebars, Helper, Output, RenderContext};

/// Register helpers for interacting with the shared data bucket
pub fn register_bucket_helpers(handlebars: &mut Handlebars, bucket: DataBucket) {
    let get_bucket = bucket.clone();
    handlebars.register_helper(
        "bucket_get",
        Box::new(
            move |h: &Helper,
                  _: &Handlebars,
                  _: &Context,
                  _: &mut RenderContext,
                  out: &mut dyn Output| {
                let key = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");
                if let Some(val) = get_bucket.get(key) {
                    out.write(&serde_json::to_string(&val).unwrap_or_default())?;
                } else {
                    out.write("null")?;
                }
                Ok(())
            },
        ),
    );

    let set_bucket = bucket.clone();
    handlebars.register_helper(
        "bucket_set",
        Box::new(
            move |h: &Helper,
                  _: &Handlebars,
                  _: &Context,
                  _: &mut RenderContext,
                  out: &mut dyn Output| {
                let key = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");
                if let Some(val) = h.param(1) {
                    set_bucket.set(key.to_string(), val.value().clone());
                }
                out.write("")?;
                Ok(())
            },
        ),
    );
}
