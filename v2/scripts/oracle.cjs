// Executes the byte-exact own TSX and locked json-ts/TypeScript libraries.
// React and Raycast OS boundaries are recording stubs; conversion code is real.
const fs=require('node:fs'),path=require('node:path'),Module=require('node:module'),cp=require('node:child_process'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');process.env.NODE_PATH=root+'/fixtures/oracle';Module._initPaths();
const ts=require(root+'/fixtures/oracle/typescript');const {json2ts}=require(root+'/fixtures/oracle/json-ts');
const cases=[];const add=(input,rootName='rootName')=>cases.push({input,rootName});
for(const x of [{},[],[{}],null,true,false,1,0,1e300,'hello',{'0':{},'01':{},'2e2':{}},{a:1,b:'x',c:null,d:true},{a:{b:2},z:[{x:1},{y:2}]},[[{a:1}],{b:2}],{a:[[],[1]]}])add(JSON.stringify(x));
for(const key of ['', 'fooBar','FOOBar','XMLHttpRequest','foo_bar','x-y','class','default','true','$ok','_ok','0','010','2e2','1.23','+1','-0','Infinity',"a'b",'a\\b','a\n','Қазақ','日本語','café','ßeta','😀','👩‍💻','𝒜bc','e\u0301','\u200c','\u037f','\u180e','\u309b','\\u0061','𐐨ello','\u{fd800}','\u{10ffff}','a\u2028','𝖆bc','\\u{61}','0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF']) {
 for(const value of [1,{}, {a:1},[],[{}],[{x:1},{y:2}], [true,false,1,null,'x']])add(JSON.stringify({[key]:value}));
}
for(const rootName of ['rootName','x','foo-bar','', ' ', '1x','0','__proto__','class','ßeta','😀','日本語','Foo_Bar','a\nb'])for(const input of ['{}','[]','[{"x":1}]','1','null'])add(input,rootName);
for(const input of ['', ' ','abc','undefined','NaN','-1','+1','!true','~1','1+2','{}; trailing','// hi\n{a:1}', '{','[','{"a":','[1,','{a:1 b:2}', '{a}', '{"a":1,"a":false}', '{["a"]:1}', '{0x10:1,1e3:2,1e999:3,00:4}', '{a:undefined,b:NaN,c:`hi`,d:-3,e:!foo}', '[1,,2]', '[true,false,null,1,"s"]', '{a:1+2,b:(1),c:foo(),d:{e:2}}', '{"a\\n":1,"a\\\'b":2,"a\\\\b":3}'])add(input);
for(const vals of [[{a:1},{b:2}],[{a:1,b:2},{a:'x',c:true}],[{x:[]},{x:3}],[{x:3},{x:[]}],[{x:[1]},{x:['s']}],[{x:{a:1}},{x:{b:2}}],[{'x-y':{a:1}},{'x-y':{b:2}}],[{x:1},{x:'s'},{x:false},{x:[]}],[{a:1},{},{a:1}], [{a:1,a2:2},{b:1,b2:2}], [{x:[]},{x:[{}]},{x:['a']} ]])add(JSON.stringify({foo:vals}));
let seed=0x19842026;const rand=n=>{seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed%n;};
const keys=['a','b','items','fooBar','foo-bar','nested','null','x_y','日本語','éé','0'];
function value(depth){switch(rand(depth>3?6:9)){case 0:return null;case 1:return !!rand(2);case 2:return rand(100)-50;case 3:return ['x','', '😀'][rand(3)];case 4:return [];case 5:return {};case 6:return Array.from({length:rand(5)},()=>value(depth+1));default:{const o={};for(let i=0,n=rand(5);i<n;i++)o[keys[rand(keys.length)]]=value(depth+1);return o;}}}
for(let i=0;i<1200;i++)add(JSON.stringify(value(0)));
function oracle(c){if(c.input.length===0)return {ok:false,error:'Empty JSON'};if(c.rootName.length===0)return {ok:false,error:'Empty interface name'};try{return{ok:true,output:json2ts(c.input,{rootName:c.rootName,flow:false,prefix:''})}}catch{return{ok:false,error:'Invalid JSON'}}}
for(const sample of ['{"a":1,"b":[true,false,null,"hey"],"c":{"x-y":{}}}', '[1,{"x":2},[3]]', '{"escape\\u0061":"value"}'])for(let i=0;i<=sample.length;i++)add(sample.slice(0,i));
for(const key of ['a','x-y','null','日本語','',"a'b"]){
 const shapes=[1,'s',null,true,[],[1],['s'],{}, {x:1}];
 for(let a=0;a<shapes.length;a++)for(let b=0;b<shapes.length;b++)add(JSON.stringify({rows:[{[key]:shapes[a]},{[key]:shapes[b]},{}]}));
}
const fixtures=cases.map(c=>({...c,expected:oracle(c)}));
if(process.argv.includes('--write'))fs.writeFileSync(root+'/tests/oracle-cases.json',JSON.stringify(fixtures,null,2)+'\n');
else assert.deepEqual(JSON.parse(fs.readFileSync(root+'/tests/oracle-cases.json')),fixtures,'oracle snapshots drifted');
const code=ts.transpileModule(fs.readFileSync(root+'/fixtures/own/src/index.tsx','utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.React,target:ts.ScriptTarget.ES2017}}).outputText;
async function own(c,failCopy=false){const calls=[];const m={exports:{}};const api={Form:Object.assign('Form',{TextArea:'Form.TextArea',TextField:'Form.TextField'}),ActionPanel:'ActionPanel',Action:{SubmitForm:'Action.SubmitForm'},Toast:{Style:{Failure:'failure'}},showToast:async(...a)=>calls.push(['toast',...a]),showHUD:async(...a)=>calls.push(['hud',...a]),popToRoot:async()=>calls.push(['root']),Clipboard:{copy:async x=>{calls.push(['copy',x]);if(failCopy)throw Error('private');}}};
// String primitive cannot retain assigned properties; use callable marker for Form.
api.Form=Object.assign(function Form(){},{TextArea:'Form.TextArea',TextField:'Form.TextField'});
const React={createElement:(type,props,...children)=>({type,props:{...props,children}})};
const quiet={log(){}};
new Function('React','require','module','exports','console',code)(React,n=>n==='@raycast/api'?api:require(root+'/fixtures/oracle/'+n),m,m.exports,quiet);
const form=m.exports.default();assert.equal(form.props.children[1].props.defaultValue,'rootName');
await form.props.actions.props.children[0].props.onSubmit({jsonTextarea:c.input,rootName:c.rootName});return calls;
}
(async()=>{
for(const c of fixtures){const calls=await own(c);if(c.expected.ok)assert.deepEqual(calls,[['copy',c.expected.output],['hud','Copied to clipboard'],['root']]);else assert.deepEqual(calls,[['toast','failure',c.expected.error]]);}
assert.deepEqual(await own({input:'{}',rootName:'r'},true),[['copy','interface R {\n}\n'],['toast','failure','Invalid JSON']]);
const bin=process.env.JSON2TS_PROBE||root+'/target/debug/json2ts-oracle-probe';
const result=cp.spawnSync(bin,[],{input:cases.map(c=>JSON.stringify(c)).join('\n')+'\n',encoding:'utf8',maxBuffer:64*1024*1024});if(result.error)throw result.error;assert.equal(result.status,0,result.stderr);
const outputs=result.stdout.trim().split('\n').map(x=>JSON.parse(x));assert.equal(outputs.length,cases.length);let failures=[];
outputs.forEach((o,i)=>{const e=fixtures[i].expected;if(o.ok!==e.ok||(o.ok&&o.output!==e.output))failures.push({i,...cases[i],expected:e,actual:o});});
fs.mkdirSync(root+'/.validation',{recursive:true});fs.writeFileSync(root+'/.validation/oracle-result.json',JSON.stringify({cases:cases.length,failures},null,2)+'\n');
console.log(`Actual TSX + locked-library oracle: ${cases.length} cases, ${failures.length} differences`);if(failures.length){console.log(JSON.stringify(failures.slice(0,20),null,2));process.exitCode=1;}
})();
