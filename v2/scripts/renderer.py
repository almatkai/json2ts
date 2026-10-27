from pathlib import Path
import shutil
r=Path(__file__).resolve().parents[1];out=r/'.validation/renderer';out.mkdir(exist_ok=True)
for p in (r/'renderer').iterdir():
 if p.is_file():shutil.copyfile(p,out/p.name)
for name in ['form-runtime.tsx','form-runtime.test.ts','index.tsx','action-runtime.tsx','action-runtime-registry.tsx']:
 src=r/'fixtures/beta/src/renderer/src/raycast-api'/name;dst=out/'src/renderer/src/raycast-api'/name;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst)
# Mock boundaries by matching the actual dependency paths, without editing source.
for path in ['src/renderer/ui/primitives.tsx','src/renderer/src/raycast-api/detail-runtime.tsx','src/renderer/src/raycast-api/grid-runtime.tsx','src/renderer/src/raycast-api/list-runtime.tsx']:
 dst=out/path;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(r/'renderer/stubs.tsx',dst)
print(out)

for source,target in [('hooks-stub.tsx','hooks.ts'),('overlay-stub.tsx','action-runtime-overlay.tsx')]:
 shutil.copyfile(r/'renderer'/source,out/'src/renderer/src/raycast-api'/target)
