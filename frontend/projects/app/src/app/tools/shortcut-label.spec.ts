import type { Shortcut } from '../editor/shortcuts';
import { describeShortcut } from './shortcut-label';

describe('describeShortcut', () => {
  it('renders a plain letter in upper case', () => {
    expect(describeShortcut({ key: 'b' })).toBe('B');
  });

  it('renders a punctuation character as typed', () => {
    expect(describeShortcut({ key: '?' })).toBe('?');
    expect(describeShortcut({ key: '[' })).toBe('[');
    expect(describeShortcut({ key: '+' })).toBe('+');
  });

  it('renders modifiers in order: Ctrl/⌘, Shift, Alt', () => {
    expect(describeShortcut({ key: 'z', primary: true })).toBe('Ctrl/⌘+Z');
    expect(describeShortcut({ key: 'z', primary: true, shift: true })).toBe('Ctrl/⌘+Shift+Z');
    expect(describeShortcut({ key: 'r', shift: true })).toBe('Shift+R');
    expect(describeShortcut({ key: 'ArrowLeft', alt: true })).toBe('Alt+←');
  });

  it('describes a registered shortcut as the help dialog lists it', () => {
    const shortcut: Shortcut = { key: 'y', primary: true, label: 'x', action: () => undefined };

    expect(describeShortcut(shortcut)).toBe('Ctrl/⌘+Y');
  });
});
