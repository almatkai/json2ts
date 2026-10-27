from pathlib import Path
import subprocess
r=Path(__file__).resolve().parents[1]
for toolchain in ['stable','1.85.0']:
 for manifest in ['Cargo.toml','.validation/harness/Cargo.toml']:
  for command in ['check','clippy']:
   args=['cargo','+'+toolchain,command,'--manifest-path',str(r/manifest),'--all-targets','--target','x86_64-pc-windows-gnu','--locked']
   if command=='clippy':args+=['--','-D','warnings']
   subprocess.run(args,cwd=r,check=True)
print('Windows GNU core + headless adapter all-target check/Clippy passed on stable and 1.85; Windows execution not run')
