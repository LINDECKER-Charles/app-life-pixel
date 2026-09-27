// The conventions of a pull request (CLAUDE.md, "Workflow" and "commit"), as pure functions.
// The pull request's title becomes the squash commit's subject, so it follows the commit
// convention; the scopes are read from CLAUDE.md, their single source.

export const COMMIT_TYPES = [
  'feat',
  'fix',
  'docs',
  'refactor',
  'perf',
  'test',
  'style',
  'build',
  'ci',
  'chore',
  'revert',
];
const TYPE_LIST = COMMIT_TYPES.join(', ');

export const MAX_SUBJECT_LENGTH = 72;

// Beyond this many changed lines, outside generated and vendored files, a review stops being
// thorough: the pull request is flagged, not refused.
export const REVIEWABLE_CHANGED_LINES = 600;

// Files a reviewer does not read line by line: lockfiles, generated code, golden files.
const UNREVIEWED_PATHS = [
  /(^|\/)Cargo\.lock$/,
  /(^|\/)package-lock\.json$/,
  /^crates\/(server|admin-server)\/openapi\.json$/,
  /(^|\/)schema\.d\.ts$/,
  /^player-js\/life-pixel\.js$/,
  /\/tests\/(golden|fixtures)\//,
];

const SCOPE_IN_COMMIT_MAP = /→ `([a-z][a-z0-9-]*)`/g;
const SUBJECT = /^(?<type>[a-z]+)(?:\((?<scope>[^)]*)\))?!?: (?<description>.+)$/;
const BRANCH = /^(?<type>[a-z]+)\/(?<description>[a-z0-9]+(?:-[a-z0-9]+)*)$/;
const MIN_BRANCH_WORDS = 2;
const MAX_BRANCH_WORDS = 5;

/** The scopes of the `## commit` section of CLAUDE.md. */
export function readScopes(claudeMd) {
  const section = claudeMd.split(/^## /m).find((part) => /^commit\r?\n/.test(part)) ?? '';
  return [...new Set([...section.matchAll(SCOPE_IN_COMMIT_MAP)].map((match) => match[1]))];
}

/** What is wrong with `title` as a Conventional Commits subject; empty when it is sound. */
export function checkTitle(title, scopes) {
  const problems = [];
  if (title.length > MAX_SUBJECT_LENGTH) {
    problems.push(`the title has ${title.length} characters, ${MAX_SUBJECT_LENGTH} at most`);
  }
  const match = SUBJECT.exec(title);
  if (!match?.groups) {
    return [...problems, 'the title does not read `type(scope): description`'];
  }
  const { type, scope, description } = match.groups;
  if (!COMMIT_TYPES.includes(type)) {
    problems.push(`\`${type}\` is not a commit type: ${TYPE_LIST}`);
  }
  if (scope !== undefined && !scopes.includes(scope)) {
    problems.push(`\`${scope}\` is not a scope of CLAUDE.md: ${scopes.join(', ')}`);
  }
  if (/^[A-Z]/.test(description)) {
    problems.push('the description must start with a lowercase letter');
  }
  if (description.endsWith('.')) {
    problems.push('the description must not end with a period');
  }
  return problems;
}

/** What is wrong with `branch` as a `type/short-description` name; empty when it is sound. */
export function checkBranch(branch) {
  const match = BRANCH.exec(branch);
  if (!match?.groups) {
    return ['the branch is not named `type/short-description`, in kebab-case without accents'];
  }
  const { type, description } = match.groups;
  const problems = [];
  if (!COMMIT_TYPES.includes(type)) {
    problems.push(`\`${type}/\` is not a commit type in its short form: ${TYPE_LIST}`);
  }
  const wordCount = description.split('-').length;
  if (wordCount < MIN_BRANCH_WORDS || wordCount > MAX_BRANCH_WORDS) {
    const expected = `${MIN_BRANCH_WORDS} to ${MAX_BRANCH_WORDS}`;
    problems.push(`the description has ${wordCount} word(s), ${expected} expected`);
  }
  return problems;
}

/** The lines a reviewer reads in `git diff --numstat` output, binary and generated files aside. */
export function countReviewedLines(numstat) {
  return numstat
    .split('\n')
    .map((line) => line.split('\t'))
    .filter(([added, , path]) => path && added !== '-')
    .filter(([, , path]) => !UNREVIEWED_PATHS.some((pattern) => pattern.test(path)))
    .reduce((total, [added, deleted]) => total + Number(added) + Number(deleted), 0);
}
