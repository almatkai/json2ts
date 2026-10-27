//! Production compiled-in beta adapter. OS boundaries are CoreContext routes.
use super::v2_sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use tezbar_json2ts::{convert, MAX_INPUT_BYTES, MAX_NAME_BYTES};

pub fn commands() -> &'static [&'static str] {
    &["index"]
}
fn fresh() -> String {
    // Process-global monotonic nonce is not rolled back with a staged search clone.
    static NONCE: AtomicU64 = AtomicU64::new(1);
    format!(
        "json2ts:{}:{}",
        uuid::Uuid::new_v4(),
        NONCE.fetch_add(1, Ordering::Relaxed)
    )
}
fn new_submit(state: &mut Value) {
    state["submit"] = json!(fresh());
}
fn form(state: &Value, changed: Option<&str>, initial: bool) -> Value {
    let mut fields = Vec::new();
    for (id, kind, title) in [
        ("jsonTextarea", "Form.TextArea", "JSON"),
        ("rootName", "Form.TextField", "Interface Name"),
    ] {
        let mut props = json!({"id":id,"title":title,"actionId":state["fields"][id]});
        if id == "jsonTextarea" {
            props["placeholder"] = json!("Enter JSON here");
        }
        // The beta reconciler treats repeated controlled values as authoritative.
        // Omit all unrelated values to retain newer queued local drafts.
        if initial || changed == Some(id) {
            props["value"] = state["draft"][id].clone();
        }
        fields.push(json!({"type":kind,"props":props}));
    }
    let mut actions = Vec::new();
    if let Some(retry) = state["retry"].as_str() {
        actions.push(json!({"id":retry,"title":if state["copied"]==true {"Retry Hide Window"}else{"Retry Copy"},"kind":"action"}));
    }
    actions.push(json!({"id":state["submit"],"title":"Generate","kind":"submit-form"}));
    if let Some(message) = state["message"].as_str() {
        fields.push(json!({"type":"Form.Description","props":{"text":message}}));
    }
    let mut view = v2_sdk::view(
        "Json2ts",
        json!({"type":"Form","props":{},"children":fields}),
        actions,
    );
    if let Some(message) = state["message"].as_str() {
        view["message"] = json!(message);
    }
    view
}
fn field_value<'a>(payload: &'a Value, key: &str, limit: usize) -> Result<&'a str, String> {
    let text = payload
        .get(key)
        .and_then(Value::as_str)
        .ok_or("Invalid form values")?;
    if text.len() > limit {
        return Err(if limit == MAX_NAME_BYTES {
            "Interface name exceeds 16 KiB limit"
        } else {
            "JSON exceeds 1 MiB limit"
        }
        .into());
    }
    Ok(text)
}
fn acknowledged(result: CoreResult) -> bool {
    let Ok(Value::Object(object)) = result else {
        return false;
    };
    object.get("ok") == Some(&json!(true))
        && object.get("error").is_none_or(Value::is_null)
        && object.get("success").is_none_or(|v| v == &json!(true))
}
async fn finish(ctx: &CoreContext, state: &mut Value) -> CoreResult {
    let output = state["output"]
        .as_str()
        .ok_or("No generated output available")?
        .to_owned();
    if state["copied"] != true {
        // Save uncertain state BEFORE awaiting. A cancelled request must not replay.
        state["copyUncertain"] = json!(true);
        state["retry"] = json!(fresh());
        if !acknowledged(v2_sdk::copy(ctx, &output).await) {
            state["message"]=json!("Copy failed or its outcome is uncertain. Retry Copy deliberately; no success was confirmed.");
            state["retry"] = json!(fresh());
            return Ok(form(state, None, false));
        }
        state["copied"] = json!(true);
        state["acknowledgedOutput"] = json!(output);
        state["copyUncertain"] = json!(false);
    }
    state["hideUncertain"] = json!(true);
    state["retry"] = json!(fresh());
    if !acknowledged(ctx.request("window:hide", vec![]).await) {
        state["message"]=json!("Copied to clipboard. Hide window failed or is uncertain; Retry Hide Window will not copy again.");
        state["retry"] = json!(fresh());
        return Ok(form(state, None, false));
    }
    state["hideUncertain"] = json!(false);
    state["retry"] = Value::Null;
    state["message"] = json!("Copied to clipboard");
    Ok(form(state, None, false))
}
pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if command != "index" {
        return Err("Unsupported Json2TS command".into());
    }
    if operation == "extension:run-command" {
        if state.get("fields").is_some() {
            return Err("Json2TS session already launched".into());
        }
        *state = json!({"fields":{"jsonTextarea":fresh(),"rootName":fresh()},"draft":{"jsonTextarea":"","rootName":"rootName"},"submit":fresh(),"copied":false});
        return Ok(form(state, None, true));
    }
    if operation == "extension:dispose-session" {
        *state = json!({});
        return Ok(v2_sdk::unchanged());
    }
    if state.get("fields").is_none() {
        return Err("Json2TS session is not active".into());
    }
    match operation {
        "extension:refresh-session" | "extension:load-more" => Ok(v2_sdk::unchanged()),
        "extension:invoke-action" => {
            let id = payload["actionId"].as_str().ok_or("Missing action ID")?;
            let field = ["jsonTextarea", "rootName"]
                .into_iter()
                .find(|key| state["fields"][*key].as_str() == Some(id));
            if let Some(key) = field {
                let values = payload["formValues"]
                    .as_object()
                    .ok_or("Invalid field callback")?;
                let value = field_value(
                    &json!(values),
                    "value",
                    if key == "rootName" {
                        MAX_NAME_BYTES
                    } else {
                        MAX_INPUT_BYTES
                    },
                )?
                .to_owned();
                state["draft"][key] = json!(value);
                if state["retry"].is_null() {
                    state["message"] = Value::Null;
                }
                // Editing is explicit; retain effect checkpoints until the next Generate.
                return Ok(form(state, Some(key), false));
            }
            if state["retry"].as_str() == Some(id) {
                state["retry"] = Value::Null;
                new_submit(state);
                return finish(ctx, state).await;
            }
            if state["submit"].as_str() != Some(id) {
                return Err("Unknown or stale Json2TS action".into());
            }
            let values = payload["formValues"]
                .as_object()
                .ok_or("Invalid form values")?;
            let values = json!(values);
            let input = field_value(&values, "jsonTextarea", MAX_INPUT_BYTES)?.to_owned();
            let root = field_value(&values, "rootName", MAX_NAME_BYTES)?.to_owned();
            let output = convert(&input, &root)?;
            state["draft"] = json!({"jsonTextarea":input,"rootName":root});
            state["output"] = json!(output);
            // Last acknowledged checkpoint stays inspectable even after a later failure.
            if state["copied"] == true {
                state["lastAcknowledgedOutput"] = state["acknowledgedOutput"].clone();
            }
            state["copied"] = json!(false);
            state["message"] = Value::Null;
            state["retry"] = Value::Null;
            new_submit(state);
            let result = finish(ctx, state).await;
            if state["copied"] == true {
                state["acknowledgedOutput"] = state["output"].clone();
            }
            result
        }
        _ => Err("Unsupported Json2TS operation".into()),
    }
}
