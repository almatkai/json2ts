# Validation

Validated in the owned Json2TS clone, from own source `8186f2229ec907bb3f705d1a641fd7159f0339f9`, using Node 26.7.0, stable Rust and Rust 1.85.0. All source/lock/license/icon files outside v2 remain byte-exact except the permitted root README discovery link. Nothing is staged, committed or pushed.

| Gate | Result / scope |
|---|---|
| Core tests | 4 tests; includes exact expected bytes for 2,187 actual-source oracle cases, duplicate/legacy quirks, malformed recovery, explicit limits and surrogate boundary |
| Actual TSX and locked-library oracle | 2,187 cases, zero differences in this corpus; actual own handleSubmit is transpiled/executed with recording Raycast/React boundaries, actual json-ts 1.6.4 + TypeScript 2.9.2 + Immutable 3.8.3 + needsquotes 1.0.0 + embedded Lodash 4.17.4; no handwritten JS converter |
| Headless Rust beta contract | 20 tests: 11 adapter tests, 5 actual NativeSessions/catalog tests, 2 unchanged SDK tests and 2 unchanged common tests; 13 malformed/failed response shapes exercised independently at both copy and hide, plus positive acknowledgements, output binding, queued drafts, disposal, cancellation and deliberate retries |
| Actual beta renderer | 13 tests: 8 unchanged form reconciliation tests plus 5 compiled-wire Form parser/Surface action-routing tests; only presentation/hook/window boundaries stubbed |
| Stable and MSRV | cargo fmt and strict all-target Clippy; core + adapter contract tests on stable and Rust 1.85.0 |
| Windows GNU | x86_64-pc-windows-gnu check + strict all-target Clippy for core and adapter harness; compilation only, no Windows execution |
| Schema and provenance | full pinned Draft 2020-12 schema validator with 24 negative cases; 100 retained-input/icon/table hash entries; exact icon 29,082 bytes and expected SHA256; manifest semantics/credits and license evidence checked; every provenance path audited for Git visibility |
| Cargo package | package verification and self-contained extracted offline core/adapter tests; harness Cargo.toml.in + lock fixture materialize omitted nested crates without needing sibling checkouts |
| Disposable V1 | actual npm test smoke (1 test) + tsc --noEmit; all original source/lock/license/icon bytes verified afterwards. V1 script imports tsx but does not declare it; the disposable node_modules receives only a symlink to separately pinned validation tsx 4.22.4, without changing manifests/locks. |

These results do not establish full TypeScript-compiler compatibility. Corpus includes scalars, nested objects, duplicate properties, null/empty values, heterogeneous/nested arrays, merging and optionality, reference collisions, numeric and Unicode identifiers, quoting/escaping, root naming, malformed and truncated inputs, missing commas/colons, ignored expressions, and deterministic generated cases. Arbitrary TypeScript programs outside the literal/recovery contract are not exhaustively verified. Unpaired UTF-16 in emitted names errors explicitly. Resource limits and beta navigation/HUD adaptation are documented in README.md.

Known integration failure against the unchanged beta: platform `window:hide` returns null after success and the core router forwards it. This is intentionally rejected by the strict contract. `clipboard:write` returns an object acknowledgement and can be confirmed. Host integration must supply an explicit acknowledged hide result; this port does not reinterpret null or edit that host code.

The real common.rs, v2_sdk.rs, native.rs (NativeSessions) and v2_catalog.rs are compiled byte-exact. The generated catalog adds the manifest and an unavailable negative fixture. Registration and installed-package lookup are explicit stubs. The Tauri AppHandle/emitter stub never emits; routers record and return supplied acknowledgements or pending futures. No genuine clipboard, selected text, paste, browser, window, focus or OS effect runs. NativeSessions' upstream Perplexity-specific native_tests.rs is retained for inspection but not executed as this extension's test suite.

Host registration/rebuild and actual macOS/Windows UI, clipboard, focus, hide, and navigation QA are **not-run**. True pop-to-root and genuine HUD/toast remain unavailable beta APIs; no equivalent is falsely reported. Both manifest verification entries remain not-run. The json-ts license declaration gap is explicitly recorded in THIRD_PARTY_NOTICES.md and PROVENANCE.json.

XERJ was attempted first with canonical root prefix `ax-d5c6418d2f28` and isolated workspace state. Autoindex received RSS memory-circuit HTTP 429 rejections, then search found no indexed match. Only this task's index process was stopped; no global restart was attempted. Index coverage is incomplete. Exact/exhaustive checks use current files, git show pins, rg, hashes and executed tests.

Reproduce from this directory (network only for initially fetching validation dependencies):

```sh
python3 scripts/materialize.py
python3 scripts/renderer.py
(cd .validation/renderer && npm ci --ignore-scripts --no-audit --no-fund)
python3 -m venv .validation/schema
.validation/schema/bin/python -m pip install -r scripts/schema-requirements.txt
cargo fmt -- --check
rustfmt --edition 2021 --check adapter/v2_json2ts.rs harness/adapter_tests.rs harness/session_tests.rs
rustfmt --edition 2021 --config skip_children=true --check harness/lib.rs.in harness/native-lib.rs.in
cargo test --locked
cargo +1.85.0 test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo +1.85.0 clippy --all-targets --locked -- -D warnings
JSON2TS_UI_FIXTURE="$PWD/.validation/ui-views.json" cargo test --manifest-path .validation/harness/Cargo.toml --locked
cargo +1.85.0 test --manifest-path .validation/harness/Cargo.toml --locked
cargo clippy --manifest-path .validation/harness/Cargo.toml --all-targets --locked -- -D warnings
cargo +1.85.0 clippy --manifest-path .validation/harness/Cargo.toml --all-targets --locked -- -D warnings
cargo build --locked --bin json2ts-oracle-probe
node scripts/oracle.cjs
node scripts/tables.cjs --check
.validation/schema/bin/python scripts/verify.py
(cd .validation/renderer && npm test)
python3 scripts/v1-check.py
python3 scripts/windows-check.py
python3 scripts/package-check.py
```

`windows-check.py` checks both toolchains and core/harness after the target is installed. `package-check.py` verifies Cargo's archive, extracts inside the owned ignored validation directory, materializes the harness from templates and runs offline core/harness tests on both toolchains. Offline refers to Cargo validation with an already populated crate cache, not a fully vendored Rust dependency tree. The extracted package needs neither Git nor the beta checkout. Python schema tool requirements and renderer tools are separately locked. See `.validation/` for local disposable build artifacts; it is intentionally ignored and not a deliverable.
