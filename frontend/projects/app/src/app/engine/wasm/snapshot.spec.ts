import { RecoverySnapshot, SNAPSHOT_INTERVAL_MS, type Snapshot } from './snapshot';

const SAVED: Snapshot = { document: Uint8Array.from([1, 2, 3]), hasUnsavedWork: false };

describe('RecoverySnapshot', () => {
  let takes: number;
  let snapshot: RecoverySnapshot;

  beforeEach(() => {
    vi.useFakeTimers();
    takes = 0;
    snapshot = new RecoverySnapshot(() => (takes += 1));
  });

  afterEach(() => {
    snapshot.dispose();
    vi.useRealTimers();
  });

  it('has nothing to reopen before a snapshot is kept', () => {
    expect(snapshot.recover()).toEqual({
      snapshot: { document: null, hasUnsavedWork: false },
      lostChanges: 0,
    });
  });

  it('counts the changes made since the kept snapshot, which a recovery loses', () => {
    snapshot.recordChange();
    snapshot.keep(SAVED);
    snapshot.recordChange();
    snapshot.recordChange();

    expect(snapshot.recover()).toEqual({ snapshot: SAVED, lostChanges: 2 });
    expect(snapshot.recover().lostChanges).toBe(0);
  });

  it('asks for a snapshot once changes have gone 30 seconds without one', () => {
    snapshot.recordChange();
    vi.advanceTimersByTime(SNAPSHOT_INTERVAL_MS / 2);
    snapshot.recordChange();
    vi.advanceTimersByTime(SNAPSHOT_INTERVAL_MS / 2 - 1);
    expect(takes).toBe(0);

    vi.advanceTimersByTime(1);
    expect(takes).toBe(1);

    vi.advanceTimersByTime(SNAPSHOT_INTERVAL_MS);
    expect(takes).toBe(1);
  });

  it('asks for none while nothing changes, or once a snapshot was kept', () => {
    vi.advanceTimersByTime(SNAPSHOT_INTERVAL_MS * 3);
    snapshot.recordChange();
    snapshot.keep(SAVED);
    vi.advanceTimersByTime(SNAPSHOT_INTERVAL_MS);

    expect(takes).toBe(0);
  });
});
