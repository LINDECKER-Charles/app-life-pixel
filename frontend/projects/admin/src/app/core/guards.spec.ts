import { TestBed } from '@angular/core/testing';
import {
  ActivatedRouteSnapshot,
  provideRouter,
  Router,
  RouterStateSnapshot,
  UrlTree,
} from '@angular/router';
import { SESSION } from '../testing/admin-test-support';
import { requireAdmin, requireSignedOut } from './guards';
import { SessionState } from './session-state';

function run(guard: typeof requireAdmin, url = '/logs?env=staging'): boolean | UrlTree {
  const state = { url } as RouterStateSnapshot;
  return TestBed.runInInjectionContext(
    () => guard({} as ActivatedRouteSnapshot, state) as boolean | UrlTree,
  );
}

describe('guards', () => {
  beforeEach(() => TestBed.configureTestingModule({ providers: [provideRouter([])] }));

  it('sends a signed-out visitor to sign in, and back afterwards', () => {
    TestBed.inject(SessionState).set(null);
    const router = TestBed.inject(Router);

    expect(router.serializeUrl(run(requireAdmin) as UrlTree)).toBe(
      '/sign-in?returnUrl=%2Flogs%3Fenv%3Dstaging',
    );
    expect(run(requireSignedOut)).toBe(true);
  });

  it('lets an admin in, and past the sign-in page', () => {
    TestBed.inject(SessionState).set(SESSION);
    const router = TestBed.inject(Router);

    expect(run(requireAdmin)).toBe(true);
    expect(router.serializeUrl(run(requireSignedOut) as UrlTree)).toBe('/');
  });
});
