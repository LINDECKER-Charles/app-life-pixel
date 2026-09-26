import { defaultEnvironment, isTime, readViewState, viewParams } from './view-state';

const MONITORED = ['staging', 'production'];

describe('view state', () => {
  it('reads the environment and the range of the URL', () => {
    expect(
      readViewState({ env: 'production', from: 'now-7d', to: 'now' }, MONITORED, 'staging'),
    ).toEqual({ env: 'production', from: 'now-7d', to: 'now' });
  });

  it('starts on the last 24 hours of the console’s own environment', () => {
    expect(readViewState({}, MONITORED, 'staging')).toEqual({
      env: 'staging',
      from: 'now-24h',
      to: 'now',
    });
  });

  it('ignores an environment the monitoring does not show, and a malformed time', () => {
    expect(
      readViewState({ env: 'moon', from: 'yesterday', to: 'now-1h' }, MONITORED, 'staging'),
    ).toEqual({ env: 'staging', from: 'now-24h', to: 'now-1h' });
  });

  it('takes the forms of time the admin server takes', () => {
    expect(isTime('now')).toBe(true);
    expect(isTime('now-30m')).toBe(true);
    expect(isTime('1767225600')).toBe(true);
    expect(isTime('2026-09-26T08:00:00Z')).toBe(true);
    expect(isTime('2026-09-26T08:00:00+02:00')).toBe(true);
    expect(isTime('now+1h')).toBe(false);
    expect(isTime('')).toBe(false);
    expect(isTime(undefined)).toBe(false);
  });

  it('falls back to the first monitored environment when its own is not monitored', () => {
    expect(defaultEnvironment(MONITORED, 'staging')).toBe('staging');
    expect(defaultEnvironment(['production'], 'local')).toBe('production');
    expect(defaultEnvironment([], 'local')).toBe('local');
  });

  it('writes the state as query parameters', () => {
    expect(viewParams({ env: 'production', from: 'now-1h', to: 'now' })).toEqual({
      env: 'production',
      from: 'now-1h',
      to: 'now',
    });
  });
});
