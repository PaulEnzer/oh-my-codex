import { describe, it, beforeEach, afterEach } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

// Import from built output after tsc: dist/notifications/tmux-detector.js
import { isTmuxAvailable } from '../../notifications/tmux-detector.js';
import { createHookPluginSdk } from '../../hooks/extensibility/sdk.js';

function baseEvent() {
  return {
    schema_version: '1',
    event: 'turn-complete',
    timestamp: new Date().toISOString(),
    source: 'native',
    context: {},
  } as const;
}

describe('compat: tmux opt-in env gate', () => {
  const originalEnv = { ...process.env } as NodeJS.ProcessEnv;
  const originalCwd = process.cwd();
  let wd: string;

  beforeEach(async () => {
    wd = await mkdtemp(join(tmpdir(), 'omx-compat-tmux-optin-'));
    process.chdir(wd);
    delete process.env.OMX_COMPAT_TMUX;
    delete process.env.OMX_NO_TMUX;
    delete process.env.TMUX;
    delete process.env.TMUX_PANE;
  });

  afterEach(async () => {
    process.chdir(originalCwd);
    process.env = { ...originalEnv } as NodeJS.ProcessEnv;
    await rm(wd, { recursive: true, force: true });
  });

  it('disables tmux paths by default when OMX_COMPAT_TMUX is not set', async () => {
    // Detector path should be false without opt-in
    assert.equal(isTmuxAvailable(), false);

    // SDK path should report no_backend without opt-in
    const sdk = createHookPluginSdk({ cwd: wd, pluginName: 'demo', event: baseEvent(), sideEffectsEnabled: true });
    const res = await sdk.tmux.sendKeys({ text: 'echo hi', submit: false });
    assert.equal(res.ok, false);
    assert.equal(res.reason, 'no_backend');
  });
});

