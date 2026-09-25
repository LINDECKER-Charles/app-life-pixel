/** A list of ids before and after a change, and the id that was active before it. */
export interface ListChange<Id> {
  readonly previousIds: readonly Id[];
  readonly ids: readonly Id[];
  readonly active: Id | null;
}

/**
 * The id to keep active once a list changes: the same one while it remains, else the nearest
 * one that remains in the former order — the next first, then the previous —, else `null`.
 */
export function nearestSurvivor<Id>({ previousIds, ids, active }: ListChange<Id>): Id | null {
  if (active === null) return null;
  if (ids.includes(active)) return active;
  const index = previousIds.indexOf(active);
  if (index < 0) return null;
  for (let distance = 1; distance < previousIds.length; distance++) {
    const survivor = [previousIds[index + distance], previousIds[index - distance]].find(
      (id) => id !== undefined && ids.includes(id),
    );
    if (survivor !== undefined) return survivor;
  }
  return null;
}
