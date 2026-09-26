import { inject, Injectable, signal } from '@angular/core';
import { Router } from '@angular/router';
import { EngineStore } from '../../engine/engine-store';
import { CurrentAnimation } from '../current-animation';
import { LIBRARY_ACCESS } from '../library-access';
import { toLibraryFailure, type LibraryFailure } from '../library-types';
import { AnimationLoader } from './animation-loader';
import { OpenConfirmation } from './open-confirmation';

const EDITOR_PATH = '/editor';
const SIGN_IN_PATH = '/sign-in';

/**
 * Opening a saved animation in the editor, from `/editor/:animationId` (accounts.md, H8): with
 * unsaved work, only once the person agrees to lose it; otherwise, or on a failure, the editor
 * keeps its work and its route goes back to it.
 */
@Injectable({ providedIn: 'root' })
export class OpenAnimationFlow {
  private readonly engine = inject(EngineStore);
  private readonly current = inject(CurrentAnimation);
  private readonly access = inject(LIBRARY_ACCESS);
  private readonly loader = inject(AnimationLoader);
  private readonly confirmation = inject(OpenConfirmation);
  private readonly router = inject(Router);
  private readonly openingSignal = signal(false);
  private readonly failureSignal = signal<LibraryFailure | null>(null);

  /** Whether an animation is being read. */
  readonly opening = this.openingSignal.asReadonly();
  /** Why the last opening failed, until the next one or its dismissal. */
  readonly failure = this.failureSignal.asReadonly();

  async open(id: string): Promise<void> {
    if (this.current.holds(id) || this.openingSignal()) return;
    if (!this.access.signedIn()) {
      const returnUrl = `${EDITOR_PATH}/${id}`;
      await this.router.navigate([SIGN_IN_PATH], { queryParams: { returnUrl } });
      return;
    }
    if (this.engine.hasUnsavedWork() && !(await this.confirmation.confirm())) {
      await this.returnToCurrent();
      return;
    }
    await this.load(id);
  }

  dismissFailure(): void {
    this.failureSignal.set(null);
  }

  private async load(id: string): Promise<void> {
    this.openingSignal.set(true);
    this.failureSignal.set(null);
    try {
      await this.loader.load(id);
    } catch (error: unknown) {
      this.failureSignal.set(toLibraryFailure(error));
      await this.returnToCurrent();
    } finally {
      this.openingSignal.set(false);
    }
  }

  /** Puts the route back on what the editor holds: its saved animation, or its unsaved work. */
  private async returnToCurrent(): Promise<void> {
    const id = this.current.id();
    const route = id === null ? EDITOR_PATH : `${EDITOR_PATH}/${id}`;
    await this.router.navigateByUrl(route, { replaceUrl: true });
  }
}
