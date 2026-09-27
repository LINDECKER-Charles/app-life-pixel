import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { vi } from 'vitest';
import { App } from './app';
import { SessionState } from './core/session-state';
import { SessionStore } from './core/session-store';
import { openPage, seriousViolations, settled, textOf } from './testing/admin-test-support';

describe('App', () => {
  it('shows the environment banner, amber for staging, on every page', async () => {
    const fixture = await openPage(App, { url: '/logs?env=production&from=now-7d' });
    const banner = fixture.nativeElement.querySelector('.banner') as HTMLElement;

    expect(banner.dataset['tone']).toBe('staging');
    expect(textOf(banner)).toContain('Staging');
  });

  it('carries the environment and the range from view to view', async () => {
    const fixture = await openPage(App, { url: '/logs?env=production&from=now-7d&level=error' });
    const users = fixture.nativeElement.querySelector('nav a[href^="/users"]');

    expect(users.getAttribute('href')).toBe('/users?env=production&from=now-7d&to=now');
  });

  it('changes the environment of the view in the URL', async () => {
    const fixture = await openPage(App, { url: '/logs?level=error' });
    const select = fixture.nativeElement.querySelector(
      'lp-view-controls select',
    ) as HTMLSelectElement;

    select.value = 'production';
    select.dispatchEvent(new Event('change'));

    await settled(() =>
      expect(TestBed.inject(Router).url).toBe('/logs?level=error&env=production'),
    );
  });

  it('goes to sign in when the session ends, and comes back afterwards', async () => {
    const fixture = await openPage(App, { url: '/users?env=staging' });

    TestBed.inject(SessionState).end();

    await settled(() =>
      expect(TestBed.inject(Router).url).toBe('/sign-in?returnUrl=%2Fusers%3Fenv%3Dstaging'),
    );
    expect(fixture.nativeElement.querySelector('lp-app-header')).toBeNull();
  });

  it('signs out', async () => {
    const fixture = await openPage(App, { url: '/' });
    const signOut = vi.spyOn(TestBed.inject(SessionStore), 'signOut').mockImplementation(() => {
      TestBed.inject(SessionState).set(null);
      return Promise.resolve();
    });

    Array.from(fixture.nativeElement.querySelectorAll('button') as NodeListOf<HTMLElement>)
      .find((button) => textOf(button) === 'Sign out')
      ?.click();

    expect(signOut).toHaveBeenCalled();
    await settled(() => expect(TestBed.inject(Router).url).toMatch(/^\/sign-in/));
  });

  it('passes axe', async () => {
    const fixture = await openPage(App, { url: '/' });

    expect(textOf(fixture.nativeElement.querySelector('.skip-link'))).toBe('Skip to the content');
    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
