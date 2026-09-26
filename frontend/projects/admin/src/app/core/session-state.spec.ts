import { TestBed } from '@angular/core/testing';
import { SESSION } from '../testing/admin-test-support';
import { SessionState } from './session-state';

describe('SessionState', () => {
  it('holds the CSRF token of the live session', () => {
    const state = TestBed.inject(SessionState);
    expect(state.csrfToken()).toBeNull();

    state.set(SESSION);

    expect(state.csrfToken()).toBe('csrf-1');
  });

  it('ends a read session, but not one never read', () => {
    const state = TestBed.inject(SessionState);
    state.end();
    expect(state.session()).toBeUndefined();

    state.set(SESSION);
    state.end();

    expect(state.session()).toBeNull();
  });
});
