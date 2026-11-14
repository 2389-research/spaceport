// ABOUTME: Runs an explicitly approved real-provider Ask smoke check against a private app.
// ABOUTME: Keeps provider sign-in in Pi and verifies cancellation, recovery, and exact child cleanup.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdtemp, mkdir, readFile, writeFile, symlink, chmod, open, stat, realpath, rm} from 'node:fs/promises';
import {homedir} from 'node:os';
import {join, dirname, isAbsolute} from 'node:path';
import {randomUUID} from 'node:crypto';
import net from 'node:net';
import {setTimeout as delay} from 'node:timers/promises';

const [mode, app, helper, pi, rawResumeRoot, scenario = 'completed'] = process.argv.slice(2);
const resumeRoot = rawResumeRoot === '-' ? undefined : rawResumeRoot;
if (mode !== '--run-approved-provider-test' || ![app, helper, pi].every(value => value && isAbsolute(value))) {
  console.error('Usage: node provider.mjs --run-approved-provider-test /absolute/spaceport /absolute/sheprd-acp /absolute/pi [completed-fixture-root|-] [completed|lifecycle]');
  console.error('Requires Doctor Biz approval. Pi uses its own existing sign-in and model settings.');
  console.error('Only the test question and private disconnected-app coverage reach the configured provider.');
  process.exit(mode === '--help' ? 0 : 2);
}
assert.ok(['completed', 'lifecycle'].includes(scenario), 'Choose the completed or lifecycle scenario.');
assert.equal(process.platform, 'darwin', 'This native provider qualification harness requires macOS.');
const root = resumeRoot || await mkdtemp('/tmp/spaq-');
let saved;
let previousReport;
if (resumeRoot) {
  previousReport = JSON.parse(await readFile(join(root, 'report.json'), 'utf8'));
  assert.equal(await realpath(root), await realpath(previousReport.root));
  const info = await stat(root);
  assert.equal(info.uid, process.getuid());
  assert.equal(info.mode & 0o777, 0o700);
  saved = JSON.parse(await readFile(join(root, 'conversation', 'checkpoint.json'), 'utf8'));
  assert.equal(saved.outcome, 'completed', 'Resume-only qualification cannot submit an unfinished request.');
} else {
  await chmod(root, 0o700);
  for (const name of ['home', 'state', 'config', 'cache', 'bin', 'conversation']) await mkdir(join(root, name), {mode: 0o700});
  await symlink(pi, join(root, 'bin', 'pi'));
}
const broker = join(root, 'state', 'spaceport', 'control.sock');
const state = join(root, 'conversation');
const endpoint = saved?.endpoint || randomUUID().replaceAll('-', '');
const conversation = saved?.conversation_id || randomUUID();
const children = [];
const report = {...previousReport, scope: 'Real installed Pi and configured provider; real private Spaceport observation broker; no personal daemon or session content.', root, completed: false, error: undefined, priorHarnessError: previousReport?.error, newPromptsSubmitted: saved ? 0 : 1};
report.userMessagesAtStart = saved?.transcript.entries.filter(entry => entry.kind.type === 'user').length || 0;

async function launch(program, args, env, label) {
  const log = await open(join(root, label + '.log'), 'w', 0o600);
  try {
    const child = spawn(program, args, {env, cwd: root, stdio: ['ignore', log.fd, log.fd]});
    children.push(child);
    child.on('error', error => { child.startError = error; });
    return child;
  } finally { await log.close(); }
}
async function until(read, predicate, child, seconds = 30) {
  const end = Date.now() + seconds * 1000;
  while (Date.now() < end) {
    if (child?.startError) throw child.startError;
    if (child && (child.exitCode !== null || child.signalCode !== null)) throw new Error('Owned child exited before expected state; inspect private logs.');
    try { const value = await read(); if (predicate(value)) return value; }
    catch (error) { if (!['ENOENT', 'ECONNREFUSED'].includes(error.code) && !(error instanceof SyntaxError)) throw error; }
    await delay(50);
  }
  throw new Error('Bounded qualification wait expired; no prompt was retried.');
}
async function call(request) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection(broker);
    let bytes = '';
    let settled = false;
    const finish = (error, value) => {
      if (settled) return;
      settled = true;
      socket.destroy();
      if (error) reject(error); else resolve(value);
    };
    socket.setTimeout(5000, () => finish(new Error('Private broker timed out.')));
    socket.on('error', error => finish(error));
    socket.on('end', () => finish(new Error('Private broker closed before its response.')));
    socket.on('connect', () => socket.write(JSON.stringify(request) + '\n'));
    socket.on('data', chunk => {
      bytes += chunk.toString();
      if (bytes.length > 256 * 1024) { finish(new Error('Broker response exceeded bound.')); return; }
      const end = bytes.indexOf('\n');
      if (end >= 0) { try { finish(null, JSON.parse(bytes.slice(0, end))); } catch (error) { finish(error); } }
    });
  });
}
const checkpoint = async () => JSON.parse(await readFile(join(state, 'checkpoint.json'), 'utf8'));
async function piSubmissions(saved) {
  const session = await realpath(saved.pi_session_file);
  assert.equal(dirname(session), await realpath(join(state, 'sessions')));
  assert.ok((await stat(session)).size <= 4 * 1024 * 1024, 'Private Pi evidence exceeds the fixture bound.');
  return (await readFile(session, 'utf8')).trim().split('\n').map(JSON.parse)
    .filter(entry => entry.type === 'message' && entry.message.role === 'user')
    .map(entry => entry.message.content);
}
async function attachHelper(waitIdle = true) {
  const socket = net.createConnection(join(root, 'helper-state', 'herdr', 'acp', endpoint + '.sock'));
  await new Promise((resolve, reject) => { socket.once('connect', resolve); socket.once('error', reject); });
  let bytes = Buffer.alloc(0);
  let status;
  let snapshot = false;
  let failure;
  const receipts = new Map();
  socket.on('error', error => { failure = error; });
  socket.on('data', chunk => {
    bytes = Buffer.concat([bytes, chunk]);
    try {
      while (bytes.length >= 4) {
        const size = bytes.readUInt32LE(0);
        assert.ok(size <= 16 * 1024 * 1024, 'Attach frame exceeds its protocol bound.');
        if (bytes.length < size + 4) break;
        const message = JSON.parse(bytes.subarray(4, size + 4));
        bytes = bytes.subarray(size + 4);
        if (message.type === 'snapshot') { snapshot = true; status = message.transcript.status; }
        if (message.type === 'change' && message.change.change === 'status') status = message.change.status;
        if (message.type === 'ack') { receipts.set(message.id, message.rejected); assert.ok(receipts.size <= 8); }
      }
    } catch (error) { failure = error; socket.destroy(); }
  });
  const send = value => {
    const body = Buffer.from(JSON.stringify(value)); const prefix = Buffer.alloc(4); prefix.writeUInt32LE(body.length);
    socket.write(Buffer.concat([prefix, body]));
  };
  send({type: 'hello', client: 'gui', revision: 1, since: null, version: 1});
  await until(async () => { if (failure) throw failure; return snapshot && (!waitIdle || status?.state === 'idle'); }, Boolean);
  const accepted = async id => {
    await until(async () => { if (failure) throw failure; return receipts.has(id); }, Boolean);
    assert.equal(receipts.get(id), null, 'Helper must acknowledge this exact command without rejection.');
  };
  return {socket, send, accepted};
}
async function progress(stage) {
  report.stage = stage;
  await writeFile(join(root, 'report.json'), JSON.stringify(report, null, 2) + '\n', {mode: 0o600});
  console.log(JSON.stringify({stage, evidence: join(root, 'report.json')}));
}
async function exited(child, seconds = 20) {
  await until(async () => child.exitCode !== null || child.signalCode !== null, Boolean, undefined, seconds);
  assert.equal(child.exitCode, 0, 'Owned child must exit successfully.');
}
async function quitHelper(child) {
  await writeFile(join(state, 'quit'), 'quit', {mode: 0o600});
  await exited(child, 10);
}
let gui;
let activeHelper;
try {
  const appEnv = {HOME: join(root, 'home'), PATH: '/usr/bin:/bin', XDG_STATE_HOME: join(root, 'state'), XDG_CONFIG_HOME: join(root, 'config'), XDG_CACHE_HOME: join(root, 'cache'), TMPDIR: root};
  gui = await launch(app, ['--socket', join(root, 'deliberately-unavailable.sock')], appEnv, saved ? 'app-resume' : 'app');
  const status = await until(() => call({method: 'status'}), value => value.windows?.length > 0, gui);
  assert.equal(status.pid, gui.pid, 'Broker must belong to the exact owned app.');
  report.appPid = status.pid;
  const observed = await call({method: 'observation', caller: {}, op: {op: 'sessions', offset: 0, limit: 1}});
  assert.equal(observed.status, 'observation');
  assert.equal(observed.reply.result, 'sessions');
  assert.deepEqual(observed.reply.sessions, [], 'Refuse provider startup if the private app sees any live session.');
  const helperEnv = {HOME: homedir(), PATH: join(root, 'bin') + ':' + dirname(process.execPath) + ':/usr/bin:/bin', XDG_STATE_HOME: join(root, 'helper-state'), TMPDIR: root};
  if (process.env.PI_CODING_AGENT_DIR) helperEnv.PI_CODING_AGENT_DIR = process.env.PI_CODING_AGENT_DIR;
  const args = ['ask', '--state-dir', state, '--endpoint', endpoint, '--broker', broker, '--conversation', conversation];
  let completed = saved;
  if (!saved) {
  activeHelper = await launch(helper, args, helperEnv, 'helper-first');
  await until(checkpoint, value => value.transcript.status.state === 'idle', activeHelper);
  const {socket, send, accepted} = await attachHelper();
  const promptId = randomUUID();
  send({type: 'command', id: promptId, command: {command: 'prompt', text: 'This is an authorized isolated Spaceport qualification. Use spaceport_list_sessions to inspect this app. Report its connected-session coverage honestly, including unavailable connections. Do not invent sessions. Keep the answer brief.', attachments: []}});
  await accepted(promptId);
  try {
    completed = await until(checkpoint, value => ['completed', 'failed', 'unknown'].includes(value.outcome), activeHelper, 120);
  } finally { socket.destroy(); }
  assert.equal(completed.outcome, 'completed', 'Real provider turn must finish successfully.');
  const entries = completed.transcript.entries;
  const tools = entries.filter(entry => entry.kind.type === 'tool').map(entry => entry.kind);
  assert.ok(tools.some(tool => tool.title === 'spaceport_list_sessions' && tool.status === 'completed'), 'Real provider must invoke the observation tool successfully.');
  const allowed = new Set(['spaceport_list_sessions', 'spaceport_read_conversation', 'spaceport_read_terminal']);
  assert.ok(tools.every(tool => allowed.has(tool.title)), 'No other tool may execute.');
  const results = tools.flatMap(tool => tool.content.filter(content => content.type === 'text').map(content => JSON.parse(content.text)));
  assert.ok(results.some(result => result.result === 'sessions' && result.sessions.length === 0), 'Tool transcript must contain the real empty session reply.');
  report.toolNames = tools.map(tool => tool.title);
  report.firstRun = completed.helper_run_id;
  await quitHelper(activeHelper);
  }
  // Worker normally clears this marker before launch. Reproduce that exact launch precondition.
  await rm(join(state, 'quit'), {force: true});
  const completedSubmissions = await piSubmissions(completed);
  activeHelper = await launch(helper, args, helperEnv, 'helper-reopen');
  const restored = await until(checkpoint, value => value.helper_run_id !== completed.helper_run_id, activeHelper);
  const completedLive = await attachHelper();
  assert.deepEqual(await piSubmissions(await checkpoint()), completedSubmissions, 'Completed reopen must not submit even a hidden Pi continuation.');
  completedLive.socket.destroy();
  assert.deepEqual(restored.request, completed.request, 'Completed request must not replay.');
  assert.equal(restored.transcript.entries.filter(entry => entry.kind.type === 'user').length, completed.transcript.entries.filter(entry => entry.kind.type === 'user').length);
  report.completedRequestPreserved = true;
  report.reopenedRun = restored.helper_run_id;
  if (scenario === 'lifecycle') {
    const userCount = value => value.transcript.entries.filter(entry => entry.kind.type === 'user').length;
    const answering = value => value.outcome === 'answering' && value.request?.phase === 'accepted';
    const question = purpose => `Synthetic Spaceport ${purpose} qualification. Use spaceport_list_sessions once. Then write 96 numbered short sentences about observing synthetic session status. Do not use personal data or other tools. This long requested answer lets the test interrupt a genuine streamed response.`;
    async function startStream(purpose) {
      const before = await checkpoint();
      const connection = await attachHelper();
      const id = randomUUID();
      report.newPromptsSubmitted += 1;
      connection.send({type: 'command', id, command: {command: 'prompt', text: question(purpose), attachments: []}});
      await connection.accepted(id);
      await until(checkpoint, value => value.request?.id === id && answering(value), activeHelper, 120);
      await until(checkpoint, value => value.request?.id === id && answering(value) && value.transcript.entries.slice(before.transcript.entries.length).some(entry => entry.kind.type === 'agent' && entry.kind.text.length > 0), activeHelper, 120);
      return {id, connection};
    }
    async function saveStop(connection) {
      await writeFile(join(state, 'stop'), 'stop', {mode: 0o600});
      connection.send({type: 'command', id: randomUUID(), command: {command: 'cancel'}});
      return until(checkpoint, value => value.outcome === 'stopped' && value.request?.phase === 'stopped' && value.transcript.status.state === 'idle', activeHelper);
    }
    await progress('starting-real-stream-for-explicit-stop');
    const stopTurn = await startStream('explicit-stop');
    const stopped = await saveStop(stopTurn.connection);
    stopTurn.connection.socket.destroy();
    report.explicitStopDuringRealStream = true;
    await quitHelper(activeHelper);
    const stoppedSubmissions = await piSubmissions(stopped);
    activeHelper = await launch(helper, args, helperEnv, 'helper-stopped-reopen');
    const stoppedReopen = await until(checkpoint, value => value.helper_run_id !== stopped.helper_run_id && value.transcript.status.state === 'idle', activeHelper);
    const stoppedLive = await attachHelper();
    assert.deepEqual(await piSubmissions(await checkpoint()), stoppedSubmissions, 'Stopped reopen must not submit even a hidden Pi continuation.');
    stoppedLive.socket.destroy();
    assert.equal(stoppedReopen.outcome, 'stopped');
    assert.equal(userCount(stoppedReopen), userCount(stopped));
    report.stoppedReopenDidNotReplay = true;
    await progress('starting-real-stream-for-graceful-interruption');
    const interruptedTurn = await startStream('graceful-interruption');
    await quitHelper(activeHelper);
    interruptedTurn.connection.socket.destroy();
    const interrupted = await checkpoint();
    assert.equal(interrupted.outcome, 'interrupted', 'Actual Pi must confirm cancellation before automatic continuation is eligible.');
    assert.equal(interrupted.request.phase, 'interrupted');
    report.confirmedGracefulInterruption = true;
    await progress('reopening-confirmed-unfinished-real-turn');
    activeHelper = await launch(helper, args, helperEnv, 'helper-interrupted-reopen');
    const resumed = await until(checkpoint, value => value.helper_run_id !== interrupted.helper_run_id && answering(value), activeHelper, 120);
    assert.equal(resumed.pi_session, interrupted.pi_session);
    assert.equal(resumed.pi_session_file, interrupted.pi_session_file);
    assert.equal(resumed.request.id, interrupted.request.id);
    assert.equal(userCount(resumed), userCount(interrupted), 'Continuation must not duplicate the user question.');
    report.exactSessionContinuationAccepted = true;
    const continuing = await attachHelper(false);
    await saveStop(continuing);
    continuing.socket.destroy();
    report.resumedTurnStopped = true;
  }
  await quitHelper(activeHelper);
  report.piUserSubmissions = (await piSubmissions(await checkpoint())).length;
  await call({method: 'app', caller: {}, op: {op: 'quit'}});
  await exited(gui);
  report.completed = true;
  report.userMessagesAtEnd = (await checkpoint()).transcript.entries.filter(entry => entry.kind.type === 'user').length;
  report.ownedChildrenReaped = children.every(child => child.exitCode !== null || child.signalCode !== null);
} catch (error) {
  report.error = error.message;
  if (activeHelper && activeHelper.exitCode === null && activeHelper.signalCode === null) {
    try { await quitHelper(activeHelper); } catch (cleanup) { report.helperCleanupError = cleanup.message; }
  }
  if (gui && gui.exitCode === null && gui.signalCode === null) {
    try { await call({method: 'app', caller: {}, op: {op: 'quit'}}); await exited(gui); }
    catch (cleanup) { report.appCleanupError = cleanup.message; }
  }
  process.exitCode = 1;
} finally {
  await writeFile(join(root, 'report.json'), JSON.stringify(report, null, 2) + '\n', {mode: 0o600});
  console.log(JSON.stringify({completed: report.completed, evidence: join(root, 'report.json'), error: report.error}));
}
