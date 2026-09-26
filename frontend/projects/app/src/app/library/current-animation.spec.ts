import { TestBed } from '@angular/core/testing';
import { CurrentAnimation } from './current-animation';

const SUMMARY = {
  id: 'a1',
  projectId: 'p1',
  title: 'Mascot',
  width: 8,
  height: 8,
  frameCount: 1,
  documentBytes: 10,
  version: 3,
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
};

describe('CurrentAnimation', () => {
  it('starts with unsaved work', () => {
    const current = TestBed.inject(CurrentAnimation);

    expect(current.state()).toEqual({ kind: 'unsaved' });
    expect(current.id()).toBeNull();
  });

  it('records the saved animation, its version and its project, until unsaved again', () => {
    const current = TestBed.inject(CurrentAnimation);

    current.setSaved(SUMMARY);
    expect(current.state()).toEqual({ kind: 'saved', id: 'a1', projectId: 'p1', version: 3 });
    expect(current.holds('a1')).toBe(true);
    expect(current.holds('a2')).toBe(false);

    current.setUnsaved();
    expect(current.holds('a1')).toBe(false);
  });
});
