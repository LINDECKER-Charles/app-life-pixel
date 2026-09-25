import { Injectable, signal } from '@angular/core';
import type { TagSpec } from '../../engine/engine-types';

/** What the tag dialog opens with: a new tag over a range, or an existing one to edit. */
export type TagDialogTarget =
  | { readonly kind: 'add'; readonly first: number; readonly last: number }
  | { readonly kind: 'edit'; readonly tag: TagSpec };

/** Whether the tag dialog is open, and on what: shared by the bars and the dialog (U3). */
@Injectable({ providedIn: 'root' })
export class TagDialogState {
  private readonly target = signal<TagDialogTarget | null>(null);

  readonly current = this.target.asReadonly();

  open(target: TagDialogTarget): void {
    this.target.set(target);
  }

  close(): void {
    this.target.set(null);
  }
}
