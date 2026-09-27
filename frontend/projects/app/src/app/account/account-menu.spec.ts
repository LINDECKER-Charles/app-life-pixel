import { HttpTestingController } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { AlertController } from '@ionic/angular';
import axe from 'axe-core';
import { AccountApi } from 'shared';
import { ACCOUNT, openAccountPage, waitForEffects } from './testing/account-test-support';
import { AccountMenu } from './account-menu';

const SERIOUS_IMPACTS = ['serious', 'critical'];

describe('AccountMenu', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('offers to sign in for a stranger', async () => {
    const fixture = await openAccountPage(AccountMenu);

    const link: HTMLAnchorElement = fixture.nativeElement.querySelector('a');
    expect(link.getAttribute('href')).toBe('/sign-in');
  });

  it('carries the current page as returnUrl once there is one to return to', async () => {
    const fixture = await openAccountPage(AccountMenu);
    await TestBed.inject(Router).navigateByUrl('/editor/abc');

    await waitForEffects(() => {
      const link: HTMLAnchorElement = fixture.nativeElement.querySelector('a');
      expect(link.getAttribute('href')).toBe('/sign-in?returnUrl=%2Feditor%2Fabc');
    });
  });

  it("shows a signed-in visitor's address, with a way to sign out", async () => {
    const fixture = await openAccountPage(AccountMenu, ACCOUNT);
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);

    expect(fixture.nativeElement.querySelector('.email')?.textContent).toBe(ACCOUNT.email);

    fixture.nativeElement.querySelector('button[aria-haspopup="menu"]')?.click();
    await fixture.whenStable();
    fixture.nativeElement.querySelector('button[role="menuitem"]')?.click();
    await fixture.whenStable();
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-out')
      .flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => expect(navigation).toHaveBeenCalledWith('/editor'));
  });

  it('is a labelled menu the keyboard opens, and Escape closes on its button', async () => {
    const fixture = await openAccountPage(AccountMenu, ACCOUNT);
    document.body.append(fixture.nativeElement);
    const root = fixture.nativeElement as HTMLElement;
    const trigger = root.querySelector<HTMLButtonElement>('button[aria-haspopup="menu"]');
    const menu = root.querySelector('[role="menu"]');
    if (!trigger || !menu) throw new Error('no account menu');
    const press = async (target: Element, key: string) => {
      target.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
      await fixture.whenStable();
    };

    expect(menu.getAttribute('aria-label')).toBe('Account');
    await press(trigger, 'ArrowDown');
    expect(trigger.getAttribute('aria-expanded')).toBe('true');
    const items = Array.from(menu.querySelectorAll('[role="menuitem"]'), (item) =>
      item.textContent?.trim(),
    );
    expect(items).toEqual(['Account', 'Library', 'Sign out']);
    expect(document.activeElement?.textContent?.trim()).toBe('Account');

    await press(document.activeElement ?? menu, 'Escape');

    expect(trigger.getAttribute('aria-expanded')).toBe('false');
    expect(document.activeElement).toBe(trigger);
    fixture.nativeElement.remove();
  });

  it('opens a blocking reload dialog once a request answers 426', async () => {
    await openAccountPage(AccountMenu, ACCOUNT);
    const alerts = TestBed.inject(AlertController);
    const present = vi.fn().mockResolvedValue(undefined);
    vi.spyOn(alerts, 'create').mockResolvedValue({ present } as never);

    TestBed.inject(AccountApi)
      .get()
      .catch(() => undefined);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/account')
      .flush(
        { code: 'client.update_required', params: { minimum: '2.0.0' } },
        { status: 426, statusText: 'Upgrade Required' },
      );

    await waitForEffects(() => expect(alerts.create).toHaveBeenCalled());
    expect(present).toHaveBeenCalled();
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(AccountMenu, ACCOUNT);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
