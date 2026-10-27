"""Disposable V1 checks; never install modules or modify lockfiles in the root.
Requires renderer validation tools installed (pinned tsx supplies missing V1 dev tool).
"""
from pathlib import Path
import shutil,subprocess,hashlib
r=Path(__file__).resolve().parents[1];out=r/'.validation/v1';out.mkdir(exist_ok=True)
for p in (r/'fixtures/own').rglob('*'):
 if p.is_file():
  dst=out/p.relative_to(r/'fixtures/own');dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dst)
lock=(out/'package-lock.json').read_bytes()
if not (out/'node_modules').exists():subprocess.run(['npm','ci','--ignore-scripts','--no-audit','--no-fund'],cwd=out,check=True)
# The source test script imports tsx, but its manifest/lock omit tsx. Provide a
# symlink to the separately locked validation tool, changing no source/lock bytes.
loader=r/'.validation/renderer/node_modules/tsx';assert loader.exists(),'npm ci in materialized renderer first'
link=out/'node_modules/tsx'
if not link.exists():link.symlink_to(loader,target_is_directory=True)
subprocess.run(['npm','test'],cwd=out,check=True)
subprocess.run(['npm','run','typecheck'],cwd=out,check=True)
assert (out/'package-lock.json').read_bytes()==lock
for p in (r/'fixtures/own').rglob('*'):
 if p.is_file():assert p.read_bytes()==(out/p.relative_to(r/'fixtures/own')).read_bytes(),p
print('Disposable V1 smoke + typecheck passed; every source/lock/license/icon byte retained')
