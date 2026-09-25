// The rules the legal pages of i18n/legal/ follow (docs/i18n.md, accounts.md's H16): every
// language holds the three pages, each using the same placeholders as every other language's.
const PLACEHOLDER_PATTERN = /\{\{(\w+)\}\}/g;

/** The pages every language must hold under `legal/<code>/`. */
export const LEGAL_PAGES = ['terms', 'privacy', 'notice'];

/** The distinct placeholder names a legal page's text uses, sorted. */
export function legalPlaceholders(text) {
  return [...new Set([...text.matchAll(PLACEHOLDER_PATTERN)].map(([, name]) => name))].sort();
}

/**
 * Checks that every language of `pages` holds each of `LEGAL_PAGES`, and that a page's
 * placeholders are the same across languages.
 *
 * `pages` maps a language code to a map of page name to its text, `undefined` when the file is
 * missing.
 */
export function checkLegalPages(pages) {
  const codes = Object.keys(pages);
  return LEGAL_PAGES.flatMap((page) => checkPage(page, codes, pages));
}

function checkPage(page, codes, pages) {
  const problems = [];
  let reference;
  for (const code of codes) {
    const text = pages[code][page];
    const relative = `legal/${code}/${page}.md`;
    if (text === undefined) {
      problems.push(`${relative}: missing`);
      continue;
    }
    const placeholders = legalPlaceholders(text);
    if (reference === undefined) {
      reference = { code, placeholders };
    } else if (placeholders.join(',') !== reference.placeholders.join(',')) {
      problems.push(
        `${relative}: placeholders [${describe(placeholders)}] differ from ` +
          `legal/${reference.code}/${page}.md's [${describe(reference.placeholders)}]`,
      );
    }
  }
  return problems;
}

function describe(placeholders) {
  return placeholders.map((name) => `{{${name}}}`).join(', ');
}
