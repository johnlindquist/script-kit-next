// Before (never after) the measured setInput, observe the cold launcher's real
// workspace revision. Publishing it advances target identity even when the rules are stock.
import type { Json } from '../devtools/driver.ts';
export function coldRootReady(state: Json): boolean {
  const search = state.rootFileSearch;
  if (state.windowVisible !== false || state.inputValue !== '' || state.oneWindow?.view !== null ||
      state.targetIdentity?.appViewVariant !== 'ScriptList' || search?.query !== '' || search.generation !== 0 ||
      search.providerLoading !== false || search.loading !== false || search.visibleResultCount !== 0 ||
      search.cacheEntryCount !== 0 || search.cacheResultCount !== 0) throw new Error('untouched_cold_root_fixture_required');
  const p = state.oneWindow.personalization;
  if (p?.scope !== 'runningWindow' || p.error !== null || p.pendingWrite !== false ||
      !Array.isArray(p.activeViewIds) || p.activeViewIds.length) throw new Error('cold_root_personalization_unavailable');
  if (p.workspace === null && p.appliedRevision === null) return false;
  if (typeof p.workspace !== 'string' || !/^[a-f0-9]{64}$/.test(p.workspace) || p.appliedRevision !== 'stock') {
    throw new Error('cold_root_workspace_not_stock');
  }
  return true;
}
