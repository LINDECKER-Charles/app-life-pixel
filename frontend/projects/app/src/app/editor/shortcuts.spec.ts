import { TestBed } from '@angular/core/testing';
import { Shortcuts, type Shortcut } from './shortcuts';

function press(target: EventTarget, init: KeyboardEventInit): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

describe('Shortcuts', () => {
  let shortcuts: Shortcuts;
  let calls: string[];

  function shortcut(name: string, keys: Omit<Shortcut, 'label' | 'action'>): Shortcut {
    return { ...keys, label: `test.${name}`, action: () => calls.push(name) };
  }

  beforeEach(() => {
    TestBed.configureTestingModule({});
    shortcuts = TestBed.inject(Shortcuts);
    calls = [];
    document.addEventListener('keydown', handle);
  });

  afterEach(() => {
    document.removeEventListener('keydown', handle);
    document.body.replaceChildren();
  });

  function handle(event: KeyboardEvent): void {
    shortcuts.handle(event);
  }

  it('runs the action of the matching shortcut instead of the default', () => {
    shortcuts.register([shortcut('pencil', { key: 'b' })]);

    const event = press(document.body, { key: 'b' });

    expect(calls).toEqual(['pencil']);
    expect(event.defaultPrevented).toBe(true);
  });

  it('tells letters apart by their modifiers', () => {
    shortcuts.register([
      shortcut('rectangle', { key: 'r' }),
      shortcut('filled', { key: 'r', shift: true }),
      shortcut('undo', { key: 'z', primary: true }),
      shortcut('redo', { key: 'z', primary: true, shift: true }),
    ]);

    press(document.body, { key: 'r' });
    press(document.body, { key: 'R', shiftKey: true });
    press(document.body, { key: 'z', ctrlKey: true });
    press(document.body, { key: 'Z', metaKey: true, shiftKey: true });
    press(document.body, { key: 'r', ctrlKey: true });

    expect(calls).toEqual(['rectangle', 'filled', 'undo', 'redo']);
  });

  it('matches a character whatever Shift it took to type', () => {
    shortcuts.register([shortcut('help', { key: '?' }), shortcut('zoom in', { key: '+' })]);

    press(document.body, { key: '?', shiftKey: true });
    press(document.body, { key: '+' });

    expect(calls).toEqual(['help', 'zoom in']);
  });

  it('ignores keys typed into a text field', () => {
    shortcuts.register([shortcut('pencil', { key: 'b' })]);
    const field = document.createElement('input');
    const area = document.createElement('textarea');
    document.body.append(field, area);

    const event = press(field, { key: 'b' });
    press(area, { key: 'b' });

    expect(calls).toEqual([]);
    expect(event.defaultPrevented).toBe(false);
  });

  it('still acts on keys pressed on a button', () => {
    shortcuts.register([shortcut('pencil', { key: 'b' })]);
    const button = document.createElement('button');
    document.body.append(button);

    press(button, { key: 'b' });

    expect(calls).toEqual(['pencil']);
  });

  it('lists what is registered, and forgets what is unregistered', () => {
    const pencil = shortcut('pencil', { key: 'b' });
    const unregister = shortcuts.register([pencil]);
    expect(shortcuts.all()).toEqual([pencil]);

    unregister();
    press(document.body, { key: 'b' });

    expect(shortcuts.all()).toEqual([]);
    expect(calls).toEqual([]);
  });
});
