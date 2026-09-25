import { dropTarget, reorderTarget } from './palette-reorder';

describe('reorderTarget', () => {
  it('moves one step, never onto entry 0', () => {
    expect(reorderTarget(2, -1, 5)).toBe(1);
    expect(reorderTarget(1, -1, 5)).toBeNull();
    expect(reorderTarget(4, 1, 5)).toBeNull();
    expect(reorderTarget(2, 1, 5)).toBe(3);
  });
});

describe('dropTarget', () => {
  it('accepts a drop on another editable slot', () => {
    expect(dropTarget(2, 4, 6)).toBe(4);
  });

  it('refuses a drop on entry 0, out of range, or on itself', () => {
    expect(dropTarget(2, 0, 6)).toBeNull();
    expect(dropTarget(2, 6, 6)).toBeNull();
    expect(dropTarget(2, 2, 6)).toBeNull();
  });
});
