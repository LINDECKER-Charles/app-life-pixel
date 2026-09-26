import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter, Router, UrlTree } from '@angular/router';
import { LIBRARY_ACCESS } from './library-access';
import { requireLibraryAccess } from './require-library-access';

function check(signedIn: boolean): boolean | UrlTree {
  TestBed.configureTestingModule({
    providers: [
      provideRouter([]),
      { provide: LIBRARY_ACCESS, useValue: { signedIn: signal(signedIn) } },
    ],
  });
  return TestBed.runInInjectionContext(
    () => requireLibraryAccess({} as never, { url: '/library' } as never) as boolean | UrlTree,
  );
}

describe('requireLibraryAccess', () => {
  it('lets a signed-in person through', () => {
    expect(check(true)).toBe(true);
  });

  it('sends a visitor to sign in, then back to the library', () => {
    const result = check(false);

    expect(result).toBeInstanceOf(UrlTree);
    expect(TestBed.inject(Router).serializeUrl(result as UrlTree)).toBe(
      '/sign-in?returnUrl=%2Flibrary',
    );
  });
});
