import { inject, Injectable } from '@angular/core';
import { EDITOR_ENGINE } from '../../engine/editor-engine';
import { CurrentAnimation } from '../current-animation';
import { LIBRARY_STORE } from '../library-store';
import { toLibraryFailure } from '../library-types';

/**
 * Puts a saved animation in the editor: reads its document from the library, opens it in the
 * engine, and records it as the current animation. Rejects with a `LibraryFailure` — the
 * library's, or the engine's refusal of the document — leaving the editor's work untouched.
 */
@Injectable({ providedIn: 'root' })
export class AnimationLoader {
  private readonly store = inject(LIBRARY_STORE);
  private readonly engine = inject(EDITOR_ENGINE);
  private readonly current = inject(CurrentAnimation);

  async load(id: string): Promise<void> {
    try {
      const { summary, document } = await this.store.openDocument(id);
      await this.engine.open(document);
      this.current.setSaved(summary);
    } catch (error: unknown) {
      throw toLibraryFailure(error);
    }
  }
}
