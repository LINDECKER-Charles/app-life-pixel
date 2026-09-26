import { inject, Injectable, signal } from '@angular/core';
import { NavigationEnd, Router } from '@angular/router';
import { filter } from 'rxjs';
import { EDITOR_ENGINE } from '../../engine/editor-engine';
import { EngineStore } from '../../engine/engine-store';
import { CurrentAnimation } from '../current-animation';
import { LIBRARY_ACCESS } from '../library-access';
import { LIBRARY_STORE } from '../library-store';
import { toLibraryFailure, type AnimationSummary, type LibraryFailure } from '../library-types';
import { SaveConflict } from './save-conflict';
import { SavePrompts, type StorageUsage } from './save-prompts';

/** Where saving stands: nothing yet, under way, or done — until the next edit. */
export type SaveStatus = 'idle' | 'saving' | 'saved';

const EDITOR_PATH = '/editor';
const SIGN_IN_PATHS = { 'sign-in': '/sign-in', 'sign-up': '/sign-up' } as const;
const UNAUTHENTICATED = 'auth.unauthenticated';
const VERSION_CONFLICT = 'document.version_conflict';
const STORAGE_EXCEEDED = 'quota.storage_exceeded';
const ANIMATION_NOT_FOUND = 'library.animation_not_found';

/** A number of `params`, or `undefined`: a problem's params are the server's to fill. */
function numberParam(failure: LibraryFailure, name: string): number | undefined {
  const value = failure.params[name];
  return typeof value === 'number' ? value : undefined;
}

/**
 * Saving the editor's work (accounts.md, H8), on Ctrl/⌘ S and the editor's Save button: a
 * visitor is asked to sign in or up, then saving goes on once back in the editor; unsaved work
 * asks for a project, then creates the animation; a saved one is saved under its version, a
 * conflict or the quota asking what to do. Success marks the engine saved; a failure keeps the
 * work and says why.
 */
@Injectable({ providedIn: 'root' })
export class SaveFlow {
  private readonly store = inject(LIBRARY_STORE);
  private readonly access = inject(LIBRARY_ACCESS);
  private readonly engine = inject(EngineStore);
  private readonly editorEngine = inject(EDITOR_ENGINE);
  private readonly current = inject(CurrentAnimation);
  private readonly prompts = inject(SavePrompts);
  private readonly conflict = inject(SaveConflict);
  private readonly router = inject(Router);
  private readonly statusSignal = signal<SaveStatus>('idle');
  private resumeInEditor = false;

  readonly status = this.statusSignal.asReadonly();

  constructor() {
    this.router.events
      .pipe(filter((event) => event instanceof NavigationEnd))
      .subscribe((event) => this.onNavigated(event.urlAfterRedirects));
  }

  /** Saves the editor's work; does nothing while a save is under way or with no document. */
  async save(): Promise<void> {
    if (this.statusSignal() === 'saving' || this.engine.document() === null) return;
    this.statusSignal.set('saving');
    const saved = await this.saveNow().catch((error: unknown) => {
      this.statusSignal.set('idle');
      throw error;
    });
    this.statusSignal.set(saved ? 'saved' : 'idle');
  }

  private async saveNow(): Promise<boolean> {
    if (!this.access.signedIn()) {
      await this.askToSignIn();
      return false;
    }
    const document = await this.editorEngine.serialize();
    try {
      return await this.finish(await this.write(document));
    } catch (error: unknown) {
      return this.recover(toLibraryFailure(error), document);
    }
  }

  /** Saves over the current animation, or creates it in the project the person picks. */
  private async write(document: Uint8Array): Promise<AnimationSummary | null> {
    const state = this.current.state();
    if (state.kind === 'saved') return this.store.saveDocument(state.id, document, state.version);
    const choice = await this.prompts.askForProject();
    if (choice === null) return null;
    const projectId =
      'projectId' in choice
        ? choice.projectId
        : (await this.store.createProject(choice.newProjectName)).id;
    return this.store.createAnimation(projectId, document);
  }

  /** Records `saved` as the editor's animation, and its route; `null` when nothing was saved. */
  private async finish(saved: AnimationSummary | null): Promise<boolean> {
    if (saved === null) return false;
    this.current.setSaved(saved);
    await this.engine.markSaved();
    const route = `${EDITOR_PATH}/${saved.id}`;
    if (this.router.url !== route) await this.router.navigateByUrl(route, { replaceUrl: true });
    return true;
  }

  /** What a failed save leads to: signing in again, a conflict, the quota, or its message. */
  private async recover(failure: LibraryFailure, document: Uint8Array): Promise<boolean> {
    switch (failure.code) {
      case UNAUTHENTICATED:
        await this.askToSignIn();
        return false;
      case VERSION_CONFLICT:
        return this.resolveConflict(document);
      case STORAGE_EXCEEDED:
        await this.prompts.showQuota(await this.usageOf(failure));
        return false;
      case ANIMATION_NOT_FOUND:
        // Deleted elsewhere: the work is kept as unsaved, and the next save asks for a project.
        this.current.setUnsaved();
        break;
    }
    await this.prompts.showFailure(failure);
    return false;
  }

  private async resolveConflict(document: Uint8Array): Promise<boolean> {
    const state = this.current.state();
    const choice = await this.prompts.askAboutConflict();
    if (choice === null || state.kind !== 'saved') return false;
    const conflict = { id: state.id, projectId: state.projectId, document };
    try {
      const saved = await this.conflict.resolve(conflict, choice);
      return saved === null ? true : await this.finish(saved);
    } catch (error: unknown) {
      return this.recover(toLibraryFailure(error), document);
    }
  }

  /** The usage the quota problem names, or the store's when it names none. */
  private async usageOf(failure: LibraryFailure): Promise<StorageUsage> {
    const usedBytes = numberParam(failure, 'used');
    const limitBytes = numberParam(failure, 'limit');
    if (usedBytes !== undefined && limitBytes !== undefined) return { usedBytes, limitBytes };
    return this.store.usage().catch(() => ({ usedBytes: 0, limitBytes: null }));
  }

  /** Sends a visitor to sign in or up, to save once back in the editor. */
  private async askToSignIn(): Promise<void> {
    const choice = await this.prompts.askToSignIn();
    if (choice === null) return;
    this.resumeInEditor = true;
    const returnUrl = this.router.url;
    await this.router.navigate([SIGN_IN_PATHS[choice]], { queryParams: { returnUrl } });
  }

  /** Back in the editor after signing in: the save the visitor asked for goes on. */
  private onNavigated(url: string): void {
    if (!this.resumeInEditor || !url.startsWith(EDITOR_PATH)) return;
    this.resumeInEditor = false;
    if (this.access.signedIn()) void this.save();
  }
}
