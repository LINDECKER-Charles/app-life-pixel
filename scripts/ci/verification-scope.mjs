// What a pull request's change calls for (devops.md, "Continuous integration"), as pure
// functions: the verification jobs skip a change to documentation alone, and CodeQL analyses
// only the languages whose sources changed. detect-scope.mjs feeds them the changed paths.

/** The languages CodeQL analyses, each with the paths of its sources. */
const CODEQL_SOURCES = {
  actions: [/^\.github\/(workflows|actions)\//],
  rust: [/\.rs$/, /(^|\/)Cargo\.(toml|lock)$/],
  'javascript-typescript': [/\.(ts|tsx|js|jsx|mjs|cjs|html)$/],
};

/** Every language CodeQL analyses: what a push or a scheduled run covers. */
export const CODEQL_LANGUAGES = Object.keys(CODEQL_SOURCES);

/** Documentation: nothing builds, tests or serves it. */
const DOCUMENTATION = [/^docs\//, /\.md$/];

/** Markdown that code reads: the legal pages the server serves and check-i18n compares. */
const MARKDOWN_READ_BY_CODE = /^i18n\//;

/** Whether a change to `path` can alter what the verification jobs check. */
export function needsVerification(path) {
  if (MARKDOWN_READ_BY_CODE.test(path)) {
    return true;
  }
  return !DOCUMENTATION.some((pattern) => pattern.test(path));
}

/** The CodeQL languages whose sources `paths` touch, in the order of CODEQL_LANGUAGES. */
export function codeqlLanguagesFor(paths) {
  return CODEQL_LANGUAGES.filter((language) =>
    paths.some((path) => CODEQL_SOURCES[language].some((pattern) => pattern.test(path))),
  );
}
