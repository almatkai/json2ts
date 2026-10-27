"""Self-contained headless harness; never edits the beta checkout.
The pinned common.rs, SDK and NativeSessions are compiled byte-exact.
Tauri emission, registry and compiled runtime registration are explicit test stubs.
"""
from pathlib import Path
import shutil
root=Path(__file__).resolve().parents[1]
out=root/'.validation/harness'
out.mkdir(parents=True,exist_ok=True)
(out/'Cargo.toml').write_bytes((root/'harness/Cargo.toml.in').read_bytes())
(out/'src').mkdir(exist_ok=True)
(out/'src/lib.rs').write_bytes((root/'harness/lib.rs.in').read_text().replace("__ROOT__",str(root)).encode())
stub=out/'tauri-stub/src';stub.mkdir(parents=True,exist_ok=True)
(out/'tauri-stub/Cargo.toml').write_text('[package]\nname="tauri"\nversion="0.0.0"\nedition="2021"\n[dependencies]\nserde_json="=1.0.145"\n')
(stub/'lib.rs').write_text('''//! HEADLESS TAURI STUB: no app, emission, window, clipboard, browser or OS calls.
#[derive(Clone)] pub struct AppHandle;
pub trait Emitter {fn emit(&self, channel:&str, payload:serde_json::Value)->Result<(),String>;}
impl Emitter for AppHandle {fn emit(&self,_:&str,_:serde_json::Value)->Result<(),String>{Err("Headless event emission unavailable".into())}}
''')
for src in (root/'harness').glob('*.rs'):
 shutil.copyfile(src,out/'src'/src.name)
lock=root/'harness/Cargo.lock.fixture'
if lock.exists():shutil.copyfile(lock,out/'Cargo.lock')
print(out)

native=out/'native-fixture';(native/'src').mkdir(parents=True,exist_ok=True)
manifest=(root/'harness/Cargo.toml.in').read_text().replace('name = "json2ts-beta-contract"','name = "json2ts-native-fixture"').replace('path = "../.."','path = "../../.."').replace('path = "tauri-stub"','path = "../tauri-stub"')
manifest='\n'.join(x for x in manifest.splitlines() if not x.startswith('json2ts-native-fixture ='))+'\n'
(native/'Cargo.toml').write_text(manifest)
(native/'src/lib.rs').write_text((root/'harness/native-lib.rs.in').read_text().replace('__ROOT__',str(root)).replace('__CATALOG__',str(native/'src/catalog')))

import json
cat=native/'src/catalog';cat.mkdir(exist_ok=True)
shutil.copyfile(root/'fixtures/beta/src-tauri/src/native_core/extensions/v2_catalog.rs',cat/'v2_catalog.rs')
items=json.loads((root/'fixtures/beta/src-tauri/src/native_core/extensions/v2_catalog.json').read_text())
manifest=json.loads((root/'manifest.json').read_text());items.append(manifest)
missing=dict(manifest);missing['id']='com.tezbar.native.json2ts-uncompiled';missing['slug']='json2ts-uncompiled';items.append(missing)
(cat/'v2_catalog.json').write_text(json.dumps(items))
