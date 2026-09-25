/**
 * Reordering helpers shared by the frame strip and the layer list: both move an item by drag and
 * drop or by one step with the keyboard, into the `position` `moveFrame` and `moveLayer` expect —
 * the item's index once the list is rebuilt (core.md, K2's splice semantics).
 */

/** The position a drop at `targetIndex` gives: the item takes that slot, in list order. */
export function dropPosition(targetIndex: number): number {
  return targetIndex;
}

/** The position moving `index` one step toward `direction` gives, or `null` at the edge. */
export function stepPosition(index: number, length: number, direction: 1 | -1): number | null {
  const next = index + direction;
  return next >= 0 && next < length ? next : null;
}
