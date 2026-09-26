import type { CurrentAnimationState } from '../../current-animation';
import type { SaveStatus } from '../save-flow';

/**
 * The one save state the document bar shows (design-system/docs/patterns.md, "Saving and unsaved
 * work"): work never saved, stored work changed since, a write under way, stored work as it was
 * written, or a write that failed.
 */
export type SaveState = 'not-saved' | 'unsaved-changes' | 'saving' | 'saved' | 'failed';

/** What the save state is derived from: `SaveFlow`, `EngineStore` and `CurrentAnimation`. */
export interface SaveStateSources {
  readonly status: SaveStatus;
  readonly hasDocument: boolean;
  readonly hasUnsavedWork: boolean;
  readonly current: CurrentAnimationState['kind'];
}

/**
 * The save state of the editor's work, or `null` with no document. "Saved" is only claimed for
 * an animation the library holds whose work has not changed since it was written or read — never
 * while a write runs or after one failed; saving is explicit, so nothing here says "autosaved".
 */
export function saveState(sources: SaveStateSources): SaveState | null {
  if (!sources.hasDocument) return null;
  if (sources.status === 'saving') return 'saving';
  if (sources.status === 'failed') return 'failed';
  if (sources.current === 'unsaved') return 'not-saved';
  return sources.hasUnsavedWork ? 'unsaved-changes' : 'saved';
}
