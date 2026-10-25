use json2ts_native_fixture::native_core::{
    common::{CoreContext, CorePaths},
    extensions::Sessions,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
fn ctx() -> (tempfile::TempDir, CoreContext) {
    let d = tempfile::tempdir().unwrap();
    let c = CoreContext::new(
        CorePaths {
            data_dir: d.path().join("data"),
            config_dir: d.path().join("config"),
            resource_dir: d.path().join("resource"),
            temp_dir: d.path().join("tmp"),
            home_dir: d.path().into(),
        },
        None,
    );
    (d, c)
}
async fn launch(s: &Sessions, c: &CoreContext) -> Value {
    s.call(c,"extension:run-command",json!({"extensionId":"com.tezbar.native.json2ts","commandName":"index","argumentValues":{}})).await
}
#[tokio::test]
async fn actual_sessions_annotate_isolate_refresh_dispose_and_clear() {
    let (_d, c) = ctx();
    let s = Sessions::default();
    let a = launch(&s, &c).await;
    let b = launch(&s, &c).await;
    assert_eq!(a["extensionId"], "com.tezbar.native.json2ts");
    assert_eq!(a["commandName"], "index");
    assert_ne!(a["sessionId"], b["sessionId"]);
    let id = a["sessionId"].clone();
    let action = a["actions"][0]["id"].clone();
    for op in ["extension:refresh-session", "extension:load-more"] {
        assert_eq!(
            s.call(&c, op, json!({"sessionId":id})).await,
            json!({"ok":true,"mode":"unchanged"})
        );
    }
    assert_eq!(s.call(&c,"extension:invoke-action",json!({"sessionId":b["sessionId"],"actionId":action,"formValues":{"jsonTextarea":"{}","rootName":"r"}})).await["ok"],false);
    assert_eq!(
        s.call(
            &c,
            "extension:search-text-changed",
            json!({"sessionId":id,"searchText":"x"})
        )
        .await["ok"],
        false
    );
    assert_eq!(
        s.call(&c, "extension:dispose-session", json!({"sessionId":id}))
            .await,
        true
    );
    assert_eq!(
        s.call(&c, "extension:dispose-session", json!({"sessionId":id}))
            .await,
        false
    );
    for op in [
        "extension:run-command",
        "extension:invoke-action",
        "extension:refresh-session",
    ] {
        assert_eq!(s.call(&c, op, json!({"sessionId":id})).await["ok"], false);
    }
    s.clear().await;
    assert_eq!(
        s.call(
            &c,
            "extension:refresh-session",
            json!({"sessionId":b["sessionId"]})
        )
        .await["ok"],
        false
    );
}
#[tokio::test]
async fn actual_sessions_action_routes_once_and_rejects_replay() {
    let (_d, c) = ctx();
    let calls = Arc::new(Mutex::new(vec![]));
    let record = calls.clone();
    c.set_router(Arc::new(move |ch, args| {
        record.lock().unwrap().push((ch, args));
        Box::pin(async { Ok(json!({"ok":true})) })
    }))
    .unwrap();
    let s = Sessions::default();
    let a = launch(&s, &c).await;
    let body = json!({"sessionId":a["sessionId"],"actionId":a["actions"][0]["id"],"formValues":{"jsonTextarea":"{a:1}","rootName":"r"}});
    assert_eq!(
        s.call(&c, "extension:invoke-action", body.clone()).await["mode"],
        "view"
    );
    assert_eq!(
        s.call(&c, "extension:invoke-action", body).await["ok"],
        false
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            (
                "clipboard:write".into(),
                vec![json!("interface R {\n    a: number;\n}\n")]
            ),
            ("window:hide".into(), vec![])
        ]
    );
}
#[tokio::test]
async fn actual_sessions_recheck_install_and_capacity_without_eviction() {
    let (_d, c) = ctx();
    let s = Sessions::default();
    let a = launch(&s, &c).await;
    for _ in 1..128 {
        assert_eq!(launch(&s, &c).await["mode"], "view");
    }
    assert_eq!(launch(&s, &c).await["ok"], false);
    assert_eq!(
        s.call(
            &c,
            "extension:refresh-session",
            json!({"sessionId":a["sessionId"]})
        )
        .await["mode"],
        "unchanged"
    );
    std::fs::create_dir_all(&c.paths.data_dir).unwrap();
    std::fs::write(c.paths.data_dir.join("uninstalled"), b"").unwrap();
    assert_eq!(
        s.call(
            &c,
            "extension:invoke-action",
            json!({"sessionId":a["sessionId"],"actionId":a["actions"][0]["id"]})
        )
        .await["ok"],
        false
    );
    assert_eq!(
        s.call(
            &c,
            "extension:refresh-session",
            json!({"sessionId":a["sessionId"]})
        )
        .await["ok"],
        false
    );
    s.clear().await;
}
#[tokio::test]
async fn actual_sessions_cancel_inflight_effect_and_busy_refresh_without_os_calls() {
    let (_d, c) = ctx();
    let entered = Arc::new(tokio::sync::Notify::new());
    let notify = entered.clone();
    c.set_router(Arc::new(move |ch, _| {
        assert_eq!(ch, "clipboard:write");
        let n = notify.clone();
        Box::pin(async move {
            n.notify_one();
            std::future::pending().await
        })
    }))
    .unwrap();
    let s = Arc::new(Sessions::default());
    let a = launch(&s, &c).await;
    let (s2, c2) = (s.clone(), c.clone());
    let body = json!({"sessionId":a["sessionId"],"actionId":a["actions"][0]["id"],"formValues":{"jsonTextarea":"{}","rootName":"r"}});
    let pending = tokio::spawn(async move { s2.call(&c2, "extension:invoke-action", body).await });
    entered.notified().await;
    assert_eq!(
        s.call(
            &c,
            "extension:refresh-session",
            json!({"sessionId":a["sessionId"]})
        )
        .await["mode"],
        "unchanged"
    );
    tokio::time::timeout(std::time::Duration::from_secs(2), s.clear())
        .await
        .unwrap();
    assert_eq!(pending.await.unwrap()["ok"], false);
}
#[test]
fn actual_beta_catalog_computes_coverage_from_registration() {
    use json2ts_native_fixture::native_core::extensions::{catalog, v2_runtime};
    let manifest: Value = serde_json::from_str(include_str!("../../../manifest.json")).unwrap();
    assert_eq!(manifest["commands"][0]["name"], "index");
    for id in ["json2ts", "raycast.json2ts", "com.tezbar.native.json2ts"] {
        assert_eq!(v2_runtime::slug(id), Some("json2ts"));
        assert!(v2_runtime::supports(id, "index"));
        assert!(!v2_runtime::supports(id, "forged"));
    }
    assert_eq!(v2_runtime::commands("json2ts"), &["index"]);
    assert_eq!(
        v2_runtime::canonical_id("json2ts"),
        manifest["id"].as_str().unwrap()
    );
    let item = catalog("json2ts").unwrap();
    assert_eq!(item["status"], "complete");
    assert_eq!(item["totalCommands"], 1);
    assert_eq!(item["commands"][0]["runtime"], "rust");
    assert_eq!(item["verification"], manifest["verification"]);
    let missing = catalog("json2ts-uncompiled").unwrap();
    assert_eq!(missing["status"], "unavailable");
    assert_eq!(missing["commands"][0]["runtime"], "unavailable");
}
