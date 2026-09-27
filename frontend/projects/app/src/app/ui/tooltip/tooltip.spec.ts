import { ChangeDetectionStrategy, Component } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Tooltip } from './tooltip';

@Component({
  imports: [Tooltip],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button type="button" aria-label="Pencil" lpTooltip="Pencil" lpTooltipShortcut="B">P</button>
  `,
})
class Host {}

describe('Tooltip', () => {
  let fixture: ComponentFixture<Host>;

  async function setup(): Promise<HTMLButtonElement> {
    fixture = TestBed.createComponent(Host);
    document.body.append(fixture.nativeElement);
    await fixture.whenStable();
    return fixture.nativeElement.querySelector('button');
  }

  function tooltip(): HTMLElement | null {
    return document.querySelector('[role="tooltip"]');
  }

  afterEach(() => {
    vi.useRealTimers();
    fixture.destroy();
    document.body.replaceChildren();
  });

  it('shows the name and the shortcut on hover, and describes the control by them', async () => {
    const button = await setup();

    button.dispatchEvent(new PointerEvent('pointerenter'));

    expect(tooltip()?.textContent).toContain('Pencil');
    expect(tooltip()?.querySelector('kbd')?.textContent).toBe('B');
    expect(button.getAttribute('aria-describedby')).toBe(tooltip()?.id);
  });

  it('shows on keyboard focus, and closes when the focus leaves', async () => {
    const button = await setup();

    // Focused as Tab does, which makes the button `:focus-visible` to jsdom too.
    document.body.addEventListener('keydown', () => button.focus(), { once: true });
    document.body.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }));
    expect(tooltip()?.textContent).toContain('B');

    button.blur();
    expect(tooltip()).toBeNull();
    expect(button.hasAttribute('aria-describedby')).toBe(false);
  });

  it('closes on Escape without moving the pointer or the focus', async () => {
    const button = await setup();
    button.dispatchEvent(new PointerEvent('pointerenter'));

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));

    expect(tooltip()).toBeNull();
  });

  it('stays while the pointer moves onto it, and closes once it leaves both', async () => {
    const button = await setup();
    vi.useFakeTimers();
    button.dispatchEvent(new PointerEvent('pointerenter'));

    button.dispatchEvent(new PointerEvent('pointerleave'));
    tooltip()?.dispatchEvent(new PointerEvent('pointerenter'));
    vi.advanceTimersByTime(1000);
    expect(tooltip()).not.toBeNull();

    tooltip()?.dispatchEvent(new PointerEvent('pointerleave'));
    vi.advanceTimersByTime(1000);
    expect(tooltip()).toBeNull();
  });
});
