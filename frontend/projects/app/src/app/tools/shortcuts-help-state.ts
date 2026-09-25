import { Injectable, signal } from '@angular/core';

/** Whether the shortcuts help dialog is open, shared by the tool bar's `?` button and the dialog. */
@Injectable({ providedIn: 'root' })
export class ShortcutsHelpState {
  private readonly openSignal = signal(false);

  readonly isOpen = this.openSignal.asReadonly();

  open(): void {
    this.openSignal.set(true);
  }

  close(): void {
    this.openSignal.set(false);
  }
}
