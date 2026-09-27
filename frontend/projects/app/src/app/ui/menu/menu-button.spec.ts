import { ChangeDetectionStrategy, Component } from '@angular/core';
import { type ComponentFixture, TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { MenuButton } from './menu-button';

const SERIOUS_IMPACTS = ['serious', 'critical'];

@Component({
  imports: [MenuButton],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <lp-menu-button label="Account">
      <span lpMenuTrigger>lee&#64;example.com</span>
      <p lpMenuHeader class="lp-menu__header">Signed in</p>
      <a role="menuitem" class="lp-menu__item" href="/account">Account</a>
      <a role="menuitem" class="lp-menu__item" href="/library">Library</a>
      <button role="menuitem" class="lp-menu__item" type="button" (click)="signedOut = true">
        Sign out
      </button>
    </lp-menu-button>
    <button type="button" class="outside">Elsewhere</button>
  `,
})
class Host {
  signedOut = false;
}

interface Menu {
  readonly fixture: ComponentFixture<Host>;
  readonly trigger: HTMLButtonElement;
  readonly menu: HTMLElement;
  readonly items: HTMLElement[];
}

async function render(): Promise<Menu> {
  const fixture = TestBed.createComponent(Host);
  document.body.append(fixture.nativeElement);
  await fixture.whenStable();
  const root = fixture.nativeElement as HTMLElement;
  const trigger = root.querySelector<HTMLButtonElement>('button[aria-haspopup="menu"]');
  const menu = root.querySelector<HTMLElement>('[role="menu"]');
  if (!trigger || !menu) throw new Error('no menu button');
  const items = Array.from(menu.querySelectorAll<HTMLElement>('[role="menuitem"]'));
  return { fixture, trigger, menu, items };
}

async function press(target: HTMLElement, key: string, fixture: ComponentFixture<Host>) {
  target.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
  await fixture.whenStable();
}

function isOpen({ trigger, menu }: Menu): boolean {
  return trigger.getAttribute('aria-expanded') === 'true' && !menu.parentElement?.hidden;
}

describe('MenuButton', () => {
  afterEach(() => document.body.replaceChildren());

  it('names the menu and ties it to its closed button', async () => {
    const menu = await render();

    expect(menu.menu.getAttribute('aria-label')).toBe('Account');
    expect(menu.trigger.getAttribute('aria-controls')).toBe(menu.menu.id);
    expect(menu.trigger.textContent).toContain('lee@example.com');
    expect(isOpen(menu)).toBe(false);
  });

  it('opens from the keyboard on its first item, or on its last with Up Arrow', async () => {
    const menu = await render();

    await press(menu.trigger, 'ArrowDown', menu.fixture);
    expect(isOpen(menu)).toBe(true);
    expect(document.activeElement).toBe(menu.items[0]);

    await press(menu.items[0], 'Escape', menu.fixture);
    await press(menu.trigger, 'ArrowUp', menu.fixture);
    expect(document.activeElement).toBe(menu.items[2]);
  });

  it('opens on a click, as Enter and Space do', async () => {
    const menu = await render();

    menu.trigger.click();
    await menu.fixture.whenStable();

    expect(isOpen(menu)).toBe(true);
    expect(document.activeElement).toBe(menu.items[0]);
  });

  it('moves among the items with the arrows, Home and End, wrapping around', async () => {
    const menu = await render();
    await press(menu.trigger, 'ArrowDown', menu.fixture);

    await press(menu.items[0], 'ArrowDown', menu.fixture);
    expect(document.activeElement).toBe(menu.items[1]);
    await press(menu.items[1], 'End', menu.fixture);
    expect(document.activeElement).toBe(menu.items[2]);
    await press(menu.items[2], 'ArrowDown', menu.fixture);
    expect(document.activeElement).toBe(menu.items[0]);
    await press(menu.items[0], 'ArrowUp', menu.fixture);
    expect(document.activeElement).toBe(menu.items[2]);
    await press(menu.items[2], 'Home', menu.fixture);
    expect(document.activeElement).toBe(menu.items[0]);
  });

  it('closes on Escape and gives the focus back to its button', async () => {
    const menu = await render();
    await press(menu.trigger, 'ArrowDown', menu.fixture);

    await press(menu.items[1], 'Escape', menu.fixture);

    expect(isOpen(menu)).toBe(false);
    expect(document.activeElement).toBe(menu.trigger);
  });

  it('runs the chosen item, then closes with the focus on its button', async () => {
    const menu = await render();
    await press(menu.trigger, 'ArrowDown', menu.fixture);

    menu.items[2].click();
    await menu.fixture.whenStable();

    expect(menu.fixture.componentInstance.signedOut).toBe(true);
    expect(isOpen(menu)).toBe(false);
    expect(document.activeElement).toBe(menu.trigger);
  });

  it('closes on a click elsewhere', async () => {
    const menu = await render();
    menu.trigger.click();
    await menu.fixture.whenStable();

    (menu.fixture.nativeElement as HTMLElement).querySelector<HTMLElement>('.outside')?.click();
    await menu.fixture.whenStable();

    expect(isOpen(menu)).toBe(false);
  });

  it('has no serious accessibility violation, open or closed', async () => {
    const menu = await render();
    const serious = async () =>
      (await axe.run(menu.fixture.nativeElement)).violations.filter((violation) =>
        SERIOUS_IMPACTS.includes(violation.impact ?? ''),
      );

    expect(await serious()).toEqual([]);
    await press(menu.trigger, 'ArrowDown', menu.fixture);
    expect(await serious()).toEqual([]);
  });
});
