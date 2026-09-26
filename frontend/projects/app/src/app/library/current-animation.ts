import { computed, Injectable, signal } from '@angular/core';
import type { AnimationSummary } from './library-types';

/** What the editor holds: work never saved, or a saved animation at the version it last read. */
export type CurrentAnimationState =
  | { readonly kind: 'unsaved' }
  | {
      readonly kind: 'saved';
      readonly id: string;
      readonly projectId: string;
      readonly version: number;
    };

const UNSAVED: CurrentAnimationState = { kind: 'unsaved' };

/**
 * Whether the editor holds a saved animation — its id, version and project — or unsaved work
 * (accounts.md, H8). It lives for the whole page, beside the engine: saving reads the version to
 * name, and every write that changes it (a save, a rename) records the new one here.
 */
@Injectable({ providedIn: 'root' })
export class CurrentAnimation {
  private readonly stateSignal = signal<CurrentAnimationState>(UNSAVED);

  readonly state = this.stateSignal.asReadonly();
  /** The saved animation's id, or `null` for unsaved work. */
  readonly id = computed(() => {
    const state = this.stateSignal();
    return state.kind === 'saved' ? state.id : null;
  });

  /** The editor now holds `animation`, as saved or read at its version. */
  setSaved(animation: AnimationSummary): void {
    const { id, projectId, version } = animation;
    this.stateSignal.set({ kind: 'saved', id, projectId, version });
  }

  /** The editor now holds work never saved: a new animation, or one deleted from the library. */
  setUnsaved(): void {
    this.stateSignal.set(UNSAVED);
  }

  /** Whether the editor holds the saved animation `id`. */
  holds(id: string): boolean {
    return this.id() === id;
  }
}
