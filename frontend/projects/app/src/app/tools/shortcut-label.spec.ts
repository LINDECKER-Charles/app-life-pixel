import { describeShortcut } from './shortcut-label';

describe('describeShortcut', () => {
  it('renders a plain letter in upper case', () => {
    expect(describeShortcut({ key: 'b', label: 'x', action: () => undefined })).toBe('B');
  });

  it('renders a punctuation character as typed', () => {
    expect(describeShortcut({ key: '?', label: 'x', action: () => undefined })).toBe('?');
    expect(describeShortcut({ key: '[', label: 'x', action: () => undefined })).toBe('[');
  });

  it('renders modifiers in order: Ctrl/⌘, Shift, Alt', () => {
    expect(describeShortcut({ key: 'z', primary: true, label: 'x', action: () => undefined })).toBe(
      'Ctrl/⌘+Z',
    );
    expect(
      describeShortcut({
        key: 'z',
        primary: true,
        shift: true,
        label: 'x',
        action: () => undefined,
      }),
    ).toBe('Ctrl/⌘+Shift+Z');
    expect(
      describeShortcut({ key: 'ArrowLeft', alt: true, label: 'x', action: () => undefined }),
    ).toBe('Alt+←');
  });
});
