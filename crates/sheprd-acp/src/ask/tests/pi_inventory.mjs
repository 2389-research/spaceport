// ABOUTME: Qualifies the shipped extension in an actual installed Pi without model prompts.
// ABOUTME: Uses isolated hostile configuration and reaps only the exact owned fixture processes.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, readFile, writeFile, realpath, access } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { randomUUID } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';

if(!process.argv[2]) throw new Error('Usage: node pi_inventory.mjs /absolute/path/to/pi [private-evidence-directory]');
const program=await realpath(process.argv[2]);
const root=await mkdtemp(join(resolve(process.argv[3]||'/tmp'),'ask-pi-'));
for(const name of ['home','config','sessions','work','tmp'])await mkdir(join(root,name),{mode:0o700});
const sentinel=join(root,'unowned.ts');
const activated=join(root,'unowned-activated');
await writeFile(sentinel,`import {writeFileSync} from 'node:fs';export default function(){writeFileSync(${JSON.stringify(activated)},'unowned extension ran');}`,{mode:0o600});
await writeFile(join(root,'config','settings.json'),JSON.stringify({extensions:[sentinel],defaultProvider:'openai',defaultModel:'gpt-4o-mini'}),{mode:0o600});
const extension=join(root,'observation.ts');
const source=(await readFile(new URL('../observation.ts',import.meta.url),'utf8')).replace('__SPACEPORT_BROKER_JSON__',JSON.stringify(join(root,'unused.sock')));
await writeFile(extension,source,{mode:0o600});
const tools=['spaceport_list_sessions','spaceport_read_conversation','spaceport_read_terminal'];
const args=['--mode','rpc','--offline','--no-approve','--no-extensions','--no-mcp','--no-skills','--no-prompt-templates','--no-context-files','--no-themes','--tools',tools.join(','),'--system-prompt','Spaceport isolated observation qualification. No model prompt will be sent.','--session-dir',join(root,'sessions'),'-e',extension];
const env={HOME:join(root,'home'),PATH:dirname(process.execPath)+':/usr/bin:/bin',PI_CODING_AGENT_DIR:join(root,'config'),PI_OFFLINE:'1',PI_TELEMETRY:'0',TMPDIR:join(root,'tmp'),XDG_CACHE_HOME:join(root,'home','.cache'),XDG_CONFIG_HOME:join(root,'home','.config')};
const reports=[];

async function inspect(label,load,expectedId) {
 const child=spawn(program,[...args,...(load?['--session',load]:[])],{cwd:join(root,'work'),env,stdio:['pipe','pipe','pipe']});
 let output='',errors='',closed=false,code,signal;const messages=[];
 const exited=new Promise(resolve=>child.once('close',(status,sig)=>{closed=true;code=status;signal=sig;resolve();}));
 child.stderr.on('data',chunk=>{errors+=chunk;for(const line of errors.split('\n')){try{const value=JSON.parse(line);if(value.type==='spaceport_ask_ready'&&!messages.some(message=>message.type===value.type))messages.push(value);}catch{}}if(errors.length>65536)child.kill('SIGTERM');});
 child.stdout.on('data',chunk=>{output+=chunk;if(output.length>65536){child.kill('SIGTERM');return;}let newline;while((newline=output.indexOf('\n'))>=0){const line=output.slice(0,newline);output=output.slice(newline+1);try{messages.push(JSON.parse(line));}catch{}}});
 const wait=async predicate=>{for(let i=0;i<300;i++){const found=messages.find(predicate);if(found)return found;if(closed)throw new Error(`Pi exited before ${label} inspection: ${errors}`);await delay(20);}throw new Error(`Pi ${label} inspection timed out: ${errors}`);};
 let result;
 try {
  const ready=await wait(message=>message.type==='spaceport_ask_ready');
  assert.equal(ready.version,'1.5.2');assert.deepEqual(ready.tools,tools);
  child.stdin.write(JSON.stringify({type:'get_state',id:'state'})+'\n');
  child.stdin.write(JSON.stringify({type:'get_commands',id:'commands'})+'\n');
  const state=await wait(message=>message.id==='state');const commands=await wait(message=>message.id==='commands');
  assert.equal(state.success,true);assert.equal(commands.success,true);assert.deepEqual(commands.data.commands,[]);
  if(expectedId)assert.equal(state.data.sessionId,expectedId);
  await assert.rejects(access(activated));
  result={label,ready,sessionId:state.data.sessionId,configuredProvider:state.data.model?.provider,configuredModel:state.data.model?.id,unownedExtensionActivated:false,promptSent:false};
 } finally {
  child.stdin.end();
  if(!closed)child.kill('SIGTERM');
  const timer=setTimeout(()=>{if(!closed)child.kill('SIGKILL');},1000);
  await exited;clearTimeout(timer);
  if(result)result.child={pid:child.pid,reaped:true,code,signal};
 }
 reports.push(result);
}

await inspect('new-session');
const sessionId=randomUUID();const session=join(root,'sessions','owned-restore.jsonl');
await writeFile(session,JSON.stringify({type:'session',version:3,id:sessionId,timestamp:new Date().toISOString(),cwd:join(root,'work')})+'\n',{mode:0o600});
await inspect('exact-session-restore',session,sessionId);
const report={scope:'Production extension loader and executable inventory; no model prompt, personal configuration, credentials, or provider calls.',program,root,reports,remaining:['real provider tool dispatch','model change and reload','streaming cancellation and durable recovery with real provider']};
await writeFile(join(root,'report.json'),JSON.stringify(report,null,2)+'\n',{mode:0o600});
console.log(JSON.stringify(report,null,2));
