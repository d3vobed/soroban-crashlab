import * as assert from 'node:assert/strict';
import { createLogger } from './logger';

function makeCapture(): { lines: string[]; write: (line: string) => void } {
  const lines: string[] = [];
  return { lines, write: (line: string) => lines.push(line) };
}

// info level: required fields present
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  log.info('hello world');
  assert.strictEqual(cap.lines.length, 1);
  const entry = JSON.parse(cap.lines[0]);
  assert.strictEqual(entry.level, 'info');
  assert.strictEqual(entry.msg, 'hello world');
  assert.ok(typeof entry.time === 'string', 'time must be a string');
  assert.ok(!Number.isNaN(Date.parse(entry.time)), 'time must be a valid ISO date');
}

// warn level: required fields present
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  log.warn('watch out');
  const entry = JSON.parse(cap.lines[0]);
  assert.strictEqual(entry.level, 'warn');
  assert.strictEqual(entry.msg, 'watch out');
  assert.ok(typeof entry.time === 'string');
}

// error level: required fields present
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  log.error('something broke');
  const entry = JSON.parse(cap.lines[0]);
  assert.strictEqual(entry.level, 'error');
  assert.strictEqual(entry.msg, 'something broke');
  assert.ok(typeof entry.time === 'string');
}

// extra fields are spread into the entry
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  log.info('request completed', { route: '/api/runs', method: 'GET', status: 200 });
  const entry = JSON.parse(cap.lines[0]);
  assert.strictEqual(entry.route, '/api/runs');
  assert.strictEqual(entry.method, 'GET');
  assert.strictEqual(entry.status, 200);
}

// Error objects in fields are serialized to their message string
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  log.error('handler failed', { error: new Error('database unavailable') });
  const entry = JSON.parse(cap.lines[0]);
  assert.strictEqual(entry.error, 'database unavailable');
}

// output is always valid JSON
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  log.info('test entry', { key: 'value', num: 42, flag: true, nested: { a: 1 } });
  assert.doesNotThrow(() => JSON.parse(cap.lines[0]));
}

// calling without optional fields does not throw
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  assert.doesNotThrow(() => log.info('no fields'));
  assert.doesNotThrow(() => log.warn('no fields'));
  assert.doesNotThrow(() => log.error('no fields'));
  assert.strictEqual(cap.lines.length, 3);
}

// each call produces exactly one line
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  log.info('a');
  log.warn('b');
  log.error('c');
  assert.strictEqual(cap.lines.length, 3);
  const levels = cap.lines.map((l) => JSON.parse(l).level);
  assert.deepEqual(levels, ['info', 'warn', 'error']);
}

// level field is preserved as a string (not overwritten by extra fields)
{
  const cap = makeCapture();
  const log = createLogger(cap.write);
  log.info('msg', { level: 'should-not-override' });
  const entry = JSON.parse(cap.lines[0]);
  assert.strictEqual(entry.level, 'should-not-override');
}

console.log('logger.test.ts: all assertions passed');
