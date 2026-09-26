import { inject, Injectable } from '@angular/core';
import { Router } from '@angular/router';
import { EDITOR_ENGINE } from '../../engine/editor-engine';
import { EngineStore } from '../../engine/engine-store';
import { CurrentAnimation, type CurrentAnimationState } from '../current-animation';
import { LIBRARY_STORE } from '../library-store';
import { toLibraryFailure } from '../library-types';
import { AnimationLoader } from '../open/animation-loader';
import { SaveConflict } from '../save/save-conflict';
import { SaveFlow } from '../save/save-flow';
import { SavePrompts } from '../save/save-prompts';
import { LibraryChangedPrompt } from './library-changed-prompt';
import { concernsAnimation, type LibraryChange, LibraryChanges } from './library-changes';

type SavedAnimation = Extract<CurrentAnimationState, { kind: 'saved' }>;

const EDITOR_PATH = '/editor';
const ANIMATION_NOT_FOUND = 'library.animation_not_found';

/**
 * Keeps the open animation in step with the library (desktop.md, T3). When its file holds another
 * version than the editor's — an agent or a person changed it —, the editor reloads it if it
 * holds no unsaved work; otherwise the person chooses: reload, keep mine — the next save then
 * meets H8's conflict dialog —, or save mine as a copy. The app's own saves leave the editor's
 * version on disk, and change nothing; a deleted file is left to the next save.
 */
@Injectable({ providedIn: 'root' })
export class OpenAnimationRefresh {
  private readonly changes = inject(LibraryChanges);
  private readonly current = inject(CurrentAnimation);
  private readonly engine = inject(EngineStore);
  private readonly editorEngine = inject(EDITOR_ENGINE);
  private readonly store = inject(LIBRARY_STORE);
  private readonly loader = inject(AnimationLoader);
  private readonly prompt = inject(LibraryChangedPrompt);
  private readonly conflict = inject(SaveConflict);
  private readonly saving = inject(SaveFlow);
  private readonly savePrompts = inject(SavePrompts);
  private readonly router = inject(Router);
  private isRefreshing = false;

  /** Follows the library's changes for the rest of the app's life. */
  follow(): void {
    this.changes.changes.subscribe((change) => void this.onChange(change));
  }

  /** What `change` does to the open animation; a failure is shown as saving shows its own. */
  async onChange(change: LibraryChange): Promise<void> {
    const state = this.current.state();
    if (state.kind !== 'saved' || !concernsAnimation(change, state.id)) return;
    if (this.isRefreshing || this.saving.status() === 'saving') return;
    this.isRefreshing = true;
    try {
      await this.refresh(state);
    } catch (error: unknown) {
      void this.savePrompts.showFailure(toLibraryFailure(error));
    } finally {
      this.isRefreshing = false;
    }
  }

  private async refresh(animation: SavedAnimation): Promise<void> {
    if (!(await this.changedOnDisk(animation))) return;
    if (!this.engine.hasUnsavedWork()) {
      await this.loader.load(animation.id);
      return;
    }
    const choice = await this.prompt.ask();
    if (choice === 'reload') await this.loader.load(animation.id);
    if (choice === 'copy') await this.saveCopy(animation);
  }

  /** Whether the file holds another version than the one the editor holds now. */
  private async changedOnDisk(animation: SavedAnimation): Promise<boolean> {
    try {
      const { summary } = await this.store.openDocument(animation.id);
      const state = this.current.state();
      return (
        state.kind === 'saved' && state.id === animation.id && state.version !== summary.version
      );
    } catch (error: unknown) {
      const failure = toLibraryFailure(error);
      if (failure.code === ANIMATION_NOT_FOUND) return false;
      throw failure;
    }
  }

  /** Saves the editor's work as a new animation beside the changed one, and follows it. */
  private async saveCopy(animation: SavedAnimation): Promise<void> {
    const document = await this.editorEngine.serialize();
    const { id, projectId } = animation;
    const copy = await this.conflict.resolve({ id, projectId, document }, 'copy');
    if (copy === null) return;
    this.current.setSaved(copy);
    await this.engine.markSaved();
    await this.router.navigateByUrl(`${EDITOR_PATH}/${copy.id}`, { replaceUrl: true });
  }
}
