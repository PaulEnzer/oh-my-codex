import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { readModeState, startMode, updateModeState } from '../../modes/base.js';
import { ensureLinkedRalphModeState } from '../../team/linked-ralph-bridge.js';

describe('ensureLinkedRalphModeState', () => {
  it('creates a linked Ralph mode state when none exists', async () => {
    const wd = await mkdtemp(join(tmpdir(), 'omx-team-linked-ralph-'));
    try {
      await ensureLinkedRalphModeState('ship it', 'ship-it', wd);
      const state = await readModeState('ralph', wd);
      assert.ok(state, 'expected ralph state to be created');
      assert.equal(state?.active, true);
      assert.equal(state?.current_phase, 'executing');
      assert.equal(state?.task_description, 'ship it');
      assert.equal(state?.linked_team, true);
      assert.equal(state?.team_name, 'ship-it');
      assert.equal(typeof state?.linked_team_started_at, 'string');
    } finally {
      await rm(wd, { recursive: true, force: true });
    }
  });

  it('updates the active Ralph state to reflect linked team monitoring', async () => {
    const wd = await mkdtemp(join(tmpdir(), 'omx-team-linked-ralph-update-'));
    try {
      await startMode('ralph', 'old task', 50, wd);
      await updateModeState('ralph', {
        current_phase: 'verifying',
      }, wd);

      await ensureLinkedRalphModeState('new linked task', 'alpha', wd);

      const state = await readModeState('ralph', wd);
      assert.ok(state, 'expected ralph state to exist');
      assert.equal(state?.active, true);
      assert.equal(state?.current_phase, 'executing');
      assert.equal(state?.task_description, 'new linked task');
      assert.equal(state?.linked_team, true);
      assert.equal(state?.team_name, 'alpha');
    } finally {
      await rm(wd, { recursive: true, force: true });
    }
  });
});
