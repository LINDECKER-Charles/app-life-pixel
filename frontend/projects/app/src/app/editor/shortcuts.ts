import { Injectable, signal } from '@angular/core';

/** A keyboard shortcut of the editor, registered by the feature that owns its action. */
export interface Shortcut {
  /**
   * `KeyboardEvent.key`: a letter in either case (`'b'`), a character (`'?'`, `'+'`, `'0'`) or a
   * named key (`'ArrowLeft'`, `'Escape'`). A character matches whatever Shift it took to type.
   */
  readonly key: string;
  /** Ctrl, or ⌘ on macOS. */
  readonly primary?: boolean;
  readonly shift?: boolean;
  readonly alt?: boolean;
  /** The i18n key naming the action, as the shortcuts dialog lists it. */
  readonly label: string;
  readonly action: () => void;
}

/** Input types that act as buttons: every other input takes text, or moves a value, with keys. */
const BUTTON_INPUT_TYPES = new Set([
  'button',
  'checkbox',
  'color',
  'file',
  'image',
  'radio',
  'reset',
  'submit',
]);
/** Keys pressed inside a dialog belong to that dialog. */
const OVERLAY_SELECTOR = 'ion-modal, ion-alert, ion-popover, ion-action-sheet, dialog';

function isTextField(element: Element): boolean {
  if (element instanceof HTMLInputElement) return !BUTTON_INPUT_TYPES.has(element.type);
  return (
    element instanceof HTMLTextAreaElement ||
    element instanceof HTMLSelectElement ||
    (element instanceof HTMLElement && element.isContentEditable)
  );
}

function isIgnored(event: KeyboardEvent): boolean {
  if (event.isComposing) return true;
  const target = event.composedPath()[0];
  return (
    target instanceof Element && (isTextField(target) || target.closest(OVERLAY_SELECTOR) !== null)
  );
}

/** A character typed with or without Shift, as opposed to a letter or a named key. */
function isCharacter(key: string): boolean {
  return key.length === 1 && key.toLowerCase() === key.toUpperCase();
}

function matches(shortcut: Shortcut, event: KeyboardEvent): boolean {
  return (
    shortcut.key.toLowerCase() === event.key.toLowerCase() &&
    (shortcut.primary ?? false) === (event.ctrlKey || event.metaKey) &&
    (shortcut.alt ?? false) === event.altKey &&
    (isCharacter(shortcut.key) || (shortcut.shift ?? false) === event.shiftKey)
  );
}

/**
 * The editor's keyboard shortcuts. Features register theirs, and unregister them when they go —
 * `inject(DestroyRef).onDestroy(shortcuts.register([...]))` —; the editor page hands every key
 * press to `handle`, which ignores keys typed into a text field or pressed inside a dialog.
 */
@Injectable({ providedIn: 'root' })
export class Shortcuts {
  private readonly registered = signal<readonly Shortcut[]>([]);

  /** Every registered shortcut, in registration order, for the shortcuts dialog (U4). */
  readonly all = this.registered.asReadonly();

  /** Adds shortcuts; returns the function that removes them. */
  register(shortcuts: readonly Shortcut[]): () => void {
    this.registered.update((list) => [...list, ...shortcuts]);
    return () => this.registered.update((list) => list.filter((item) => !shortcuts.includes(item)));
  }

  /** Runs the action of the first shortcut the key press matches, instead of its default. */
  handle(event: KeyboardEvent): void {
    if (event.defaultPrevented || isIgnored(event)) return;
    const shortcut = this.registered().find((candidate) => matches(candidate, event));
    if (!shortcut) return;
    event.preventDefault();
    shortcut.action();
  }
}
