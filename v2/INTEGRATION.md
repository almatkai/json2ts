# Production beta integration (not applied)

Target beta: `fd300e276356587726566026f072563eb9c34cb1`. The dirty `fix/deep-search-state` host checkout was read only via `git show`. None of the following host edits, registration, profile installation or rebuilds have been performed.

1. Vendor the **contents of this v2 directory** at the stable path `src-tauri/native-extensions/json2ts/`. Keep its Cargo.lock, notices/provenance and icon. Do not point a production build at a temporary clone.
2. In `src-tauri/Cargo.toml`, add `tezbar-json2ts = { path = "native-extensions/json2ts" }` to dependencies. The core requires Rust 1.85+, serde_json 1.0.145, fancy-regex 0.14.0 and ryu-js 1.0.2, all locked. The adapter uses the host's existing serde_json and uuid dependencies and its existing SDK.
3. In `src-tauri/src/native_core/extensions/mod.rs`, add the sibling module:

   ```rust
   #[path = "../../../native-extensions/json2ts/adapter/v2_json2ts.rs"]
   mod v2_json2ts;
   ```

   `v2_json2ts` calls `super::v2_sdk` and `crate::native_core::common`; compile it inside that actual module hierarchy. Do not compile the headless harness as a production replacement for Tauri or the host router.
4. In `v2_runtime.rs`, extend the existing `slug` accepted-value match with `"json2ts"`; its existing prefix stripping supplies `raycast.json2ts` and `com.tezbar.native.json2ts`. Add these arms to the existing dispatch functions:

   ```rust
   // commands(slug)
   "json2ts" => super::v2_json2ts::commands(),
   // handle(ctx, slug, command, operation, payload, state)
   "json2ts" => super::v2_json2ts::handle(ctx, command, operation, payload, state).await,
   ```

   Preserve every unrelated arm. `canonical_id("json2ts")` must remain `com.tezbar.native.json2ts`; `supports` must accept only `index`. An installed marker/manifest is insufficient without these compiled dispatch arms.
5. Append `manifest.json` as an object in the actual bundled `src-tauri/src/native_core/extensions/v2_catalog.json`. Leave V1 intact. If a Recommended/catalog surface separately references this extension, retain its V1 entry and place the approved V2 replacement as appropriate; do not claim static JSON means executable coverage. No host catalog or ledger edits are part of this port.
6. Copy `assets/command-icon.png` **unchanged** to `src/renderer/public/extensions/icons/json2ts.png`. The manifest URL is exactly `/extensions/icons/json2ts.png`. Assert size 29082 and SHA256 `0b083ac11c306e7704f359a6d8b15a95a77f6a0b9f2ea76c6d628f8a3d43cd7c` after the copy.
7. Address the **known acknowledgement contract mismatch** in the host: pinned `platform.rs` lines 212–236 performs `window:hide` and returns `Ok(Value::Null)`. `native_core/mod.rs` forwards the result unchanged. SDK copy's `clipboard:write` correctly returns `{"ok":true}` (launcher/clipboard.rs lines 480–482). To satisfy this port's required strict contract, the host hide route must return an explicit `{"ok":true}` only after successful hiding, and propagate failures; it must not convert a transport error or arbitrary null response into success in the adapter. Preserve acknowledgement semantics for unrelated channels. That host change is outside this clone's authorized scope and remains unapplied. With the unchanged pinned route, the adapter correctly preserves copied output and shows “hide failed or is uncertain,” rather than claiming a confirmed hide.
8. Rebuild the host Rust binary and renderer. Through the actual compiled `v2_catalog`, assert the new entry has `status:complete`, `totalCommands:1`, command `index` runtime `rust` on each intended OS. Compare declared commands to `v2_json2ts::commands()` and canonical/alias dispatch. Increment any fixed catalog-count assertion only to reflect the host's actual final catalog. Both verification entries remain `not-run` until genuine QA artifacts exist.

The adapter signature is the beta's actual `handle(ctx, command, operation, payload, state) -> CoreResult`. It consumes the beta's `extension:run-command`, `extension:invoke-action`, `extension:search-text-changed`, `extension:refresh-session`, `extension:load-more`, `extension:dispose-session` operations. Launch arguments arrive under `argumentValues`, preferences under `preferences`; this source declares neither and does not consume them. Sessions, preference snapshot injection, cancellation and state serialization remain the host NativeSessions implementation's responsibility.

Only two OS routes are needed: SDK `copy` dispatches `clipboard:write` with `[computed_output]`; `window:hide` receives `[]` only after a strict copy acknowledgement. Copy and hide both demand object `ok:true`, `error` absent/null, and `success` absent/true. No clipboard-read, selection, paste, open, storage or unknown legacy profile migration is implemented because the source requires none. Do not add extra `paste-text`/`paste`/focus calls: those routes already own their OS orchestration and this command never pastes.

The beta renderer does not navigate to root on `no-view`. This adapter therefore returns the updated Form and inline status, with acknowledged hide for the HUD adaptation. It never claims to have popped navigation. Genuine HUD/toast, true pop-to-root and real OS focus/window behavior are still beta integration/QA gaps. Field IDs and callback IDs stay stable; callbacks receive `formValues.value`, and Generate receives the full map. Omitted unrelated controlled values preserve newer queued drafts; successful submission returns fresh Generate action capability IDs.

The self-contained contract harness compiles **byte-exact** common.rs, SDK, NativeSessions and catalog logic from the pinned beta. Its generated registration/installed-package lookup, Tauri event emitter, recording OS router and presentation primitives are **headless test stubs**. The catalog fixture appends this manifest plus an uncompiled negative entry solely within the disposable harness; no host registration is asserted. The preserved upstream native_tests.rs tests target other extensions and are not executed; this port's tests call the real NativeSessions with this adapter instead.

Resolve the json-ts license provenance gap in THIRD_PARTY_NOTICES.md before redistribution. Parent review/publication is separate from implementation; no push, merge or production deploy is implied here.
