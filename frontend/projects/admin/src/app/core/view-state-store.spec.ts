import { TestBed } from '@angular/core/testing';
import { provideRouter, Router } from '@angular/router';
import { SESSION } from '../testing/admin-test-support';
import { SessionState } from './session-state';
import { ViewStateStore } from './view-state-store';

describe('ViewStateStore', () => {
  let router: Router;

  beforeEach(() => {
    TestBed.configureTestingModule({ providers: [provideRouter([{ path: '**', children: [] }])] });
    TestBed.inject(SessionState).set(SESSION);
    router = TestBed.inject(Router);
  });

  it('reads the environment and the range of the URL', async () => {
    await router.navigateByUrl('/logs?env=production&from=now-7d&to=now');
    const store = TestBed.inject(ViewStateStore);

    expect(store.state()).toEqual({ env: 'production', from: 'now-7d', to: 'now' });
    expect(store.params()).toEqual({ env: 'production', from: 'now-7d', to: 'now' });
  });

  it('follows the URL from one view to the next', async () => {
    const store = TestBed.inject(ViewStateStore);
    await router.navigateByUrl('/');
    expect(store.state().env).toBe('staging');

    await router.navigateByUrl('/alerts?env=production');

    expect(store.state()).toEqual({ env: 'production', from: 'now-24h', to: 'now' });
  });

  it('changes the environment or the range in the URL, keeping the view’s other parameters', async () => {
    await router.navigateByUrl('/logs?level=error&env=staging');
    const store = TestBed.inject(ViewStateStore);

    await store.update({ env: 'production', from: 'now-1h' });

    expect(router.url).toBe('/logs?level=error&env=production&from=now-1h');
    expect(store.state()).toEqual({ env: 'production', from: 'now-1h', to: 'now' });
  });
});
