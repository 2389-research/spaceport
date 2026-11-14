// ABOUTME: Verifies the private live lifecycle evidence without displaying transcript content.
// ABOUTME: Cross-checks request counts, allowed tools, and file permissions against durable transcript evidence.
import assert from 'node:assert/strict';
import {readFile, stat, realpath} from 'node:fs/promises';
import {join, dirname} from 'node:path';
const root = process.argv[2];
assert.ok(root, 'Usage: node provider_lifecycle_audit.mjs /absolute/private/fixture');
const reportPath = join(root, 'report.json');
const report = JSON.parse(await readFile(reportPath, 'utf8'));
const checkpointPath = join(root, 'conversation', 'checkpoint.json');
const saved = JSON.parse(await readFile(checkpointPath, 'utf8'));
assert.equal((await stat(root)).mode & 0o777, 0o700);
assert.equal((await stat(root)).uid, process.getuid());
assert.equal((await stat(checkpointPath)).mode & 0o777, 0o600);
for (const name of ['completed', 'completedRequestPreserved', 'explicitStopDuringRealStream', 'stoppedReopenDidNotReplay', 'confirmedGracefulInterruption', 'exactSessionContinuationAccepted', 'resumedTurnStopped', 'ownedChildrenReaped']) assert.equal(report[name], true, name);
assert.equal(saved.outcome, 'stopped');
assert.equal(saved.request.phase, 'stopped');
const users = saved.transcript.entries.filter(entry => entry.kind.type === 'user');
assert.equal(users.length, 3);
for (const purpose of ['explicit-stop', 'graceful-interruption']) assert.equal(users.filter(entry => entry.kind.text.startsWith('Synthetic Spaceport ' + purpose + ' qualification.')).length, 1);
const allowed = new Set(['spaceport_list_sessions', 'spaceport_read_conversation', 'spaceport_read_terminal']);
const tools = saved.transcript.entries.filter(entry => entry.kind.type === 'tool');
assert.ok(tools.length >= 1);
assert.ok(tools.every(entry => allowed.has(entry.kind.title)));
assert.equal(report.userMessagesAtEnd, users.length);
assert.equal(report.newPromptsSubmitted, users.length - report.userMessagesAtStart);
const session = await realpath(saved.pi_session_file);
assert.equal(dirname(session), await realpath(join(root, 'conversation', 'sessions')));
assert.ok((await stat(session)).size <= 4 * 1024 * 1024);
const submissions = (await readFile(session, 'utf8')).trim().split('\n').map(JSON.parse)
  .filter(entry => entry.type === 'message' && entry.message.role === 'user')
  .map(entry => entry.message.content.filter(block => block.type === 'text').map(block => block.text).join('\n'));
assert.equal(submissions.length, 4, 'Three questions and one continuation; no hidden completed/stopped replay.');
assert.equal(submissions.filter(text => text.startsWith('Spaceport question (literal user text follows):')).length, 3);
assert.equal(submissions.filter(text => text.startsWith('Continue the unfinished Ask Spaceport question using the saved conversation.')).length, 1);
for (const user of users) assert.equal(submissions.filter(text => text === 'Spaceport question (literal user text follows):\n' + user.kind.text).length, 1);
console.log(JSON.stringify({completed: true, userMessages: users.length, piSubmissions: submissions.length, newPromptsSubmitted: report.newPromptsSubmitted, lifecyclePrompts: 2, finalOutcome: saved.outcome, allowedToolCalls: tools.length, report: reportPath}));
