use crate::native_core::{
    common::{CoreContext, CorePaths, CoreResult},
    extensions::v2_json2ts::handle,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
type Calls = Arc<Mutex<Vec<(String, Vec<Value>)>>>;
fn context(responses: Vec<CoreResult>) -> (tempfile::TempDir, CoreContext, Calls) {
    let dir = tempfile::tempdir().unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let record = calls.clone();
    let ctx = CoreContext::headless(CorePaths {
        data_dir: dir.path().join("data"),
        config_dir: dir.path().join("config"),
        resource_dir: dir.path().join("resource"),
        temp_dir: dir.path().join("temp"),
        home_dir: dir.path().into(),
    });
    let responses = Arc::new(Mutex::new(std::collections::VecDeque::from(responses)));
    ctx.set_router(Arc::new(move |ch, args| {
        record.lock().unwrap().push((ch.clone(), args));
        assert!(["clipboard:write", "window:hide"].contains(&ch.as_str()));
        let r = responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected OS effect");
        Box::pin(async move { r })
    }))
    .unwrap();
    (dir, ctx, calls)
}
async fn launch(ctx: &CoreContext) -> (Value, Value) {
    let mut s = json!({});
    let v = handle(
        ctx,
        "index",
        "extension:run-command",
        &json!({"argumentValues":{"jsonTextarea":"ignored"},"preferences":{"ignored":true}}),
        &mut s,
    )
    .await
    .unwrap();
    (s, v)
}
async fn invoke(ctx: &CoreContext, s: &mut Value, id: Value, values: Value) -> CoreResult {
    handle(
        ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":id,"formValues":values,"text":"FORGED COPY TARGET","output":"FORGED"}),
        s,
    )
    .await
}
fn values() -> Value {
    json!({"jsonTextarea":"{\"x\":1}","rootName":"rootName"})
}
fn good() -> CoreResult {
    Ok(json!({"ok":true}))
}
#[tokio::test]
async fn initial_fields_and_actual_generate_effect_order() {
    let (_d, c, calls) = context(vec![good(), good()]);
    let (mut s, v) = launch(&c).await;
    assert_eq!(v["root"]["children"][0]["props"]["id"], "jsonTextarea");
    assert_eq!(v["root"]["children"][1]["props"]["value"], "rootName");
    assert_eq!(v["actions"][0]["title"], "Generate");
    assert_eq!(v["actions"][0]["kind"], "submit-form");
    let id = s["submit"].clone();
    let result = invoke(&c, &mut s, id.clone(), values()).await.unwrap();
    assert_eq!(result["mode"], "view");
    assert_eq!(result["message"], "Copied to clipboard");
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            (
                "clipboard:write".into(),
                vec![json!("interface RootName {\n    x: number;\n}\n")]
            ),
            ("window:hide".into(), vec![])
        ]
    );
    assert!(invoke(&c, &mut s, id, values()).await.is_err());
    assert_eq!(calls.lock().unwrap().len(), 2);
    assert!(!c.paths.data_dir.exists()); // Source needs no storage or migration.
}
#[tokio::test]
async fn validation_order_and_malformed_events_preserve_bindings() {
    let (_d, c, calls) = context(vec![]);
    let (mut s, _) = launch(&c).await;
    let id = s["submit"].clone();
    for (v, error) in [
        (json!({"jsonTextarea":"","rootName":""}), "Empty JSON"),
        (
            json!({"jsonTextarea":"{}","rootName":""}),
            "Empty interface name",
        ),
        (
            json!({"jsonTextarea":"private data","rootName":"r"}),
            "Invalid JSON",
        ),
    ] {
        assert_eq!(invoke(&c, &mut s, id.clone(), v).await.unwrap_err(), error);
        assert_eq!(s["submit"], id);
    }
    let old = s.clone();
    for (action, v) in [
        (json!("forged"), values()),
        (id.clone(), json!(null)),
        (id.clone(), json!({"jsonTextarea":1,"rootName":"r"})),
        (
            s["fields"]["rootName"].clone(),
            json!({"rootName":"bad-wire"}),
        ),
        (s["fields"]["jsonTextarea"].clone(), json!({"value":42})),
    ] {
        assert!(invoke(&c, &mut s, action, v).await.is_err());
        assert_eq!(s, old);
    }
    assert!(calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn stable_fields_and_omitted_values_retain_unrelated_drafts() {
    let (_d, c, _calls) = context(vec![]);
    let (mut s, _) = launch(&c).await;
    let fields = s["fields"].clone();
    let response = invoke(
        &c,
        &mut s,
        fields["jsonTextarea"].clone(),
        json!({"value":"new JSON draft"}),
    )
    .await
    .unwrap();
    assert_eq!(
        response["root"]["children"][0]["props"]["value"],
        "new JSON draft"
    );
    assert!(response["root"]["children"][1]["props"]
        .get("value")
        .is_none());
    let response = invoke(
        &c,
        &mut s,
        fields["rootName"].clone(),
        json!({"value":"newName"}),
    )
    .await
    .unwrap();
    assert!(response["root"]["children"][0]["props"]
        .get("value")
        .is_none());
    assert_eq!(s["fields"], fields);
    let before = s.clone();
    for op in ["extension:refresh-session", "extension:load-more"] {
        assert_eq!(
            handle(&c, "index", op, &json!({}), &mut s).await.unwrap()["mode"],
            "unchanged"
        );
        assert_eq!(s, before);
    }
    assert!(handle(
        &c,
        "index",
        "extension:search-text-changed",
        &json!({"searchText":"discarded"}),
        &mut s.clone()
    )
    .await
    .is_err());
    assert_eq!(s, before);
}
fn bad_responses() -> Vec<CoreResult> {
    vec![
        Ok(Value::Null),
        Ok(json!(true)),
        Ok(json!(false)),
        Ok(json!([])),
        Ok(json!("ok")),
        Ok(json!({})),
        Ok(json!({"ok":false})),
        Ok(json!({"ok":"true"})),
        Ok(json!({"ok":true,"error":"private /Users/secret"})),
        Ok(json!({"ok":true,"error":false})),
        Ok(json!({"ok":true,"success":false})),
        Ok(json!({"ok":true,"success":null})),
        Err("private /Users/secret".into()),
    ]
}
#[tokio::test]
async fn strict_copy_acknowledgements_require_deliberate_retry() {
    for bad in bad_responses() {
        let (_d, c, calls) = context(vec![bad, good(), good()]);
        let (mut s, _) = launch(&c).await;
        let old = s["submit"].clone();
        let v = invoke(&c, &mut s, old.clone(), values()).await.unwrap();
        assert_eq!(v["mode"], "view");
        assert!(v["message"].as_str().unwrap().contains("Copy failed"));
        assert!(!v.to_string().contains("/Users/secret"));
        assert_eq!(s["copied"], false);
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert!(invoke(&c, &mut s, old, values()).await.is_err());
        handle(&c, "index", "extension:refresh-session", &json!({}), &mut s)
            .await
            .unwrap();
        assert_eq!(calls.lock().unwrap().len(), 1);
        let retry = v["actions"][0]["id"].clone();
        let result = invoke(
            &c,
            &mut s,
            retry,
            json!({"jsonTextarea":"FORGED","rootName":"evil"}),
        )
        .await
        .unwrap();
        assert_eq!(result["mode"], "view");
        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded[0].1, recorded[1].1); // canonical output
    }
}
#[tokio::test]
async fn acknowledged_copy_survives_every_hide_failure() {
    for bad in bad_responses() {
        let (_d, c, calls) = context(vec![good(), bad, good()]);
        let (mut s, _) = launch(&c).await;
        let id = s["submit"].clone();
        let v = invoke(&c, &mut s, id, values()).await.unwrap();
        assert_eq!(s["copied"], true);
        assert_eq!(s["acknowledgedOutput"], s["output"]);
        assert!(v["message"]
            .as_str()
            .unwrap()
            .contains("Copied to clipboard"));
        assert_eq!(v["actions"][0]["title"], "Retry Hide Window");
        let retry = s["retry"].clone();
        assert_eq!(
            invoke(&c, &mut s, retry, json!({})).await.unwrap()["mode"],
            "view"
        );
        assert_eq!(
            calls
                .lock()
                .unwrap()
                .iter()
                .filter(|(ch, _)| ch == "clipboard:write")
                .count(),
            1
        );
        assert_eq!(calls.lock().unwrap().len(), 3);
    }
}
#[tokio::test]
async fn scoped_fresh_ids_and_disposal() {
    let (_d, c, _) = context(vec![]);
    let (mut a, _) = launch(&c).await;
    let (mut b, _) = launch(&c).await;
    let before = b.clone();
    assert!(invoke(&c, &mut b, a["submit"].clone(), values())
        .await
        .is_err());
    assert_eq!(b, before);
    let old = a["fields"]["jsonTextarea"].clone();
    handle(&c, "index", "extension:dispose-session", &json!({}), &mut a)
        .await
        .unwrap();
    assert_eq!(a, json!({}));
    assert!(invoke(&c, &mut a, old.clone(), json!({"value":"x"}))
        .await
        .is_err());
    handle(&c, "index", "extension:run-command", &json!({}), &mut a)
        .await
        .unwrap();
    assert_ne!(a["fields"]["jsonTextarea"], old);
    assert!(invoke(&c, &mut a, old, json!({"value":"x"})).await.is_err());
}
#[tokio::test]
async fn cancellation_keeps_uncertain_checkpoint_and_never_repeats_on_heartbeat() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = CoreContext::headless(CorePaths {
        data_dir: dir.path().into(),
        config_dir: dir.path().into(),
        resource_dir: dir.path().into(),
        temp_dir: dir.path().into(),
        home_dir: dir.path().into(),
    });
    ctx.set_router(Arc::new(|ch, _| {
        assert_eq!(ch, "clipboard:write");
        Box::pin(std::future::pending())
    }))
    .unwrap();
    let (mut s, _) = launch(&ctx).await;
    let old = s["submit"].clone();
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(10),
        invoke(&ctx, &mut s, old.clone(), values())
    )
    .await
    .is_err());
    assert_eq!(s["copyUncertain"], true);
    assert!(s["retry"].is_string());
    assert!(invoke(&ctx, &mut s, old, values()).await.is_err());
    assert_eq!(
        handle(
            &ctx,
            "index",
            "extension:refresh-session",
            &json!({}),
            &mut s
        )
        .await
        .unwrap()["mode"],
        "unchanged"
    );
}
#[tokio::test]
async fn export_actual_adapter_wire_for_renderer() {
    let (_d, c, _) = context(vec![]);
    let (mut s, initial) = launch(&c).await;
    let root_id = s["fields"]["rootName"].clone();
    let changed = invoke(&c, &mut s, root_id, json!({"value":"ChangedRoot"}))
        .await
        .unwrap();
    let (_d, fail_ctx, _) = context(vec![Ok(Value::Null)]);
    let (mut failed, _) = launch(&fail_ctx).await;
    let id = failed["submit"].clone();
    let failure = invoke(&fail_ctx, &mut failed, id, values()).await.unwrap();
    if let Ok(path) = std::env::var("JSON2TS_UI_FIXTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(
                &json!({"initial":initial,"changed":changed,"failure":failure}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
#[tokio::test]
async fn accepted_acknowledgement_shapes_and_completed_form_remain_usable() {
    for ack in [
        json!({"ok":true}),
        json!({"ok":true,"success":true}),
        json!({"ok":true,"error":null,"success":true}),
    ] {
        let (_d, c, calls) = context(vec![Ok(ack.clone()), Ok(ack), good(), good()]);
        let (mut s, _) = launch(&c).await;
        let id = s["submit"].clone();
        let result = invoke(&c, &mut s, id, values()).await.unwrap();
        assert_eq!(result["message"], "Copied to clipboard");
        assert_eq!(result["actions"][0]["id"], s["submit"]);
        let next = s["submit"].clone();
        let result = invoke(
            &c,
            &mut s,
            next,
            json!({"jsonTextarea":"[]","rootName":"r"}),
        )
        .await
        .unwrap();
        assert_eq!(result["message"], "Copied to clipboard");
        assert_eq!(
            s["lastAcknowledgedOutput"],
            "interface RootName {\n    x: number;\n}\n"
        );
        assert_eq!(calls.lock().unwrap().len(), 4);
    }
}
#[tokio::test]
async fn cancelled_hide_preserves_acknowledged_copy_and_exposes_only_deliberate_retry() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = CoreContext::headless(CorePaths {
        data_dir: dir.path().into(),
        config_dir: dir.path().into(),
        resource_dir: dir.path().into(),
        temp_dir: dir.path().into(),
        home_dir: dir.path().into(),
    });
    let calls = Arc::new(Mutex::new(vec![]));
    let record = calls.clone();
    ctx.set_router(Arc::new(move |ch, _| {
        record.lock().unwrap().push(ch.clone());
        Box::pin(async move {
            match ch.as_str() {
                "clipboard:write" => Ok(json!({"ok":true})),
                "window:hide" => std::future::pending().await,
                _ => panic!("unexpected route"),
            }
        })
    }))
    .unwrap();
    let (mut s, _) = launch(&ctx).await;
    let id = s["submit"].clone();
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(10),
        invoke(&ctx, &mut s, id, values())
    )
    .await
    .is_err());
    assert_eq!(s["copied"], true);
    assert_eq!(s["acknowledgedOutput"], s["output"]);
    assert_eq!(s["hideUncertain"], true);
    let field = s["fields"]["rootName"].clone();
    let view = invoke(&ctx, &mut s, field, json!({"value":"new draft"}))
        .await
        .unwrap();
    assert_eq!(view["actions"][0]["title"], "Retry Hide Window");
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["clipboard:write", "window:hide"]
    );
}
