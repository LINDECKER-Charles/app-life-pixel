import { Injectable, signal } from '@angular/core';
import type { LibraryFailure } from '../library-types';

/** How a visitor chose to go on: with an account they have, or a new one. */
export type SignInChoice = 'sign-in' | 'sign-up';
/** Where unsaved work goes: a project of the library, or a new one by this name. */
export type ProjectChoice = { readonly projectId: string } | { readonly newProjectName: string };
/** What to do when the saved animation changed elsewhere since the editor read it. */
export type ConflictChoice = 'reload' | 'overwrite' | 'copy';
/** The account's storage, when a save would exceed it. */
export interface StorageUsage {
  readonly usedBytes: number;
  readonly limitBytes: number | null;
}

/** The question the save dialog shows, with the function its answer settles. */
export type SavePrompt =
  | { readonly kind: 'sign-in'; readonly answer: (choice: SignInChoice | null) => void }
  | { readonly kind: 'project'; readonly answer: (choice: ProjectChoice | null) => void }
  | { readonly kind: 'conflict'; readonly answer: (choice: ConflictChoice | null) => void }
  | { readonly kind: 'quota'; readonly usage: StorageUsage; readonly answer: () => void }
  | { readonly kind: 'failure'; readonly failure: LibraryFailure; readonly answer: () => void };

/**
 * The questions saving asks, one at a time (accounts.md, H8): `SaveFlow` awaits them, the save
 * dialog shows the current one and answers it. Dismissing the dialog answers `null`.
 */
@Injectable({ providedIn: 'root' })
export class SavePrompts {
  private readonly currentSignal = signal<SavePrompt | null>(null);

  /** The question shown now, or `null`. */
  readonly current = this.currentSignal.asReadonly();

  askToSignIn(): Promise<SignInChoice | null> {
    return new Promise((resolve) => this.show({ kind: 'sign-in', answer: this.settle(resolve) }));
  }

  askForProject(): Promise<ProjectChoice | null> {
    return new Promise((resolve) => this.show({ kind: 'project', answer: this.settle(resolve) }));
  }

  askAboutConflict(): Promise<ConflictChoice | null> {
    return new Promise((resolve) => this.show({ kind: 'conflict', answer: this.settle(resolve) }));
  }

  showQuota(usage: StorageUsage): Promise<void> {
    return new Promise((resolve) =>
      this.show({ kind: 'quota', usage, answer: this.settle(resolve) }),
    );
  }

  showFailure(failure: LibraryFailure): Promise<void> {
    return new Promise((resolve) =>
      this.show({ kind: 'failure', failure, answer: this.settle(resolve) }),
    );
  }

  answerSignIn(choice: SignInChoice): void {
    const prompt = this.currentSignal();
    if (prompt?.kind === 'sign-in') prompt.answer(choice);
  }

  answerProject(choice: ProjectChoice): void {
    const prompt = this.currentSignal();
    if (prompt?.kind === 'project') prompt.answer(choice);
  }

  answerConflict(choice: ConflictChoice): void {
    const prompt = this.currentSignal();
    if (prompt?.kind === 'conflict') prompt.answer(choice);
  }

  /** Closes the question shown, as a dismissal: `null`, or nothing for a message. */
  dismiss(): void {
    const prompt = this.currentSignal();
    if (prompt === null) return;
    if (prompt.kind === 'quota' || prompt.kind === 'failure') prompt.answer();
    else prompt.answer(null);
  }

  private show(prompt: SavePrompt): void {
    this.dismiss();
    this.currentSignal.set(prompt);
  }

  /** `resolve`, once, closing the question first so that the next one can open. */
  private settle<T>(resolve: (value: T) => void): (value: T) => void {
    let settled = false;
    return (value: T) => {
      if (settled) return;
      settled = true;
      this.currentSignal.set(null);
      resolve(value);
    };
  }
}
