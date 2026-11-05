// Runs only through the host's verified installed-app connection. No launch, native
// input, OS permission requests, network, clipboard, or developer checkout helpers.
import type { InstalledKit } from './lib/installed-release-host';
import { SmokeFault } from './lib/installed-release-contract';
import { createHash, randomUUID } from 'node:crypto';
import { existsSync, lstatSync, mkdtempSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const domains = {
  accessibility: 'Accessibility',
  screenRecording: 'ScreenRecording',
  postEvents: 'EventSynthesizing',
  inputMonitoring: 'InputMonitoring',
  microphone: 'Microphone',
} as const;
type Domain = keyof typeof domains;
const domainIds = Object.keys(domains) as Domain[];
const statusCodes = {
  Authorized: 'granted',
  Denied: 'denied',
  NotDetermined: 'notAsked',
  Unknown: 'unknown',
} as const;
const features = [
  'textExpansion',
  'dictation',
  'pasteIntoApps',
  'selectedText',
  'windowControl',
  'screenCapture',
] as const;
const humanSteps = [
  'mic-first-use-deny',
  'mic-allow-relaunch-and-sound',
  'ax-selected-text',
  'dictation-original-field-once',
  'optional-original-window-ocr',
  'target-specific-apple-events',
  'upgrade-preservation-and-repeat',
].map((id) => ({ id, status: 'requires-human-evidence' }));

type Row = Record<string, any>;
class ProofError extends Error {
  constructor(
    readonly code: string,
    readonly status: 'fail' | 'blocked' = 'fail',
  ) {
    super(code);
  }
}
function requireProof(
  condition: unknown,
  code: string,
  status: 'fail' | 'blocked' = 'fail',
): asserts condition {
  if (!condition) throw new ProofError(code, status);
}
function digest(bytes: Buffer | string) {
  return createHash('sha256').update(bytes).digest('hex');
}
function installedFile(root: string, relative: string) {
  const path = join(root, relative);
  requireProof(
    realpathSync(path) === join(realpathSync(root), relative) && lstatSync(path).isFile(),
    'unexpected-file-identity',
  );
  return path;
}
function oneRow(rows: unknown, key: string, value: string): Row {
  requireProof(Array.isArray(rows), 'missing-inventory');
  const matches = rows.filter((row) => row?.[key] === value);
  requireProof(matches.length === 1, 'incomplete-or-ambiguous-inventory');
  return matches[0];
}
async function observe<T>(read: () => Promise<T>, matches: (value: T) => boolean, code: string) {
  const deadline = performance.now() + 10_000;
  do {
    const value = await read();
    if (matches(value)) return value;
    await Bun.sleep(120);
  } while (performance.now() < deadline);
  throw new ProofError(code);
}

// Keep only known enum values and numeric/boolean facts. In particular, feature
// messages, device labels/errors, document paths, and query contents are not receipts.
function inventory(s: Row) {
  const p = s.permissions,
    d = s.dictation;
  requireProof(
    p?.source === 'permissions.service' &&
      p.schemaVersion === 1 &&
      p.bundleIdentifier === 'com.scriptkit.app',
    'not-installed-permission-service',
    'blocked',
  );
  requireProof(p.readinessScope === 'privacyPermissions', 'missing-permission-scope', 'blocked');
  const permissions = Object.fromEntries(
    domainIds.map((id) => {
      const r = oneRow(p.permissions, 'permission', id);
      requireProof(
        Object.values(statusCodes).includes(r.status) &&
          typeof r.grantLooksStuck === 'boolean' &&
          (r.requestedAtUnixMs === null ||
            (Number.isSafeInteger(r.requestedAtUnixMs) && r.requestedAtUnixMs > 0)),
        'invalid-domain-status',
      );
      const observation = r.status === 'granted' ? 'authorizedBySystem'
        : r.status === 'unknown' ? 'unavailable' : r.status === 'notAsked' ? 'notDeterminedBySystem'
        : id === 'microphone' ? 'deniedOrRestrictedBySystem' : 'notGrantedByPreflight';
      requireProof(r.observation === observation, 'permission-observation-disagrees');
      return [
        id,
        {
          status: r.status as string,
          observation,
          requestedAtUnixMs: r.requestedAtUnixMs as number | null,
          grantLooksStuck: r.grantLooksStuck,
        },
      ];
    }),
  );
  const readiness = Object.fromEntries(
    features.map((id) => {
      const r = oneRow(p.features, 'feature', id);
      requireProof(
        ['ready', 'blocked', 'askOnUse', 'degraded', 'unknown'].includes(r.readiness) &&
          Array.isArray(r.missing) &&
          r.missing.length <= domainIds.length &&
          r.missing.every((x: Domain) => domainIds.includes(x)),
        'invalid-feature-readiness',
      );
      return [id, { readiness: r.readiness as string, missing: r.missing as Domain[] }];
    }),
  );
  const mic = d?.setup?.microphone,
    devices = mic?.deviceSnapshot;
  requireProof(
    d?.source === 'runtime.dictation.automationState' &&
      d.passive === true &&
      d.isRecording === false &&
      mic?.source === 'passiveAVFoundationEnumeration' &&
      mic.noPermissionPrompt === true,
    'no-live-passive-dictation-state',
    'blocked',
  );
  const micStatus = permissions.microphone.status;
  requireProof(
    (
      { Granted: 'granted', Denied: 'denied', NotDetermined: 'notAsked', Unknown: 'unknown' } as Record<
        string,
        string
      >
    )[mic.permissionStatus] === micStatus,
    'microphone-preflights-disagree',
  );
  requireProof(
    devices?.rawDeviceIdsRedacted === true &&
      devices.deviceLabelsReturned === false &&
      typeof devices.available === 'boolean' &&
      Number.isSafeInteger(devices.deviceCount) &&
      devices.deviceCount >= 0 &&
      typeof d.setup.ready === 'boolean',
    'invalid-device-inventory',
  );
  // Kit doesn't enumerate devices without a grant. Zero then says nothing about hardware.
  let availability: string;
  if (micStatus !== 'granted') {
    requireProof(
      mic.status === 'permissionNeeded' && !d.setup.ready,
      'ungranted-microphone-reported-operational',
    );
    availability = 'not-enumerated-without-grant';
  } else if (!devices.available) {
    requireProof(
      mic.status === 'enumerationFailed' && !d.setup.ready,
      'failed-enumeration-reported-operational',
    );
    availability = 'unavailable';
  } else if (!devices.deviceCount) {
    requireProof(mic.status === 'noDevices' && !d.setup.ready, 'missing-device-reported-operational');
    availability = 'no-devices';
  } else {
    requireProof(['ready', 'savedDeviceMissing'].includes(mic.status), 'inconsistent-device-status');
    availability = mic.status === 'ready' ? 'available-not-tested' : 'saved-device-missing';
  }
  const axStatus = permissions.accessibility.status;
  if (axStatus !== 'granted')
    requireProof(
      readiness.selectedText.readiness === (axStatus === 'unknown' ? 'unknown' : 'blocked'),
      'selected-text-readiness-disagrees',
    );
  const dictationPermission = micStatus === 'denied' ? 'blocked' : micStatus === 'notAsked' ? 'askOnUse'
    : micStatus === 'unknown' || axStatus === 'unknown' ? 'unknown'
    : axStatus === 'granted' ? 'ready' : 'degraded';
  requireProof(readiness.dictation.readiness === dictationPermission, 'dictation-readiness-disagrees');
  const post = [axStatus, permissions.postEvents.status];
  requireProof(readiness.pasteIntoApps.readiness !== 'ready' || post.includes('granted'), 'paste-ready-without-confirmed-grant');
  const pastePermission = post.includes('granted') ? 'ready' : post.includes('unknown') ? 'unknown' : 'degraded';
  requireProof(readiness.pasteIntoApps.readiness === pastePermission, 'paste-readiness-disagrees');
  return { permissions, readiness, readinessScope: 'privacyPermissions', availability, dictationSetupReady: d.setup.ready, captureVerified: false };
}

// Don't treat the settings writer's acknowledgement as its oracle. The installed
// SDK must independently read back the new revision; all pre-existing bytes outside
// the scalar (or its newly inserted declaration and separator) must survive.
function retainedBytes(
  before: string,
  after: string,
  oldValue: number,
  newValue: number,
  wasDefault: boolean,
) {
  if (!wasDefault) {
    const match = new RegExp(`^(\\s*editorFontSize\\s*:\\s*)${oldValue}(?=[\\s,}])`, 'gm');
    return (
      [...before.matchAll(match)].length === 1 &&
      before.replace(match, (_, prefix) => prefix + newValue) === after
    );
  }
  const insertion = `\n  editorFontSize: ${newValue}\n`;
  if (after.split(insertion).length !== 2) return false;
  const stripped = after.replace(insertion, '');
  if (stripped === before) return true;
  let i = 0;
  while (i < before.length && stripped[i] === before[i]) i++;
  return stripped[i] === ',' && stripped.slice(i + 1) === before.slice(i);
}

// This is the ONLY generated executable. No checkout imports, native APIs or child
// processes here: the installed SDK owns the prompt; its installed settings module
// owns its version-matched Bun CLI. Do not run it from the verifier process.
function sdkScript(root: string, dir: string, pid: number, nonce: string, revision: string, readOnlyValue?: number) {
  const q = JSON.stringify;
  return `import { writeFileSync, renameSync, realpathSync } from 'node:fs';
import { configPath } from ${q(installedFile(root, 'sdk/config-document.ts'))};
import { config } from ${q(installedFile(root, 'sdk/kit-sdk-config.ts'))};
const dir = ${q(dir)}, nonce = ${q(nonce)}, expected = ${q(installedFile(root, 'config.ts'))};
function publish(name, value) { const file = dir + '/' + name; writeFileSync(file+'.tmp', JSON.stringify(value), {flag:'wx',mode:0o600}); renameSync(file+'.tmp',file); }
if (process.ppid !== ${pid} || !process.versions.bun || process.env.SDK_TEST_AUTOSUBMIT === '1' || realpathSync(configPath()) !== realpathSync(expected)) throw Error('wrong SDK execution context');
const old = await config.get('editorFontSize');
if (realpathSync(old.path) !== realpathSync(expected) || !old.exists || old.revision !== ${q(revision)} || !Number.isFinite(old.value)) throw Error('wrong installed settings');
publish('started.json',{nonce,ppid:process.ppid,bun:process.versions.bun});
const value = await form('<form><input name="address" type="email" value="incomplete"></form>');
publish('submitted.json',{nonce, address:value.address});
if (value.address !== nonce+'@example.invalid') throw Error('wrong form result');
${readOnlyValue !== undefined ? `const saved = await config.get('editorFontSize');
if (old.value !== ${q(readOnlyValue)} || saved.value !== old.value || saved.revision !== old.revision || saved.isDefault !== false || realpathSync(saved.path)!==realpathSync(expected)) throw Error('upgraded SDK did not retain the prior setting');
publish('saved.json',{nonce,value:saved.value,revision:saved.revision,readOnly:true});` : `const changed = await config.set('editorFontSize',old.value === 18 ? 19 : 18,old.revision);
const saved = await config.get('editorFontSize');
if (realpathSync(saved.path) !== realpathSync(expected) || saved.revision !== changed.revision || saved.value !== changed.value || saved.isDefault !== false) throw Error('failed readback');
publish('saved.json',{nonce,old:old.value,wasDefault:old.isDefault,value:saved.value,revision:saved.revision});`}
`;
}

export async function openSavedDayFromLauncher(date:string, ui:{state:()=>Promise<Row>;elements:()=>Promise<Row>;filter:(v:string)=>Promise<unknown>;enter:()=>Promise<unknown>}) {
  requireProof(/^\d{4}-\d{2}-\d{2}$/.test(date),'invalid-original-day');
  await ui.filter('Search Notes');
  await observe(ui.state,s=>s.inputValue==='Search Notes' && s.oneWindow?.enter?.id==='builtin/search-notes' && !s.oneWindow.enterArmed,'search-notes-not-selected');
  await ui.enter();
  await observe(ui.state,s=>s.oneWindow?.view==='notes','notes-list-not-open');
  await ui.filter(date);
  const weekday=new Intl.DateTimeFormat('en-US',{weekday:'long',timeZone:'UTC'}).format(new Date(date+'T12:00:00Z'));
  const allowed=new Set(['notes:open:Today','notes:open:'+date+' · '+weekday]);
  const ready=await observe(ui.state,s=>s.inputValue===date && allowed.has(s.oneWindow?.enter?.id) && !s.oneWindow.enterArmed,'original-day-not-ready');
  const enter=ready.oneWindow.enter.id, title=enter.slice('notes:open:'.length);
  // The public label is a process-private HMAC, not SHA-256(title). The reducer names the
  // Enter target; bind that to one selected measured row and verify the exact document ID
  // and original bytes in the editor before doing anything else.
  await observe(ui.elements,e=>{
    const found=e.elements.filter((r:Row)=>/^notes-row:\d+$/.test(r.semanticId) && r.content?.text?.rawContentReturned===false && r.content.text.charLength===[...title].length && r.content.text.byteLength===Buffer.byteLength(title));
    const chosen=found.filter((r:Row)=>r.selected===true && !r.actionDisabled);const fp=chosen[0]?.content.text.fingerprint;
    return chosen.length===1 && typeof fp==='string' && /^sha256:[0-9a-f]{64}$/.test(fp) && found.filter((r:Row)=>r.content.text.fingerprint===fp).length===1;
  },'original-day-not-selected');
  const still=await ui.state();requireProof(still.inputValue===date && still.oneWindow?.enter?.id===enter && !still.oneWindow.enterArmed,'original-day-selection-changed');
  await ui.enter();
}

// Script labels are authored content. The public element protocol gives them a process-private
// fingerprint, unlike the literal labels of built-ins. The signed starter bytes and the exact
// source-owned row ID bind the target; no private label or fingerprint needs to leave this Mac.
export function installedStarterObservation(rows: Row[], semanticId: string) {
  const exact = rows.filter(row => row.semanticId === semanticId);
  const row = exact.length === 1 ? exact[0] : undefined;
  const name = row?.content?.text;
  const measuredName = row?.text == null && row?.value == null &&
    name?.contentKind === 'externalContent' && name.rawContentReturned === false &&
    name.charLength === 'Hello World'.length && name.byteLength === Buffer.byteLength('Hello World') &&
    typeof name.fingerprint === 'string' && /^sha256:[0-9a-f]{64}$/.test(name.fingerprint);
  return { exactRows: exact.length, scriptRows: rows.filter(r => r.kind === 'script').length,
    measuredName, ready: measuredName && row?.role === 'row' && row.kind === 'script' &&
      row.selectable === true && !row.actionDisabled };
}

export async function runInstalledJourneys(kit: InstalledKit) {
  const checkpoints: Record<string, ReturnType<typeof inventory>> = {};
  const passed: string[] = [];
  const evidence: Row = {};
  const base = {
    schemaVersion: 1,
    evidenceSource: 'installed-app-stdin-and-owned-files',
    passed,
    evidence,
    checkpoints,
    statusScope: 'automated-installed-journeys-only',
    osDialogVisibility: 'not-observed-by-automation',
    humanSteps,
  };
  let stage = 'verify-main-window';
  try {
    const request = async (command: Row, expect: string) => {
      const requestId = randomUUID();
      const result = await kit.request({ ...command, requestId }, expect, 6000);
      requireProof(result?.type === expect && result.requestId === requestId, 'uncorrelated-response');
      return result;
    };
    const main = async () => {
      const result = await request({ type: 'listAutomationWindows' }, 'automationWindowListResult');
      requireProof(
        Array.isArray(result.windows) && result.windows.length === 1,
        'no-unique-main-window',
        'blocked',
      );
      const w = result.windows[0];
      requireProof(
        w.id === 'main' &&
          w.kind === 'main' &&
          w.pid === kit.pid &&
          Number.isSafeInteger(kit.pid) &&
          kit.pid > 0 &&
          Number.isSafeInteger(w.generation) &&
          w.generation > 0,
        'wrong-window-identity',
        'blocked',
      );
      return w;
    };
    const { generation } = await main();
    const target = { type: 'instance', id: 'main', generation };
    const identity = async () => {
      const w = await main();
      requireProof(w.generation === generation, 'window-lifetime-changed', 'blocked');
      return w;
    };
    const state = async () => {
      const s = await request({ type: 'getState', target }, 'stateResult');
      requireProof(
        s.permissions?.bundleIdentifier === 'com.scriptkit.app' &&
          s.promptType !== 'target_resolution_failed',
        'wrong-state-owner',
        'blocked',
      );
      evidence.lastAppState = {
        surface: ['none','scriptList','form','dayPage','permissionsWizard'].includes(s.promptType) ? s.promptType : 'other',
        visible: s.windowVisible === true, visibleChoices: Number.isSafeInteger(s.visibleChoiceCount) ? s.visibleChoiceCount : null,
        inputLength: typeof s.inputValue === 'string' ? s.inputValue.length : null,
      };
      return s;
    };
    const elements = async () => {
      const r = await request({ type: 'getElements', target, limit: 100 }, 'elementsResult');
      requireProof(Array.isArray(r.elements) && r.projectionQuality !== 'unsupported', 'missing-elements');
      evidence.lastProjection = {surface: ['scriptList','form-prompt','dayPage','permissionsWizard'].includes(r.semanticSurface) ? r.semanticSurface : 'other', elementCount:r.elements.length};
      return r;
    };
    const external = async (command: Row) => {
      await identity();
      const result = await request(command, 'externalCommandResult');
      requireProof(result.ok === true && result.command === command.type, 'external-command-refused');
    };
    const key = (key: string, modifiers: string[] = []) =>
      external({ type: 'simulateKey', target, key, modifiers });
    const batch = async (command: Row) => {
      const r = await request({ type: 'batch', target, commands: [command] }, 'batchResult');
      requireProof(
        r.success === true &&
          r.results?.length === 1 &&
          r.results[0]?.success === true &&
          r.results[0].command === command.type,
        'batch-step-refused',
      );
    };
    const checkpoint = async (name: string) => {
      await identity();
      const snapshot = inventory(await state());
      checkpoints[name] = snapshot;
      const initial = checkpoints.start;
      if (initial)
        for (const id of domainIds) {
          const prev = initial.permissions[id],
            now = snapshot.permissions[id];
          requireProof(
            now.requestedAtUnixMs === prev.requestedAtUnixMs && now.status === prev.status,
            'permission-baseline-changed',
          );
        }
      return snapshot;
    };
    const waitView = (prompt: string) =>
      observe(state, (s) => s.promptType === prompt && s.windowVisible === true, `missing-${prompt}-view`);
    const wizard = async (snap: ReturnType<typeof inventory>) => {
      const r = await elements();
      requireProof(
        r.semanticSurface === 'permissionsWizard' &&
          oneRow(r.elements, 'semanticId', 'panel:permissions-wizard').text === 'Set Up Permissions',
        'wrong-permission-view',
      );
      for (const id of domainIds) {
        const row = oneRow(r.elements, 'semanticId', `permission-row:${domains[id]}`);
        requireProof(
          row.role === 'permission' &&
            statusCodes[row.statusKind as keyof typeof statusCodes] === snap.permissions[id].status,
          'permission-row-disagrees',
        );
      }
    };
    const baseline = await checkpoint('start');
    evidence.owner = { pid: kit.pid, generation, id: target.id };
    evidence.localPermissions = {
      baselineNotGranted: domainIds.filter((id) =>
        ['denied', 'notAsked'].includes(baseline.permissions[id].status),
      ),
      baselineUnknown: domainIds.filter((id) => baseline.permissions[id].status === 'unknown'),
      alreadyGrantedNotTestedWithout: domainIds.filter((id) => baseline.permissions[id].status === 'granted'),
      scope: 'Only completed local actions; no proof of cross-app input, capture or OS dialog visibility.',
    };
    stage = 'launcher';
    const first = await state();
    evidence.startupKitPermissionView = first.promptType === 'permissionsWizard';
    if (evidence.startupKitPermissionView) {
      await wizard(baseline);
      await key('escape');
      // A directly opened wizard resets the list only AFTER the hide transition.
      await observe(state, (s) => s.promptType === 'none', 'startup-wizard-did-not-dismiss');
    }
    const launcher = async () => {
      if (!(await state()).windowVisible) {
        await identity();
        const ack = await request({ type: 'show' }, 'windowVisibilityAck');
        requireProof(ack.windowVisible === true, 'show-not-acknowledged');
      }
      await waitView('none');
      requireProof((await identity()).visible === true, 'launcher-not-visible');
    };
    await launcher();
    if (kit.freshStarter) {
      stage = 'fresh-starter';
      const { sha256, semanticId } = kit.freshStarter;
      const file = installedFile(kit.kitPath, 'plugins/main/scripts/hello-world.ts');
      const checkStarter = () => requireProof(digest(readFileSync(file)) === sha256, 'starter-bytes-disagree');
      checkStarter();
      await batch({ type: 'setInput', text: 'Hello World' });
      await observe(elements, r => {
        const observed = installedStarterObservation(r.elements, semanticId);
        evidence.starterRow = observed;
        return r.semanticSurface === 'scriptList' && observed.ready;
      }, 'starter-not-in-launcher');
      checkStarter();
      await batch({ type: 'selectBySemanticId', semanticId, submit: true });
      await observe(state, s => s.windowVisible === true && s.promptType === 'arg' &&
        s.placeholder === "What's your name?" && s.choiceCount === 3 && s.selectedValue === 'World', 'starter-name-prompt-not-observed');
      await key('enter');
      await waitView('div');
      await key('enter');
      await observe(state, s => s.promptType === 'none', 'starter-did-not-close');
      await launcher();
      evidence.freshStarter = { sha256, dispatchedFromLauncher: true, namePromptAnswered: true, displayOpenedAndDismissed: true };
      await checkpoint('freshStarter');
      passed.push(stage);
      stage = 'launcher';
    }
    await batch({ type: 'setInput', text: 'permissions' });
    const filtered = await observe(
      state,
      (s) =>
        s.promptType === 'none' &&
        s.inputValue === 'permissions' &&
        Number.isSafeInteger(s.visibleChoiceCount) &&
        s.visibleChoiceCount >= 2,
      'filter-not-applied',
    );
    const choices = new Map([
      ['Check Permissions', 'builtin/check-permissions'],
      ['Set Up Permissions', 'builtin/setup-permissions'],
    ]);
    const labels = [...choices.keys()];
    const findChoice = (r: Row, label: string) =>
      r.elements.find(
        (x: Row) =>
          x.role === 'row' &&
          x.kind === 'built-in' &&
          x.selectable === true &&
          !x.actionDisabled &&
          x.text === label &&
          x.semanticId === `main-list-row:v2:${digest(choices.get(label)!)}`,
      );
    const r = await observe(
      elements,
      (r) => r.semanticSurface === 'scriptList' && labels.every((label) => findChoice(r, label)),
      'missing-filtered-builtins',
    );
    for (const label of labels) {
      const semanticId = findChoice(r, label).semanticId;
      await batch({ type: 'selectBySemanticId', semanticId, submit: false });
      await observe(
        elements,
        (r) => findChoice(r, label)?.selected === true && findChoice(r, label)?.semanticId === semanticId,
        'launcher-selection-not-changed',
      );
    }
    evidence.launcher = {
      visible: true,
      filteredCount: filtered.visibleChoiceCount,
      selectedBuiltins: labels,
    };
    await checkpoint('launcher');
    passed.push(stage);

    stage = 'set-up-permissions';
    await batch({
      type: 'selectBySemanticId',
      semanticId: findChoice(r, labels[1]).semanticId,
      submit: true,
    });
    await waitView('permissionsWizard');
    await wizard(await checkpoint('wizard'));
    await key('escape'); // never submit a permission row
    await observe(state, (s) => s.promptType === 'none', 'wizard-did-not-dismiss');
    passed.push(stage);

    stage = 'accessibility-preflight';
    const beforeAx = await checkpoint('beforeAx');
    const ax = await request({ type: 'checkAccessibility' }, 'accessibilityStatus');
    requireProof(
      ax.granted === (beforeAx.permissions.accessibility.status === 'granted'),
      'ax-preflight-disagrees',
    );
    await checkpoint('afterAx');
    evidence.ax = { trusted: ax.granted, userDenialWitnessed: false };
    passed.push(stage);

    stage = 'notes-save-and-readback';
    if (kit.originalData) {
      const orig=kit.originalData;
      await launcher();
      await openSavedDayFromLauncher(orig.date,{state,elements,filter:text=>batch({type:'setInput',text}),enter:()=>key('enter')});
      const doc=await waitView('dayPage');
      const dayIds=(await elements()).elements.filter((r:Row)=>/^day:\d{4}-\d{2}-\d{2}$/.test(r.semanticId));
      requireProof(dayIds.length===1 && dayIds[0].semanticId==='day:'+orig.date && doc.dayPage?.dirty===false &&
        typeof doc.inputValue==='string' && digest(doc.inputValue)===orig.noteSha256 && digest(readFileSync(installedFile(kit.kitPath,'brain/days/'+orig.date+'.md')))===orig.noteSha256,'upgraded-editor-original-note-mismatch');
      evidence.notes={date:orig.date,sha256:orig.noteSha256,reopenedOriginal:true};
    } else {
    const openTodayFromLauncher = async () => {
      const title = 'Quick Note Capture', id = 'main-list-row:v2:'+digest('builtin/quick-capture');
      await launcher();
      await batch({ type: 'setInput', text: title });
      await observe(elements, r => {
        const matches = r.elements.filter((row: Row) => row.semanticId === id);
        return r.semanticSurface === 'scriptList' && matches.length === 1 && matches[0].text === title &&
          matches[0].kind === 'built-in' && matches[0].role === 'row' && matches[0].selectable === true && !matches[0].actionDisabled;
      }, 'quick-note-not-in-launcher');
      await batch({ type: 'selectBySemanticId', semanticId: id, submit: true });
      return waitView('dayPage');
    };
    const initialNote = await openTodayFromLauncher();
    // Both clean installs and upgrades are supported; never replace arbitrary prior text.
    const prior = initialNote.inputValue;
    requireProof(
      typeof prior === 'string' && /^(?:Installed smoke [a-f0-9-]{36}\n)*$/.test(prior),
      'non-test-day-page',
      'blocked',
    );
    const note = `${prior}Installed smoke ${randomUUID()}\n`;
    const noteView = await elements();
    const dayIds = noteView.elements.filter(
      (x: Row) => typeof x.semanticId === 'string' && /^day:\d{4}-\d{2}-\d{2}$/.test(x.semanticId),
    );
    requireProof(noteView.semanticSurface === 'dayPage' && dayIds.length === 1, 'wrong-day-document');
    const date = dayIds[0].semanticId.slice(4);
    const day = installedFile(kit.kitPath, `brain/days/${date}.md`);
    requireProof(readFileSync(day, 'utf8') === prior, 'day-page-not-bound-to-kit-home');
    await batch({ type: 'setInput', text: note });
    await observe(state, (s) => s.promptType === 'dayPage' && s.inputValue === note, 'note-edit-not-applied');
    await key('s', ['cmd']);
    requireProof(readFileSync(day, 'utf8') === note, 'note-not-on-disk');
    await key('escape');
    await observe(state, (s) => s.promptType === 'none', 'note-did-not-close');
    let reopened;
    if (kit.verifyStdinNoteReopen) {
      await launcher();
      await kit.send({ type: 'openNotes' });
      reopened = await waitView('dayPage');
    } else reopened = await openTodayFromLauncher();
    requireProof(reopened.inputValue === note && reopened.dayPage?.dirty === false, 'reopened-note-mismatch');
    evidence.notes = { date, sha256:digest(note), savedAndReopened:true, entry:'builtin/quick-capture',
      reopenedVia:kit.verifyStdinNoteReopen ? 'stdin/openNotes' : 'builtin/quick-capture' };
    }
    await checkpoint('notes');
    passed.push(stage);

    stage = 'installed-sdk-form';
    const dir = mkdtempSync(join(realpathSync(kit.scratchPath), 'installed-journey-'));
    const nonce = randomUUID();
    const configFile = installedFile(kit.kitPath, 'config.ts');
    // Reject invalid UTF-8 rather than lose original bytes to replacement characters.
    const utf8 = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
    const before = utf8.decode(readFileSync(configFile));
    for (const name of ['kit-sdk.ts', 'config-cli.ts', 'config-schema.ts'])
      installedFile(kit.kitPath, `sdk/${name}`);
    const script = join(dir, 'exercise.ts');
    if(kit.originalData)requireProof(digest(before)===kit.originalData.configSha256,'upgraded-config-original-bytes-mismatch');
    writeFileSync(script, sdkScript(kit.kitPath, dir, kit.pid, nonce, digest(before),kit.originalData?.editorFontSize), {
      flag: 'wx',
      mode: 0o600,
    });
    const sideEffect = async (name: string) => {
      const path = join(dir, name);
      await observe(async () => existsSync(path), Boolean, `missing-${name}`);
      return JSON.parse(readFileSync(installedFile(dir, name), 'utf8'));
    };
    await identity();
    await kit.send({ type: 'run', path: script });
    const form = await waitView('form'),
      start = await sideEffect('started.json');
    requireProof(
      start.nonce === nonce &&
        start.ppid === kit.pid &&
        typeof start.bun === 'string' &&
        /^\d{1,3}\.\d{1,3}\.\d{1,4}$/.test(start.bun),
      'wrong-sdk-child',
    );
    requireProof(typeof form.promptId === 'string' && form.promptId.length > 0, 'missing-form-id');
    const field = (r: Row) =>
      r.semanticSurface === 'form-prompt' &&
      r.elements.find((x: Row) => x.semanticId === 'input:form-address');
    const initial = await elements();
    requireProof(
      field(initial)?.focused === true &&
        field(initial)?.content?.value?.charLength === 'incomplete'.length &&
        !field(initial).statusKind,
      'wrong-sdk-form',
    );
    const enterForm = async () => {
      const current = await state();
      requireProof(
        current.promptType === 'form' && current.promptId === form.promptId && current.windowVisible,
        'form-owner-changed',
      );
      await key('enter');
    };
    await enterForm();
    await observe(
      elements,
      (r) =>
        field(r)?.statusKind === 'invalid' &&
        r.elements.some((x: Row) => x.role === 'alert' && x.semanticId === 'panel:form-error-0'),
      'invalid-form-not-rejected',
    );
    requireProof(
      (await state()).promptId === form.promptId && !existsSync(join(dir, 'submitted.json')),
      'invalid-form-dispatched',
    );
    const address = nonce + '@example.invalid';
    // Batch replies during an active SDK prompt go to the script's channel. Send
    // once and observe the actual field; never infer success from writing stdin.
    await kit.send({
      type: 'batch',
      target,
      requestId: randomUUID(),
      commands: [{ type: 'setInput', text: address }],
    });
    await observe(
      elements,
      (r) => field(r)?.content?.value?.charLength === address.length && !field(r).statusKind,
      'valid-form-edit-not-applied',
    );
    await enterForm();
    const submitted = await sideEffect('submitted.json');
    requireProof(submitted.nonce === nonce && submitted.address === address, 'form-result-not-delivered');
    evidence.sdk = { bun: start.bun, childOfApp: true, invalidRejected: true, validRoundTrip: true };
    passed.push(stage);

    stage = 'installed-settings';
    const saved = await sideEffect('saved.json');
    const after = utf8.decode(readFileSync(installedFile(kit.kitPath, 'config.ts')));
    if(kit.originalData) {
      requireProof(saved.nonce===nonce && saved.readOnly===true && saved.value===kit.originalData.editorFontSize &&
        before===after && digest(after)===kit.originalData.configSha256 && saved.revision===kit.originalData.configSha256,'upgraded-setting-changed');
      evidence.settings={key:'editorFontSize',value:saved.value,revision:saved.revision,revisionMatchedDisk:true,readOnly:true};
    } else {
    requireProof(
      saved.nonce === nonce &&
        Number.isFinite(saved.old) &&
        typeof saved.wasDefault === 'boolean' &&
        saved.value === (saved.old === 18 ? 19 : 18) &&
        digest(after) === saved.revision &&
        retainedBytes(before, after, saved.old, saved.value, saved.wasDefault),
      'settings-readback-or-neighbors-changed',
    );
    evidence.settings = {
      key: 'editorFontSize',
      old: saved.old,
      value: saved.value,
      revision: saved.revision,
      revisionMatchedDisk: true,
      neighboringBytesRetained: true,
    };
    }
    await checkpoint('sdkAndSettings');
    passed.push(stage);
    return { ...base, status: 'pass' as const };
  } catch (error) {
    if(error instanceof SmokeFault) evidence.transport = ['protocol_response_refused','protocol_response_timeout','installed_instance_changed','unbound_ui_target','unreviewed_command','incomplete_protocol_write','unexpected_protocol_spool'].includes(error.code) ? error.code : 'other_host_refusal';
    // Untrusted transport, SDK and filesystem errors can contain paths, raw values
    // or complete responses. Never include those strings in a published receipt.
    return {
      ...base,
      status: error instanceof ProofError || error instanceof SmokeFault ? error.status : ('blocked' as const),
      stage,
      reason: error instanceof ProofError ? error.code : 'transport-or-owned-file-unavailable',
    };
  }
}
