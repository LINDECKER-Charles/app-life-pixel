import type { Shortcut } from '../editor/shortcuts';

/** How a named key is written in the shortcuts dialog: mostly its own name, a few spelled out. */
const NAMED_KEYS: Readonly<Record<string, string>> = {
  ArrowLeft: '←',
  ArrowRight: '→',
  ArrowUp: '↑',
  ArrowDown: '↓',
  Escape: 'Esc',
  ' ': 'Space',
};

function keyLabel(key: string): string {
  if (key in NAMED_KEYS) return NAMED_KEYS[key];
  return key.length === 1 ? key.toUpperCase() : key;
}

/** Renders a shortcut's key combination, e.g. `Ctrl/⌘+Shift+Z`, `[`, `?`. */
export function describeShortcut(shortcut: Shortcut): string {
  const parts: string[] = [];
  if (shortcut.primary) parts.push('Ctrl/⌘');
  if (shortcut.shift) parts.push('Shift');
  if (shortcut.alt) parts.push('Alt');
  parts.push(keyLabel(shortcut.key));
  return parts.join('+');
}
