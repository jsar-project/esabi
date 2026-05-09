import assert from 'node:assert/strict';
import test from 'node:test';

import { formatRunResultDisplay } from './display.ts';
import type { RunResult } from './types.ts';

test('formats execution errors with stack text', () => {
  const runResult: RunResult = {
    ok: false,
    mode: 'script',
    result: null,
    stdout: ['before throw'],
    stderr: [],
    error: {
      kind: 'execution',
      name: 'TypeError',
      message: 'Playground demo failure',
      stack: 'TypeError: Playground demo failure\n    at <eval> (playground-entry:2)'
    }
  };

  const display = formatRunResultDisplay(runResult);

  assert.equal(display.hasError, true);
  assert.equal(display.errorKindText, 'execution');
  assert.equal(display.errorNameText, 'TypeError');
  assert.equal(display.errorMessageText, 'Playground demo failure');
  assert.match(display.errorStackText, /TypeError: Playground demo failure/);
  assert.equal(display.stdoutText, 'before throw');
  assert.equal(display.stderrText, 'No stderr.');
});

test('formats execution errors without stack text', () => {
  const runResult: RunResult = {
    ok: false,
    mode: 'script',
    result: null,
    stdout: [],
    stderr: [],
    error: {
      kind: 'execution',
      name: 'ThrownValue',
      message: 'plain failure',
      stack: null
    }
  };

  const display = formatRunResultDisplay(runResult);

  assert.equal(display.hasError, true);
  assert.equal(display.errorKindText, 'execution');
  assert.equal(display.errorNameText, 'ThrownValue');
  assert.equal(display.errorMessageText, 'plain failure');
  assert.equal(display.errorStackText, 'No stack.');
  assert.equal(display.stdoutText, 'No stdout.');
  assert.equal(display.stderrText, 'No stderr.');
});
