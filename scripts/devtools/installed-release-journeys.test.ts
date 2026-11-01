import { describe, expect, it } from 'bun:test';
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { createHash } from 'node:crypto';
import { runInstalledJourneys } from './installed-release-journeys';

type Obj = Record<string, any>;
const domainRows = [
  ['accessibility', 'Accessibility'],
  ['screenRecording', 'ScreenRecording'],
  ['postEvents', 'EventSynthesizing'],
  ['inputMonitoring', 'InputMonitoring'],
  ['microphone', 'Microphone'],
];
const code: Record<string, string> = {
  denied: 'Denied',
  granted: 'Authorized',
  unknown: 'Unknown',
  notAsked: 'NotDetermined',
};
const commandRows = ['Check Permissions', 'Set Up Permissions'].map((text, i) => ({
  text,
  role: 'row',
  kind: 'built-in',
  selectable: true,
  semanticId:
    'main-list-row:v2:' +
    createHash('sha256')
      .update(i ? 'builtin/setup-permissions' : 'builtin/check-permissions')
      .digest('hex'),
}));

// CONTROLLED TRANSPORT FIXTURE, not a .app, real consent or a real device. Model
// only the allowed wire effects. The generated script actually runs in an isolated
// Bun child with a stubbed form() and copies of the production settings modules.
async function exercise(
  options: {
    fault?: string;
    mic?: string;
    ax?: string;
    deviceCount?: number;
    enumerationFailed?: boolean;
    defaultSetting?: boolean;
  } = {},
) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'installed-journey-test-')));
  const kitPath = join(root, '.scriptkit'),
    scratchPath = join(root, 'scratch'),
    sdk = join(kitPath, 'sdk');
  mkdirSync(sdk, { recursive: true });
  mkdirSync(scratchPath);
  const configFile = join(kitPath, 'config.ts');
  writeFileSync(
    configFile,
    'export default {\n  // PRIVATE_FIXTURE_COMMENT\n' +
      (options.defaultSetting ? '' : '  editorFontSize: 16, // retain\n') +
      '  hotkey: {modifiers:["meta"],key:"Semicolon"},\n} satisfies Config;\n',
  );
  for (const name of ['kit-sdk-config.ts', 'config-cli.ts', 'config-schema.ts', 'config-document.ts'])
    copyFileSync(join(import.meta.dir, '..', name), join(sdk, name));
  if (options.fault === 'neighbors') {
    // A corrupt writer AND reader can agree on a new digest; independent old
    // bytes must still catch lost neighboring comments.
    const helper = join(sdk, 'kit-sdk-config.ts');
    copyFileSync(helper, join(sdk, 'fixture-real-config.ts'));
    writeFileSync(
      helper,
      `import {config as actual} from './fixture-real-config.ts';
      import {readFileSync, writeFileSync} from 'node:fs';
      export const config = {...actual, async set(...args) {
        const changed = await actual.set(...args), {path} = await actual.get(args[0]);
        writeFileSync(path,readFileSync(path,'utf8').replace('PRIVATE_FIXTURE_COMMENT','lost comment'));
        return {...changed, revision:(await actual.get(args[0])).revision};
      }};`,
    );
  }
  const preload = join(sdk, 'kit-sdk.ts');
  writeFileSync(
    preload,
    `globalThis.form = async html => { if (!html.includes('type="email"')) throw Error('unexpected fixture'); return {address:await Bun.stdin.text()}; };`,
  );
  const date = '2030-01-03',
    day = join(kitPath, `brain/days/${date}.md`);
  mkdirSync(dirname(day), { recursive: true });
  writeFileSync(day, '');
  const fault = options.fault,
    generation = 7,
    pid = process.pid;
  const mic = options.mic ?? 'notAsked',
    count = options.deviceCount ?? 0;
  const p = Object.fromEntries(
    domainRows.map(([id]) => [
      id,
      id === 'microphone'
        ? mic
        : mic === 'unknown' && ['accessibility', 'postEvents'].includes(id)
          ? 'unknown'
          : 'denied',
    ]),
  );
  if (options.ax) p.accessibility = options.ax;
  let windowQueries = 0,
    view = 'permissionsWizard',
    visible = true,
    input = '',
    selected = '',
    invalid = false,
    field = 'incomplete',
    noteOpens = 0,
    appPid = pid;
  let child: ReturnType<typeof Bun.spawn> | undefined;
  const seen: string[] = [];
  if (fault === 'pid') appPid++;
  function state() {
    const deviceStatus =
      mic !== 'granted'
        ? 'permissionNeeded'
        : options.enumerationFailed
          ? 'enumerationFailed'
          : count === 0
            ? 'noDevices'
            : 'ready';
    return {
      windowVisible: visible,
      promptType: view,
      inputValue: input,
      visibleChoiceCount: input === 'permissions' ? 2 : 80,
      ...(view === 'form' ? { promptId: 'fixture-form-id' } : {}),
      dayPage: { dirty: false },
      permissions: {
        source: 'permissions.service',
        schemaVersion: 1,
        bundleIdentifier: fault === 'bundle' ? 'wrong.bundle' : 'com.scriptkit.app',
        readinessScope: fault === 'scope' ? 'hardware-tested' : 'privacyPermissions',
        permissions: domainRows.map(([permission]) => ({
          permission,
          status: p[permission],
          observation: fault === 'denial' ? 'personClickedDeny' : p[permission] === 'granted' ? 'authorizedBySystem'
            : p[permission] === 'unknown' ? 'unavailable' : p[permission] === 'notAsked' ? 'notDeterminedBySystem'
            : permission === 'microphone' ? 'deniedOrRestrictedBySystem' : 'notGrantedByPreflight',
          requestedAtUnixMs: null,
          grantLooksStuck: false,
        })),
        features: [
          'textExpansion',
          'dictation',
          'pasteIntoApps',
          'selectedText',
          'windowControl',
          'screenCapture',
        ].map((feature) => ({
          feature,
          readiness:
            feature === 'dictation'
              ? mic === 'denied'
                ? 'blocked'
                : mic === 'notAsked'
                  ? 'askOnUse'
                  : fault === 'false-dictation' ? 'ready'
                    : mic === 'unknown' || p.accessibility === 'unknown' ? 'unknown'
                    : p.accessibility === 'granted' ? 'ready' : 'degraded'
              : feature === 'selectedText' ? (p.accessibility === 'unknown' ? 'unknown' : p.accessibility === 'granted' ? 'ready' : 'blocked')
              : feature === 'pasteIntoApps' ? (fault === 'false-paste' || p.accessibility === 'granted' ? 'ready' : p.accessibility === 'unknown' ? 'unknown' : 'degraded')
              : 'blocked',
          missing: [],
          message: 'PRIVATE_FIXTURE_MESSAGE',
        })),
      },
      dictation: {
        source: 'runtime.dictation.automationState',
        passive: true,
        isRecording: false,
        setup: {
          ready: deviceStatus === 'ready',
          microphone: {
            permissionStatus: mic === 'granted' ? 'Granted' : code[mic],
            status: deviceStatus,
            source: fault === 'evaluator' ? 'ownedFixture' : 'passiveAVFoundationEnumeration',
            noPermissionPrompt: true,
            deviceSnapshot: {
              available: !options.enumerationFailed,
              deviceCount: count,
              rawDeviceIdsRedacted: true,
              deviceLabelsReturned: false,
              error: 'PRIVATE_FIXTURE_ERROR',
            },
          },
        },
      },
    };
  }
  function elements() {
    const rows =
      view === 'permissionsWizard'
        ? [
            { semanticId: 'panel:permissions-wizard', text: 'Set Up Permissions' },
            ...domainRows.map(([id, kind]) => ({
              role: 'permission',
              semanticId: 'permission-row:' + kind,
              statusKind: fault === 'wizard' && id === 'accessibility' ? 'Authorized' : code[p[id]],
            })),
          ]
        : view === 'dayPage'
          ? [{ semanticId: `day:${date}` }]
          : view === 'form'
            ? [
                {
                  semanticId: 'input:form-address',
                  focused: true,
                  content: { value: { charLength: field.length } },
                  ...(invalid ? { statusKind: 'invalid' } : {}),
                },
                ...(invalid ? [{ role: 'alert', semanticId: 'panel:form-error-0' }] : []),
              ]
            : commandRows.map((x) => ({ ...x, selected: selected === x.semanticId }));
    return {
      projectionQuality: 'complete',
      semanticSurface: view === 'none' ? 'scriptList' : view === 'form' ? 'form-prompt' : view,
      elements: rows,
    };
  }
  const kit = {
    pid,
    kitPath,
    scratchPath,
    async request(c: Obj) {
      seen.push(c.type);
      if (fault === 'dead') throw new Error('PRIVATE_FIXTURE_TRANSPORT');
      if (['getState', 'getElements', 'batch', 'simulateKey'].includes(c.type))
        expect(c.target).toEqual({ type: 'instance', id: 'main', generation });
      let res: Obj;
      switch (c.type) {
        case 'listAutomationWindows':
          ++windowQueries;
          res = {
            type: 'automationWindowListResult',
            windows: [
              {
                id: 'main',
                kind: 'main',
                visible,
                pid: appPid,
                generation: generation + (fault === 'generation' && windowQueries > 1 ? 1 : 0),
              },
            ],
          };
          break;
        case 'show':
          visible=true;
          res={type:'windowVisibilityAck',windowVisible:true};
          break;
        case 'getState':
          res = { type: 'stateResult', ...state() };
          break;
        case 'getElements':
          res = { type: 'elementsResult', ...elements() };
          break;
        case 'batch': {
          const [cmd] = c.commands;
          if (cmd.type === 'setInput') input = cmd.text;
          else if (cmd.type === 'selectBySemanticId') {
            expect(view).toBe('none');
            expect(commandRows.some((r) => r.semanticId === cmd.semanticId)).toBe(true);
            selected = cmd.semanticId;
            if (cmd.submit) {
              expect(selected).toBe(commandRows[1].semanticId);
              view = 'permissionsWizard';
            }
          } else throw Error('prohibited batch');
          res = {
            type: 'batchResult',
            success: fault !== 'batch',
            results: [{ success: true, command: cmd.type }],
          };
          break;
        }
        case 'simulateKey': {
          if (c.key === 'escape') { visible = view === 'permissionsWizard' && !input ? false : visible; view = 'none'; }
          else if (c.key === 's' && view === 'dayPage') {
            if (fault !== 'disk') writeFileSync(day, input);
          } else if (c.key === 'enter' && view === 'form') {
            if (field === 'incomplete') invalid = true;
            else {
              const stream = child!.stdin as Bun.FileSink;
              stream.write(fault === 'form-value' ? 'wrong@example.invalid' : field);
              stream.end();
              await child!.exited;
              view = 'none';
            }
          } else throw Error('prohibited key (no permission row activation)');
          res = { type: 'externalCommandResult', ok: true, command: c.type };
          break;
        }
        case 'checkAccessibility':
          if (fault === 'grant-changed') p.screenRecording = 'granted';
          res = { type: 'accessibilityStatus', granted: (p.accessibility === 'granted') !== (fault === 'ax') };
          break;
        default:
          throw Error('unexpected request');
      }
      return { ...res, requestId: fault === 'stale' ? 'old-request-id' : c.requestId };
    },
    async send(c: Obj) {
      seen.push(c.type);
      if (c.type === 'openNotes') {
        view = 'dayPage';
        ++noteOpens;
        input = fault === 'note' && noteOpens > 1 ? '' : readFileSync(day, 'utf8');
      } else if (c.type === 'run') {
        expect(realpathSync(c.path).startsWith(scratchPath + '/installed-journey-')).toBe(true);
        child = Bun.spawn(
          [
            process.execPath,
            '--config=/dev/null',
            '--no-install',
            '--no-env-file',
            '--preload',
            preload,
            c.path,
          ],
          {
            cwd: scratchPath,
            env: { HOME: root, SK_PATH: kitPath, PATH: '/usr/bin:/bin', SCRIPT_KIT_NONINTERACTIVE: '1' },
            stdin: 'pipe',
            stdout: 'ignore',
            stderr: 'ignore',
          },
        );
        view = 'form';
      } else if (c.type === 'batch' && view === 'form' && c.commands[0].type === 'setInput') {
        field = c.commands[0].text;
        invalid = false;
      } else throw Error('unexpected send');
    },
  };
  try {
    const result = await runInstalledJourneys(kit);
    expect(JSON.stringify(result)).not.toContain('PRIVATE_FIXTURE_');
    return { result, seen };
  } finally {
    if (child && child.exitCode === null) {
      child.kill();
      await child.exited;
    }
    rmSync(root, { recursive: true, force: true });
  }
}

describe('installed release journeys — controlled transport fixtures only', () => {
  for (const [mic, availability, deviceCount, enumerationFailed] of [
    ['notAsked', 'not-enumerated-without-grant', 0, false],
    ['denied', 'not-enumerated-without-grant', 0, false],
    ['unknown', 'not-enumerated-without-grant', 0, false],
    ['granted', 'no-devices', 0, false],
    ['granted', 'unavailable', 0, true],
    ['granted', 'available-not-tested', 1, false],
  ] as const)
    it(`${mic}, ${availability}: local actions and independent persistence roundtrips`, async () => {
      const { result, seen } = await exercise({
        mic,
        deviceCount,
        enumerationFailed,
        defaultSetting: mic === 'notAsked',
      });
      expect(result).toMatchObject({
        status: 'pass',
        passed: [
          'launcher',
          'set-up-permissions',
          'accessibility-preflight',
          'notes-save-and-readback',
          'installed-sdk-form',
          'installed-settings',
        ],
        osDialogVisibility: 'not-observed-by-automation',
        evidence: { startupKitPermissionView: true, settings: { key: 'editorFontSize', value: 18 } },
        checkpoints: { start: { availability, permissions: { microphone: { status: mic } } } },
      });
      expect(result.checkpoints.start.readinessScope).toBe('privacyPermissions');
      if (mic === 'unknown') expect(result.checkpoints.start.permissions.microphone.observation).toBe('unavailable');
      expect(result.humanSteps).toHaveLength(7);
      expect(seen).toContain('run');
    });
  it('does not equate ready permissions with a working microphone', async () => {
    const { result } = await exercise({ mic: 'granted', ax: 'granted', deviceCount: 0 });
    expect(result.checkpoints.start).toMatchObject({ readiness: { dictation: { readiness: 'ready' } },
      availability: 'no-devices', dictationSetupReady: false, captureVerified: false });
    expect(result.status).toBe('pass');
  });
  for (const [fault, reason] of [['false-paste', 'paste-ready-without-confirmed-grant'], ['false-dictation', 'dictation-readiness-disagrees']])
    it(`refuses ${fault} with unknown access`, async () => {
      const { result, seen } = await exercise({ fault, mic: 'unknown' });
      expect(result).toMatchObject({ status: 'fail', reason });
      expect(seen).not.toContain('simulateKey');
    });
  for (const [fault, reason] of [
    ['denial', 'permission-observation-disagrees'],
    ['scope', 'missing-permission-scope'],
    ['dead', 'transport-or-owned-file-unavailable'],
    ['pid', 'wrong-window-identity'],
    ['bundle', 'wrong-state-owner'],
    ['evaluator', 'no-live-passive-dictation-state'],
    ['generation', 'window-lifetime-changed'],
    ['stale', 'uncorrelated-response'],
    ['wizard', 'permission-row-disagrees'],
    ['batch', 'batch-step-refused'],
    ['ax', 'ax-preflight-disagrees'],
    ['grant-changed', 'permission-baseline-changed'],
    ['disk', 'note-not-on-disk'],
    ['note', 'reopened-note-mismatch'],
    ['form-value', 'form-result-not-delivered'],
    ['neighbors', 'settings-readback-or-neighbors-changed'],
  ])
    it(`refuses ${fault} instead of passing on command acknowledgements`, async () => {
      const { result, seen } = await exercise({ fault });
      expect(result.status).not.toBe('pass');
      expect(result).toMatchObject({ reason });
      if (!['neighbors', 'form-value'].includes(fault)) expect(seen).not.toContain('run');
      if (['dead', 'pid', 'bundle', 'generation', 'stale', 'evaluator'].includes(fault))
        expect(seen).not.toContain('simulateKey');
    });
});
