/**
 * Where an Alt+arrow press or a drop moves a palette entry: one step, and never onto entry 0,
 * the checkerboard eraser slot that never moves and is never moved onto (editor.md, U4).
 */
export function reorderTarget(from: number, direction: -1 | 1, paletteSize: number): number | null {
  const target = from + direction;
  return target >= 1 && target < paletteSize ? target : null;
}

/** Where a drop lands: any editable slot, entry 0 excluded; a drop there or on itself does nothing. */
export function dropTarget(from: number, to: number, paletteSize: number): number | null {
  return to >= 1 && to < paletteSize && to !== from ? to : null;
}
