// Validation/build-time only: extract exact Unicode/word tables from pinned code.
const fs=require('node:fs'), vm=require('node:vm'), path=require('node:path');
const root=path.resolve(__dirname,'..');
function context(file,append){ const c={module:{exports:{}},exports:{},require}; vm.runInNewContext(fs.readFileSync(file,'utf8')+'\n'+append,c); return c; }
const nq=context(root+'/fixtures/oracle/needsquotes/index.js','this.tables={es5:regexIdentifierNameES5.source,es6:regexIdentifierNameES6.source,reserved:regexES3ReservedWord.source};');
const ld=context(root+'/fixtures/oracle/json-ts/_.js','');
// _.js wraps its internals. Insert a capture at the beginning of its final closure.
let src=fs.readFileSync(root+'/fixtures/oracle/json-ts/_.js','utf8');
src=src.replace('lodash.startCase = startCase;', 'globalThis.tables = {words: reUnicodeWord.source, deburr: deburredLetters}; lodash.startCase = startCase;');
const c={module:{exports:{}},exports:{}};vm.runInNewContext(src,c);
function unitsRegex(s){return s.replace(/\\u([a-fA-F0-9]{4})/g,(_,n)=>{let cp=parseInt(n,16);if(cp>=0xd800&&cp<=0xdfff)cp+=0xf0000;return '\\x{'+cp.toString(16)+'}';}).replace(/\\b/g,'(?![A-Za-z0-9_])');}
const upper={};for(let cp=0;cp<=0x10ffff;cp++){if(cp>=0xd800&&cp<=0xdfff)continue;const x=String.fromCodePoint(cp),u=x.toUpperCase();if(x!==u)upper[cp]=u;}
const generated=JSON.stringify({es5:unitsRegex(nq.tables.es5),es6:unitsRegex(nq.tables.es6),reserved:nq.tables.reserved,words:unitsRegex(c.tables.words),deburr:c.tables.deburr,upper},null,2)+'\n';
if(process.argv.includes('--check'))require('node:assert/strict').equal(fs.readFileSync(root+'/src/tables.json','utf8'),generated);else fs.writeFileSync(root+'/src/tables.json',generated);
