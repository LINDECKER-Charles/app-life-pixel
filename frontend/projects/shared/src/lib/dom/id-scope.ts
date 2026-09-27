/** The scopes created so far, for the whole page: each takes the next number. */
let scopesCreated = 0;

/** The ids of one component instance, each `<name>-<instance>-<part>`. */
export type IdScope = (part: string) => string;

/**
 * A fresh scope of element ids for one component instance. Ionic's router outlet keeps the pages
 * it leaves in the DOM, so a page's template can live there twice: a fixed id would then be
 * duplicated, and a `for` or `aria-describedby` would point at the hidden copy. Every id a
 * template refers to comes from its instance's scope instead.
 */
export function idScope(name: string): IdScope {
  scopesCreated += 1;
  const prefix = `${name}-${scopesCreated}`;
  return (part) => `${prefix}-${part}`;
}
