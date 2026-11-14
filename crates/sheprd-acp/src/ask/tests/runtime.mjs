// ABOUTME: Exercises the actual helper executable with a private fake Pi process.
// ABOUTME: Covers attach input, durable receipts, cancellation, restart, and exact child cleanup.
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import net from 'node:net';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';
import { test } from 'node:test';

const binary=process.env.SPACEPORT_ASK_TEST_HELPER;
assert.ok(binary,'Set SPACEPORT_ASK_TEST_HELPER to the built helper executable.');
const fakePi=await readFile(new URL('../../../../test-support/ask-pi.mjs',import.meta.url),'utf8');

async function until(read, predicate, child) {
  for(let i=0;i<300;i++) {
    if(child?.exitCode !== null && child?.exitCode !== undefined) throw new Error('Helper exited before checkpoint: '+child.exitCode);
    try {const value=await read(); if(predicate(value)) return value;} catch(error) {if(error.code!=='ENOENT' && !(error instanceof SyntaxError)) throw error;}
    await delay(20);
  }
  throw new Error('Timed out waiting for fixture checkpoint.');
}

async function fixture(t,ready=true,emptySession=false) {
 const root=await mkdtemp('/tmp/spask-');
 const bin=join(root,'bin');await mkdir(bin);
 await writeFile(join(bin,'pi'),fakePi,{mode:0o700});
 const state=join(root,'conversation');
 const endpoint=randomUUID().replaceAll('-','');
 const env={PATH:bin+':'+process.env.PATH,HOME:root,XDG_STATE_HOME:join(root,'state'),SPACEPORT_FIXTURE_READY:ready?'1':'0',SPACEPORT_FIXTURE_EMPTY_SESSION:emptySession?'1':'0'};
 const children=[];
 const launch=()=>{const child=spawn(binary,['ask','--state-dir',state,'--endpoint',endpoint,'--broker',join(root,'control.sock'),'--conversation',conversation],{env,stdio:['ignore','pipe','pipe']});children.push(child);return child;};
 const conversation=randomUUID();
 const checkpoint=async()=>JSON.parse(await readFile(join(state,'checkpoint.json'),'utf8'));
 t.after(async()=>{
  try {await writeFile(join(state,'fixture-stop'),'stop');}catch(error){if(error.code!=='ENOENT')throw error;}
  for(const child of children){if(child.exitCode===null && child.signalCode===null){child.kill('SIGKILL');await new Promise(resolve=>child.once('exit',resolve));}}
  let pid;
  try {pid=Number(await readFile(join(state,'pid'),'utf8'));} catch(error) {if(error.code!=='ENOENT') throw error;}
  if(pid) await until(async()=>{try{process.kill(pid,0);return false;}catch(error){if(error.code==='ESRCH')return true;throw error;}},gone=>gone);
  await rm(root,{recursive:true,force:true});
 });
 return {root,state,endpoint,checkpoint,launch,env};
}

async function attach(root,endpoint) {
 const socket=net.createConnection(join(root,'state','herdr','acp',endpoint+'.sock'));
 await new Promise((resolve,reject)=>{socket.once('connect',resolve);socket.once('error',reject);});
 const send=value=>{const text=Buffer.from(JSON.stringify(value));const prefix=Buffer.alloc(4);prefix.writeUInt32LE(text.length);socket.write(Buffer.concat([prefix,text]));};
 const messages=[];let incoming=Buffer.alloc(0);
 socket.on('data',chunk=>{
  incoming=Buffer.concat([incoming,chunk]);
  while(incoming.length>=4 && incoming.length>=4+incoming.readUInt32LE(0)) {
   const size=incoming.readUInt32LE(0);messages.push(JSON.parse(incoming.subarray(4,4+size)));incoming=incoming.subarray(4+size);
  }
 });

 send({type:'hello',client:'gui',revision:1,since:null,version:1});
 return {socket,send,messages};
}

async function close(child,state) {
 await writeFile(join(state,'quit'),'quit');
 await Promise.race([new Promise(resolve=>child.once('exit',resolve)),delay(5000,undefined,{ref:false}).then(()=>{throw new Error('Helper did not quit.');})]);
 assert.equal(child.exitCode,0);
 const pid=Number(await readFile(join(state,'pid'),'utf8'));
 assert.throws(()=>process.kill(pid,0),error=>error.code==='ESRCH');
}

test('helper checkpoints literal questions and rejects mutation controls',async t=>{
 const f=await fixture(t); const child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 const {socket,send}=await attach(f.root,f.endpoint); t.after(()=>socket.destroy());
 send({type:'command',id:randomUUID(),command:{command:'set_auto_accept',on:true}});
 send({type:'command',id:randomUUID(),command:{command:'prompt',text:'/bash literal question',attachments:[]}});
 const state=await until(f.checkpoint,state=>state.outcome==='completed',child);
 assert.equal(state.request.phase,'completed');
 assert.equal(state.transcript.auto_accept,false);
 assert.ok(state.transcript.entries.some(entry=>entry.kind?.text==='/bash literal question' || entry.text==='/bash literal question'));
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 const prompt=commands.find(command=>command.type==='prompt');
 assert.ok(prompt.message.startsWith('Spaceport question'));
 assert.ok(!commands.some(command=>command.type==='set_model'));
 await close(child,f.state);
});

test('clean quit checkpoints interruption and restart sends one explicit continuation',async t=>{
 const f=await fixture(t);let child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 let {socket,send}=await attach(f.root,f.endpoint);t.after(()=>socket.destroy());
 send({type:'command',id:randomUUID(),command:{command:'prompt',text:'hold this question',attachments:[]}});
 await until(f.checkpoint,state=>state.request?.phase==='accepted',child);
 await close(child,f.state);socket.destroy();
 const interrupted=await f.checkpoint();assert.equal(interrupted.outcome,'interrupted');
 child=f.launch();
 await until(f.checkpoint,state=>state.helper_run_id!==interrupted.helper_run_id && state.request?.phase==='accepted',child);
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 const prompts=commands.filter(command=>command.type==='prompt');assert.equal(prompts.length,2);
 assert.ok(prompts[1].message.includes('Continue the unfinished'));
 await close(child,f.state);
});

test('helper refuses a missing owned-extension readiness receipt before prompt acceptance',async t=>{
 const f=await fixture(t,false);const child=f.launch();
 const state=await until(f.checkpoint,state=>state.transcript.status.state==='idle'||child.exitCode!==null);
 assert.notEqual(state.transcript.status.state,'idle');
 assert.notEqual(child.exitCode,0);
});

test('Stop clears queues, prevents continuation, and refuses attachment or mutation input',async t=>{
 const f=await fixture(t);let child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 let {socket,send}=await attach(f.root,f.endpoint);t.after(()=>socket.destroy());
 send({type:'command',id:randomUUID(),command:{command:'prompt',text:'hold question',attachments:[]}});
 await until(f.checkpoint,state=>state.request?.phase==='accepted',child);
 for(const command of [
  {command:'prompt',text:'queued question',attachments:[]},
  {command:'prompt',text:'file question',attachments:[{path:'/never-read'}]},
  {command:'set_auto_accept',on:true},
  {command:'set_option',id:'model',value:{type:'choice',id:'other'}},
  {command:'answer',option:'allow',permission:1},
 ]) send({type:'command',id:randomUUID(),command});
 send({type:'command',id:randomUUID(),command:{command:'cancel'}});
 await until(f.checkpoint,state=>state.outcome==='stopped'&&state.transcript.status.state==='idle',child);
 await close(child,f.state);socket.destroy();
 const stopped=await f.checkpoint(); child=f.launch();
 await until(f.checkpoint,state=>state.helper_run_id!==stopped.helper_run_id&&state.transcript.status.state==='idle',child);
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 assert.equal(commands.filter(command=>command.type==='prompt').length,1);
 assert.ok(commands.findIndex(command=>command.type==='clear_queue')<commands.findIndex(command=>command.type==='abort'));
 assert.equal((await f.checkpoint()).outcome,'stopped');
 await close(child,f.state);
});

test('uncertain checkpoint refuses agent startup and never resends',async t=>{
 const f=await fixture(t);let child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 await close(child,f.state);
 const state=await f.checkpoint();
 state.request={id:randomUUID(),text:'unknown prior question',phase:'accepted'};
 state.outcome='answering';
 await writeFile(join(f.state,'checkpoint.json'),JSON.stringify(state));
 const commands=await readFile(join(f.state,'commands.jsonl'),'utf8');
 child=f.launch();
 await until(async()=>child.exitCode,code=>code!==null);
 assert.equal(child.exitCode,1);
 const after=await f.checkpoint();
 assert.equal(after.outcome,'unknown');assert.equal(after.request.phase,'unknown');
 assert.equal(after.helper_run_id,state.helper_run_id);
 assert.equal(await readFile(join(f.state,'commands.jsonl'),'utf8'),commands);
});

test('durable Stop before immediate quit prevents continuation even without Cancel delivery',async t=>{
 const f=await fixture(t);let child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 const {socket,send}=await attach(f.root,f.endpoint);t.after(()=>socket.destroy());
 send({type:'command',id:randomUUID(),command:{command:'prompt',text:'hold durable stop',attachments:[]}});
 await until(f.checkpoint,state=>state.request?.phase==='accepted',child);
 await writeFile(join(f.state,'stop'),'stop');
 await close(child,f.state);socket.destroy();
 const stopped=await f.checkpoint();
 assert.equal(stopped.outcome,'stopped');assert.equal(stopped.request.phase,'stopped');
 child=f.launch();
 await until(f.checkpoint,state=>state.helper_run_id!==stopped.helper_run_id&&state.transcript.status.state==='idle',child);
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 assert.equal(commands.filter(command=>command.type==='prompt').length,1);
 await close(child,f.state);
});

test('Stop before delayed acceptance cancels again after the actual start',async t=>{
 const f=await fixture(t);const child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 const {socket,send}=await attach(f.root,f.endpoint);t.after(()=>socket.destroy());
 send({type:'command',id:randomUUID(),command:{command:'prompt',text:'gate acceptance hold question',attachments:[]}});
 await until(()=>readFile(join(f.state,'acceptance-held'),'utf8'),value=>value==='held',child);
 await writeFile(join(f.state,'stop'),'stop');
 const stopped=await until(f.checkpoint,state=>state.outcome==='stopped' && state.transcript.status.state==='idle',child);
 assert.equal(stopped.request.phase,'stopped');
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 assert.equal(commands.filter(command=>command.type==='abort').length,2);
 await close(child,f.state);
});

test('quit before delayed acceptance cancels the actual start before resumption',async t=>{
 const f=await fixture(t);const child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 const {socket,send}=await attach(f.root,f.endpoint);t.after(()=>socket.destroy());
 send({type:'command',id:randomUUID(),command:{command:'prompt',text:'gate acceptance hold question',attachments:[]}});
 await until(()=>readFile(join(f.state,'acceptance-held'),'utf8'),value=>value==='held',child);
 await close(child,f.state);
 const interrupted=await f.checkpoint();
 assert.equal(interrupted.outcome,'interrupted');
 assert.equal(interrupted.request.phase,'interrupted');
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 assert.equal(commands.filter(command=>command.type==='abort').length,2);
});

test('reopening an unused session does not require a Pi file that never materialized',async t=>{
 const f=await fixture(t,true,true);let child=f.launch();
 const first=await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 assert.equal(first.request,null);
 await assert.rejects(readFile(first.pi_session_file),error=>error.code==='ENOENT');
 await close(child,f.state);
 child=f.launch();
 const restored=await until(f.checkpoint,state=>state.helper_run_id!==first.helper_run_id && state.transcript.status.state==='idle',child);
 assert.equal(restored.request,null);
 assert.equal(restored.outcome,'ready');
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 assert.ok(!commands.some(command=>command.type==='prompt'));
 await close(child,f.state);
});

test('retry attempts stay working and reject a second prompt until the recovered run settles',async t=>{
 const f=await fixture(t);const child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 const {socket,send,messages}=await attach(f.root,f.endpoint);t.after(()=>socket.destroy());
 send({type:'command',id:randomUUID(),command:{command:'prompt',text:'retry gate hold question',attachments:[]}});
 await until(()=>readFile(join(f.state,'retry-held'),'utf8'),value=>value==='held',child);
 const backoff=await until(f.checkpoint,state=>state.transcript.entries.some(entry=>entry.kind?.text?.startsWith('Retrying after an error:')),child);
 assert.equal(backoff.outcome,'answering');assert.equal(backoff.request.phase,'accepted');
 assert.equal(backoff.transcript.status.state,'working');
 const second=randomUUID();
 send({type:'command',id:second,command:{command:'prompt',text:'second question',attachments:[]}});
 const rejection=await until(async()=>messages.find(message=>message.type==='ack'&&message.id===second),message=>Boolean(message),child);
 assert.equal(rejection.rejected,'unsupported');
 await writeFile(join(f.state,'retry-release'),'recover');
 const settled=await until(f.checkpoint,state=>state.outcome==='completed',child);
 assert.equal(settled.transcript.status.state,'idle');
 assert.ok(settled.transcript.entries.some(entry=>entry.kind?.text?.includes('fixture recovered answer')));
 assert.equal(settled.transcript.entries.filter(entry=>entry.kind?.type==='turn').at(-1)?.kind?.end,'done');
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 assert.equal(commands.filter(command=>command.type==='prompt').length,1);
 await close(child,f.state);
});

for(const action of ['Stop','quit']) {
 test(`${action} during retry backoff uses authoritative aborted settlement`,async t=>{
  const f=await fixture(t);const child=f.launch();
  await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
  const {socket,send}=await attach(f.root,f.endpoint);t.after(()=>socket.destroy());
  send({type:'command',id:randomUUID(),command:{command:'prompt',text:'retry gate hold question',attachments:[]}});
  await until(()=>readFile(join(f.state,'retry-held'),'utf8'),value=>value==='held',child);
  const backoff=await until(f.checkpoint,state=>state.transcript.entries.some(entry=>entry.kind?.text?.startsWith('Retrying after an error:')),child);
  assert.equal(backoff.outcome,'answering');
  if(action==='Stop') {
   await writeFile(join(f.state,'stop'),'stop');
   await until(f.checkpoint,state=>state.outcome==='stopped'&&state.transcript.status.state==='idle',child);
  }
  await close(child,f.state);
  const settled=await f.checkpoint();
  assert.equal(settled.outcome,action==='Stop'?'stopped':'interrupted');
  assert.equal(settled.request.phase,action==='Stop'?'stopped':'interrupted');
  assert.equal(settled.transcript.entries.filter(entry=>entry.kind?.type==='turn').at(-1)?.kind?.end,'cancelled');
  const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
  assert.equal(commands.filter(command=>command.type==='prompt').length,1);
  assert.ok(commands.some(command=>command.type==='abort'));
 });
}

test('an aborted attempt without a run abort never becomes a completed request',async t=>{
 const f=await fixture(t);const child=f.launch();
 await until(f.checkpoint,state=>state.transcript.status.state==='idle',child);
 const {socket,send}=await attach(f.root,f.endpoint);t.after(()=>socket.destroy());
 const text='attempt abort without cancel';
 send({type:'command',id:randomUUID(),command:{command:'prompt',text,attachments:[]}});
 const settled=await until(f.checkpoint,state=>state.request?.text===text && ['completed','unknown'].includes(state.request.phase) && state.transcript.turn===null,child);
 assert.equal(settled.outcome,'unknown');
 assert.equal(settled.request.phase,'unknown');
 assert.equal(settled.transcript.entries.filter(entry=>entry.kind?.type==='turn').at(-1)?.kind?.end,'cancelled');
 const commands=(await readFile(join(f.state,'commands.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
 assert.equal(commands.filter(command=>command.type==='prompt').length,1);
 await close(child,f.state);
});
