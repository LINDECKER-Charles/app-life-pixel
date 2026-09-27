// Colours come from the semantic tokens of `_tokens.scss` (CLAUDE.md, "Interface"): a stylesheet
// names a token, never a literal colour. A justified exception carries a
// `stylelint-disable-next-line <rule> -- <why>` comment; a bare one is refused.

const COLOUR_FUNCTIONS = [
  'rgb',
  'rgba',
  'hsl',
  'hsla',
  'hwb',
  'lab',
  'lch',
  'oklab',
  'oklch',
  'color',
];

/** @type {import('stylelint').Config} */
export default {
  customSyntax: 'postcss-scss',
  ignoreFiles: ['projects/shared/src/styles/_tokens.scss'],
  reportDescriptionlessDisables: true,
  reportInvalidScopeDisables: true,
  reportNeedlessDisables: true,
  rules: {
    'color-named': 'never',
    'color-no-hex': true,
    'function-disallowed-list': COLOUR_FUNCTIONS,
  },
};
