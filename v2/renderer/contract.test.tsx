import React from 'react'
import {readFileSync} from 'node:fs'
import {describe,it,expect,vi} from 'vitest'
import TestRenderer,{act} from 'react-test-renderer'
import {FormRuntime} from './src/renderer/src/raycast-api/form-runtime'
// HEADLESS window event boundary; no browser/OS effects.
vi.stubGlobal('window',{addEventListener:vi.fn(),removeEventListener:vi.fn(),setTimeout,clearTimeout})
const wire=JSON.parse(readFileSync('../ui-views.json','utf8'))
function props(root=wire.initial.root){return {root,title:'Json2ts',onBack:vi.fn(),onSubmitForm:vi.fn(),onChangeField:vi.fn(),onInvokeAction:vi.fn(),onOpenActions:vi.fn()}}
describe('actual compiled adapter through unchanged beta form parser and reconciliation',()=>{
 it('parses fields, exact labels, placeholder, default root and full submit map',()=>{
  const p=props();let t:any;act(()=>{t=TestRenderer.create(<FormRuntime {...p}/>)});
  expect(t.root.findByType('textarea').props.placeholder).toBe('Enter JSON here');
  expect(t.root.findByType('input').props.value).toBe('rootName');
  expect(t.root.findAllByType('label').map(x=>x.children.join(''))).toEqual(['JSON','Interface Name']);
  act(()=>t.root.findByType('textarea').props.onChange({target:{value:'{"x":1}'}}));
  expect(p.onChangeField).toHaveBeenCalledWith(wire.initial.root.children[0].props.actionId,'{"x":1}');
  act(()=>t.root.findByType('form').props.onSubmit({preventDefault(){}}));
  expect(p.onSubmitForm).toHaveBeenCalledWith({jsonTextarea:'{"x":1}',rootName:'rootName'});
 });
 it('retains newer queued JSON draft when root callback omits JSON controlled value',()=>{
  const p=props();let t:any;act(()=>{t=TestRenderer.create(<FormRuntime {...p}/>)});
  act(()=>t.root.findByType('textarea').props.onChange({target:{value:'newer unsent draft'}}));
  act(()=>{t.update(<FormRuntime {...p} root={wire.changed.root}/>)});
  expect(t.root.findByType('textarea').props.value).toBe('newer unsent draft');
  expect(t.root.findByType('input').props.value).toBe('ChangedRoot');
  expect(wire.initial.root.children.map(n=>n.props.id)).toEqual(wire.changed.root.children.map(n=>n.props.id));
 });
 it('routes Back without changing a mounted draft and exposes field callback IDs',()=>{
  const p=props();let t:any;act(()=>{t=TestRenderer.create(<FormRuntime {...p}/>)});
  act(()=>t.root.findByType('input').props.onChange({target:{value:'draft'}}));
  act(()=>t.root.findByProps({'aria-label':'Back'}).props.onClick());
  expect(p.onBack).toHaveBeenCalledOnce();expect(t.root.findByType('input').props.value).toBe('draft');
  expect(p.onChangeField).toHaveBeenCalledWith(wire.initial.root.children[1].props.actionId,'draft');
 });
})
import {ExtensionRuntimeSurface} from './src/renderer/src/raycast-api/index'
describe('unchanged beta Surface action routing',()=>{
 it('wraps field callbacks in formValues.value and submits the full map to Generate',()=>{
  const invoke=vi.fn();let t:any;act(()=>{t=TestRenderer.create(<ExtensionRuntimeSurface sessionId="fixture" title="Json2ts" extensionId="com.tezbar.native.json2ts" commandName="index" root={wire.initial.root} actions={wire.initial.actions} onBack={vi.fn()} onSearchTextChanged={vi.fn()} onLoadMore={vi.fn()} onInvokeAction={invoke}/>)});
  act(()=>t.root.findByType('textarea').props.onChange({target:{value:'{}'}}));
  expect(invoke).toHaveBeenCalledWith(wire.initial.root.children[0].props.actionId,{value:'{}'});
  act(()=>t.root.findByType('form').props.onSubmit({preventDefault(){}}));
  expect(invoke).toHaveBeenLastCalledWith(wire.initial.actions[0].id,{jsonTextarea:'{}',rootName:'rootName'});
 });
 it('routes explicit retry from the action panel without event-supplied copy text',()=>{
  const invoke=vi.fn(),back=vi.fn();let t:any;act(()=>{t=TestRenderer.create(<ExtensionRuntimeSurface sessionId="fixture" title="Json2ts" extensionId="com.tezbar.native.json2ts" commandName="index" root={wire.failure.root} actions={wire.failure.actions} onBack={back} onSearchTextChanged={vi.fn()} onLoadMore={vi.fn()} onInvokeAction={invoke}/>)});
  act(()=>t.root.findByProps({'aria-label':'Open actions'}).props.onClick());
  act(()=>t.root.findByProps({'data-action':wire.failure.actions[0].id}).props.onClick());
  expect(invoke).toHaveBeenCalledWith(wire.failure.actions[0].id);
  act(()=>t.root.findByProps({'aria-label':'Back'}).props.onClick());expect(back).toHaveBeenCalledOnce();
 });
})
