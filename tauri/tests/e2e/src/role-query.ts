// The functions below run in the webview, not in Node: ROLE_QUERY_SCRIPT joins their sources,
// so each may call the others by name and nothing else of this module. WebKitGTK's WebDriver
// has no command answering an element's role or accessible name, so they compute both — for the
// roles and naming rules the app uses, not the whole of ARIA.

/** An accessible name to match: the exact text, or a pattern's source and flags. */
type NameMatch = { readonly exact: string } | { readonly pattern: string; readonly flags: string };

/** The elements that have `role` without saying so, as far as the app needs. */
function implicitSelector(role: string): string {
  const selectors: Partial<Record<string, string>> = {
    button: 'button, ion-button, input[type="button"], input[type="submit"]',
    link: 'a[href]',
    textbox: 'input:not([type]), input[type="text"], textarea',
    spinbutton: 'input[type="number"]',
    radio: 'input[type="radio"]',
    table: 'table',
    row: 'tr',
    rowheader: 'th[scope="row"]',
    cell: 'td',
  };
  return selectors[role] ?? '';
}

/** The elements of `scope` with `role`, given or implicit. */
function candidates(scope: ParentNode, role: string): Element[] {
  const implicit = implicitSelector(role);
  const selector = implicit === '' ? `[role="${role}"]` : `[role="${role}"], ${implicit}`;
  return Array.from(scope.querySelectorAll(selector)).filter((element) => {
    const given = element.getAttribute('role');
    return given === null || given === role;
  });
}

/** Whether a person sees the element: laid out, and not hidden behind an open dialog. */
function isShown(element: Element): boolean {
  const hidden = element.closest('[aria-hidden="true"], [inert]');
  return hidden === null && element.getClientRects().length > 0;
}

function collapse(text: string | null): string {
  return (text ?? '').replace(/\s+/g, ' ').trim();
}

/** The accessible name: `aria-labelledby`, `aria-label`, a field's labels, else the text. */
function accessibleName(element: Element): string {
  const labelledBy = element.getAttribute('aria-labelledby');
  if (labelledBy !== null) {
    const ids = labelledBy.split(/\s+/);
    return collapse(ids.map((id) => document.getElementById(id)?.textContent ?? '').join(' '));
  }
  const label = collapse(element.getAttribute('aria-label'));
  if (label !== '') return label;
  if (element instanceof HTMLInputElement && element.labels !== null) {
    const labels = Array.from(element.labels, (item) => item.textContent);
    if (labels.length > 0) return collapse(labels.join(' '));
  }
  const caption = element instanceof HTMLTableElement ? element.caption : null;
  return collapse((caption ?? element).textContent);
}

function nameMatches(name: string, match: NameMatch | null): boolean {
  if (match === null) return true;
  if ('exact' in match) return name === match.exact;
  return new RegExp(match.pattern, match.flags).test(name);
}

/** The shown elements of `role` inside `scope`, or the page, whose name matches. */
function findByRole(scope: Element | null, role: string, match: NameMatch | null): Element[] {
  return candidates(scope ?? document, role).filter(
    (element) => isShown(element) && nameMatches(accessibleName(element), match),
  );
}

/** The script WebDriver runs with the arguments of `findByRole`, answering its elements. */
export const ROLE_QUERY_SCRIPT = [
  implicitSelector,
  candidates,
  isShown,
  collapse,
  accessibleName,
  nameMatches,
  findByRole,
]
  .map(String)
  .concat('return findByRole(arguments[0], arguments[1], arguments[2]);')
  .join('\n');
