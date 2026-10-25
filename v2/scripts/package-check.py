from pathlib import Path
import subprocess,tarfile,json,os
r=Path(__file__).resolve().parents[1]
subprocess.run(['cargo','package','--allow-dirty','--locked'],cwd=r,check=True)
archive=r/'target/package/tezbar-json2ts-0.1.0.crate'
out=r/'.validation/extracted';out.mkdir(exist_ok=True)
with tarfile.open(archive) as tar:
 for m in tar.getmembers():assert not m.name.startswith('/') and '..' not in Path(m.name).parts
 paths={m.name.split('/',1)[1] for m in tar.getmembers() if '/' in m.name}
 required={f['path'] for f in json.loads((r/'PROVENANCE.json').read_text())['files']}
 required|={'PROVENANCE.json','LICENSE','harness/Cargo.toml.in','harness/Cargo.lock.fixture','harness/native-lib.rs.in','adapter/v2_json2ts.rs','tests/oracle-cases.json'}
 assert required<=paths,sorted(required-paths)
 assert not any('/target/' in p or p.startswith('.validation/') for p in paths)
 tar.extractall(out)
root=out/'tezbar-json2ts-0.1.0'
subprocess.run(['python3','scripts/materialize.py'],cwd=root,check=True)
for tc in ['stable','1.85.0']:
 for manifest in ['Cargo.toml','.validation/harness/Cargo.toml']:
  subprocess.run(['cargo','+'+tc,'test','--manifest-path',manifest,'--offline','--locked'],cwd=root,check=True)
# This validation uses the external tool env only; all validated files are extracted.
python=r/'.validation/schema/bin/python'
subprocess.run([str(python),'scripts/verify.py'],cwd=root,check=True)
subprocess.run(['cargo','build','--offline','--locked','--bin','json2ts-oracle-probe'],cwd=root,check=True)
subprocess.run(['node','scripts/oracle.cjs'],cwd=root,check=True)
print(f'Cargo archive verified: {len(paths)} paths; self-contained extracted offline tests and actual-source oracle passed')
