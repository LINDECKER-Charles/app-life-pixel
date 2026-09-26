import { saveState, type SaveStateSources } from './save-state';

/** A saved animation, unchanged, with no save under way: override what a case needs. */
function sources(overrides: Partial<SaveStateSources> = {}): SaveStateSources {
  return {
    status: 'idle',
    hasDocument: true,
    hasUnsavedWork: false,
    current: 'saved',
    ...overrides,
  };
}

describe('saveState', () => {
  it('shows nothing without a document', () => {
    expect(saveState(sources({ hasDocument: false }))).toBeNull();
  });

  it('says "not saved" for work the library does not hold, changed or not', () => {
    expect(saveState(sources({ current: 'unsaved' }))).toBe('not-saved');
    expect(saveState(sources({ current: 'unsaved', hasUnsavedWork: true }))).toBe('not-saved');
  });

  it('says "unsaved changes" for a saved animation changed since', () => {
    expect(saveState(sources({ hasUnsavedWork: true }))).toBe('unsaved-changes');
  });

  it('says "saving" while a write runs, never "saved"', () => {
    expect(saveState(sources({ status: 'saving' }))).toBe('saving');
    expect(saveState(sources({ status: 'saving', current: 'unsaved' }))).toBe('saving');
  });

  it('says "saved" for a saved animation unchanged since it was written or read', () => {
    expect(saveState(sources())).toBe('saved');
    expect(saveState(sources({ status: 'saved' }))).toBe('saved');
  });

  it('keeps "saved" from coming back once the work changes after a save', () => {
    expect(saveState(sources({ status: 'saved', hasUnsavedWork: true }))).toBe('unsaved-changes');
  });

  it('says "failed" after a write failed, for new work as for a saved animation', () => {
    expect(saveState(sources({ status: 'failed', hasUnsavedWork: true }))).toBe('failed');
    expect(saveState(sources({ status: 'failed', current: 'unsaved' }))).toBe('failed');
  });
});
