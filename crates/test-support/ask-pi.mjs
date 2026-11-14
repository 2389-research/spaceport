#!/usr/bin/env node
// ABOUTME: Implements isolated Pi RPC fixture responses without model or provider calls.
// ABOUTME: Exercises helper recovery and ignored cancellation with private cleanup markers.
import {createInterface} from 'node:readline';
import {writeFileSync,mkdirSync,appendFileSync,existsSync} from 'node:fs';
import {join} from 'node:path';
if(process.argv.includes('--version')) {console.log('1.5.2');process.exit(0);}
const cwd=process.cwd();
const sessionFile=join(cwd,'sessions','fixture.jsonl');
mkdirSync(join(cwd,'sessions'),{recursive:true});
if(process.env.SPACEPORT_FIXTURE_EMPTY_SESSION!=='1') writeFileSync(sessionFile,'fixture session');
writeFileSync(join(cwd,'pid'),String(process.pid));
const ignoredAbort=process.env.SPACEPORT_FIXTURE_IGNORE_ABORT==='1';
if(ignoredAbort) {
 process.on('SIGTERM',()=>writeFileSync(join(cwd,'termination-observed'),'observed'));
 setInterval(()=>{if(existsSync(join(cwd,'fixture-stop')))process.exit(0);},20);
}
const output=value=>console.log(JSON.stringify(value));
if(process.env.SPACEPORT_FIXTURE_READY!=='0') process.stderr.write(JSON.stringify({type:'spaceport_ask_ready',version:'1.5.2',tools:['spaceport_list_sessions','spaceport_read_conversation','spaceport_read_terminal']})+String.fromCharCode(10));
const response=(command,data)=>output({type:'response',id:command.id,command:command.type,success:true,data});
const end=reason=>{
 output({type:'agent_end',messages:[{role:'assistant',stopReason:reason}]});
 output({type:'agent_settled',aborted:reason==='aborted'});
};
let delayedPrompt=null;
let retryTimer=null;
let retryPending=false;

for await(const line of createInterface({input:process.stdin})) {
 const command=JSON.parse(line);
 appendFileSync(join(cwd,'commands.jsonl'),JSON.stringify(command)+'\n');
 if(command.type==='get_state') response(command,{sessionId:'fixture-session',sessionFile});
 else if(command.type==='get_available_models') response(command,{models:[]});
 else if(command.type==='get_commands') response(command,{commands:[]});
 else if(command.type==='prompt') {
  if(command.message.includes('gate acceptance')) {
   delayedPrompt=command;
   writeFileSync(join(cwd,'acceptance-held'),'held');
   continue;
  }
  response(command,{disposition:'started'});
  output({type:'agent_start'});
  if(command.message.includes('retry gate')) {
   retryPending=true;
   output({type:'agent_end',willRetry:true,messages:[{role:'assistant',stopReason:'error',errorMessage:'fixture retryable failure'}]});
   output({type:'auto_retry_start',errorMessage:'fixture retryable failure'});
   writeFileSync(join(cwd,'retry-held'),'held');
   retryTimer=setInterval(()=>{
    if(existsSync(join(cwd,'fixture-stop')))process.exit(0);
    if(!existsSync(join(cwd,'retry-release')))return;
    clearInterval(retryTimer);retryPending=false;
    output({type:'auto_retry_end',success:true});
    output({type:'message_start',message:{role:'assistant'}});
    output({type:'message_update',assistantMessageEvent:{type:'text_delta',delta:'fixture recovered answer'}});
    end('stop');
   },20);
   continue;
  }
  output({type:'message_start'
,message:{role:'assistant'}});
  output({type:'message_update',assistantMessageEvent:{type:'text_delta',delta:'fixture partial answer'}});
  if(command.message.includes('attempt abort without cancel')) {
   output({type:'agent_end',willRetry:false,messages:[{role:'assistant',stopReason:'aborted'}]});
   output({type:'agent_settled',aborted:false});
   continue;
  }
  if(!command.message.includes('hold')) end('stop');

 } else if(command.type==='abort') {
  if(ignoredAbort) {writeFileSync(join(cwd,'abort-observed'),'observed');response(command,{});continue;}
  response(command,{});
  if(retryPending) {
   clearInterval(retryTimer);retryPending=false;
   output({type:'agent_settled',aborted:true});
   continue;
  }
  if(delayedPrompt) {
   response(delayedPrompt,{disposition:'started'});
   output({type:'agent_start'});
   output({type:'message_start',message:{role:'assistant'}});
   output({type:'message_update',assistantMessageEvent:{type:'text_delta',delta:'started after early cancellation'}});
   delayedPrompt=null;
   continue;
  }
  end('aborted');
 } else response(command,{});
}
