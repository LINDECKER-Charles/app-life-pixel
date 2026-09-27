// What a pull request's change calls for (devops.md, "Continuous integration"), as pure
// functions: the verification jobs skip a change to documentation alone. detect-scope.mjs feeds
// them the changed paths.

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
