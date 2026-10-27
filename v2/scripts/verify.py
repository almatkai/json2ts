"""Verify retained inputs and full pinned Draft 2020-12 manifest schema.
--record writes provenance after byte comparisons to source pins and npm-ci files.
Normal verification is self-contained and does not need the host checkout or network.
"""
from pathlib import Path
import hashlib,json,subprocess,sys,copy
R=Path(__file__).resolve().parents[1]
OWN='8186f2229ec907bb3f705d1a641fd7159f0339f9'
BETA='fd300e276356587726566026f072563eb9c34cb1'
ORIGINAL='6734d85992290066fd31a7f5bee3cca3d3545a1a'
ICON='0b083ac11c306e7704f359a6d8b15a95a77f6a0b9f2ea76c6d628f8a3d43cd7c'
def sha(b):return hashlib.sha256(b).hexdigest()
lock=json.loads((R/'fixtures/own/package-lock.json').read_text())
versions={'json-ts':'1.6.4','immutable':'3.8.3','needsquotes':'1.0.0','typescript':'2.9.2'}
if '--record' in sys.argv:
 files=[]
 for p in sorted((R/'fixtures').rglob('*')):
  if not p.is_file():continue
  rel=p.relative_to(R).as_posix();parts=p.relative_to(R/'fixtures').parts;kind=parts[0];suffix='/'.join(parts[1:]);data=p.read_bytes()
  if kind=='own':
   assert data==subprocess.check_output(['git','-C',str(R.parent),'show',OWN+':'+suffix]),rel
   source={'repository':'https://github.com/almatkai/json2ts','commit':OWN,'path':suffix}
  elif kind=='beta':
   assert data==subprocess.check_output(['git','-C','/Users/almat/Desktop/code/Tezbar APP/Tezbar','show',BETA+':'+suffix]),rel
   source={'repository':'https://github.com/almatkai/Tezbar','commit':BETA,'path':suffix}
  elif kind=='oracle':
   name=parts[1];suffix='/'.join(parts[2:]);module_path='node_modules/'+name if name!='typescript' else 'node_modules/json-ts/node_modules/typescript'
   actual=R/'.validation/v1'/module_path/suffix;assert actual.read_bytes()==data,rel
   meta=lock['packages'][module_path];assert meta['version']==versions[name]
   source={'npm':name,'version':meta['version'],'resolved':meta['resolved'],'integrity':meta['integrity'],'path':suffix}
  elif kind=='original':source={'repository':'https://github.com/raycast/extensions','commit':ORIGINAL,'path':'extensions/json2ts/'+suffix}
  elif suffix=='beta-LICENSE':
   assert data==(R/'fixtures/beta/LICENSE').read_bytes()
   source={'repository':'https://github.com/almatkai/Tezbar','commit':BETA,'path':'LICENSE'}
  elif suffix=='raycast-LICENSE':source={'repository':'https://github.com/raycast/extensions','commit':ORIGINAL,'path':'LICENSE'}
  else:raise AssertionError(rel)
  files.append({'path':rel,'bytes':len(data),'sha256':sha(data),'source':source})
 for name in ['assets/command-icon.png','src/tables.json']:
  data=(R/name).read_bytes();files.append({'path':name,'bytes':len(data),'sha256':sha(data),'source':{'kind':'exact original icon copy' if name.startswith('assets') else 'scripts/tables.cjs derived from pinned needsquotes and Lodash; uppercase from Node 26.7.0'}})
 provenance={'ownSourceSHA':OWN,'originalSHA':ORIGINAL,'betaSHA':BETA,'aggregateSPDX':'MIT AND ISC AND Apache-2.0 AND LicenseRef-json-ts-Unspecified','resolvedConversionVersions':versions,'embeddedLodash':'4.17.4','jsonTsGitHead':'eb6d3acdae92f03a31f9f0494ac6fa464bdb1891','files':files}
 (R/'PROVENANCE.json').write_text(json.dumps(provenance,indent=2)+'\n')
p=json.loads((R/'PROVENANCE.json').read_text());assert p['ownSourceSHA']==OWN and p['betaSHA']==BETA
paths={entry['path'] for entry in p['files']}
assert {f.relative_to(R).as_posix() for f in (R/'fixtures').rglob('*') if f.is_file()} <= paths
for entry in p['files']:
 b=(R/entry['path']).read_bytes();assert len(b)==entry['bytes'] and sha(b)==entry['sha256'],entry['path']
for path in ['assets/command-icon.png','fixtures/original/assets/command-icon.png','fixtures/own/assets/command-icon.png']:
 b=(R/path).read_bytes();assert len(b)==29082 and sha(b)==ICON
for name,v in versions.items():assert json.loads((R/'fixtures/oracle'/name/'package.json').read_text())['version']==v
assert 'license' not in json.loads((R/'fixtures/oracle/json-ts/package.json').read_text())
assert p['aggregateSPDX']=='MIT AND ISC AND Apache-2.0 AND LicenseRef-json-ts-Unspecified'
# A full standards-compliant validator executes the pinned schema, not a handwritten subset.
from jsonschema import Draft202012Validator
schema=json.loads((R/'fixtures/beta/schemas/extension-v2.schema.json').read_text());Draft202012Validator.check_schema(schema);validator=Draft202012Validator(schema)
m=json.loads((R/'manifest.json').read_text());validator.validate(m)
mutations=[('schemaVersion',1),('id','raycast.json2ts'),('runtime','javascript'),('icon','assets/command-icon.png'),('author','forbidden'),('slug','Json2TS'),('replaces',[]),('replaces',['raycast.json2ts','raycast.json2ts']),('commands',[]),('platforms',['macos']),('verification',[{'platform':'macOS','state':'not-run'}])]
neg=[]
for key,val in mutations:
 n=copy.deepcopy(m);n[key]=val;neg.append(n)
for key in schema['required']:
 n=copy.deepcopy(m);del n[key];neg.append(n)
for key,val in [('name',''),('platforms',[]),('runtime','other')]:
 n=copy.deepcopy(m);n['commands'][0][key]=val;neg.append(n)
for n in neg:assert not validator.is_valid(n),n
assert set(m)=={'schemaVersion','id','slug','name','version','description','icon','runtime','replaces','platforms','commands','verification'}
assert m['id']=='com.tezbar.native.json2ts' and m['replaces']==['raycast.json2ts']
assert m['commands']==[{'name':'index','title':'Json2ts','description':'Generate TypeScript interfaces from JSON','mode':'view','platforms':['macOS','Windows'],'runtime':'rust'}]
assert m['verification'][0]['state']==m['verification'][1]['state']=='not-run'
# Git visibility auditing never stages files and gracefully skips Git for extracted Cargo packages.
if (R.parent/'.git').exists():
 hidden=[]
 for rel in paths:
  r=subprocess.run(['git','-C',str(R.parent),'check-ignore','--no-index','--quiet','v2/'+rel])
  if r.returncode==0:hidden.append(rel)
 assert not hidden,hidden
 changed=subprocess.check_output(['git','-C',str(R.parent),'diff','--name-only',OWN]).decode().splitlines()
 assert all(x=='README.md' or x.startswith('v2/') for x in changed),changed
 for rel in subprocess.check_output(['git','-C',str(R.parent),'ls-tree','-r','--name-only',OWN]).decode().splitlines():
  if rel!='README.md':assert (R.parent/rel).read_bytes()==subprocess.check_output(['git','-C',str(R.parent),'show',OWN+':'+rel]),rel
print(f"Verified {len(p['files'])} provenance entries, exact icon, SPDX evidence, full schema + {len(neg)} negative cases; V1 bytes and Git visibility audited when in checkout")
