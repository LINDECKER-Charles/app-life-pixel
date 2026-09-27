// Checks a pull request against the conventions of CLAUDE.md: its title, its branch's name, and
// the size of its change. Run by the `pull-request` workflow, which passes the pull request in
// PR_TITLE, PR_BRANCH, BASE_SHA and HEAD_SHA; prints GitHub annotations; fails on a broken
// convention, and only warns about the size.

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import {
  REVIEWABLE_CHANGED_LINES,
  checkBranch,
  checkTitle,
  countReviewedLines,
  readScopes,
} from './pull-request-rules.mjs';

const REQUIRED_VARIABLES = ['PR_TITLE', 'PR_BRANCH', 'BASE_SHA', 'HEAD_SHA'];

function readEnvironment() {
  const missing = REQUIRED_VARIABLES.filter((name) => !process.env[name]);
  if (missing.length > 0) {
    throw new Error(`missing environment variable(s): ${missing.join(', ')}`);
  }
  return Object.fromEntries(REQUIRED_VARIABLES.map((name) => [name, process.env[name]]));
}

function changedLines(baseSha, headSha) {
  const numstat = execFileSync('git', ['diff', '--numstat', `${baseSha}...${headSha}`], {
    encoding: 'utf8',
  });
  return countReviewedLines(numstat);
}

function report(level, subject, problems) {
  for (const problem of problems) {
    console.log(`::${level} title=${subject}::${problem}`);
  }
}

function main() {
  const { PR_TITLE, PR_BRANCH, BASE_SHA, HEAD_SHA } = readEnvironment();
  const scopes = readScopes(readFileSync('CLAUDE.md', 'utf8'));
  const titleProblems = checkTitle(PR_TITLE, scopes);
  const branchProblems = checkBranch(PR_BRANCH);
  report('error', 'Pull request title', titleProblems);
  report('error', 'Branch name', branchProblems);

  const lines = changedLines(BASE_SHA, HEAD_SHA);
  console.log(`${lines} reviewed line(s) changed, generated and vendored files aside`);
  if (lines > REVIEWABLE_CHANGED_LINES) {
    const advice = `split it below ${REVIEWABLE_CHANGED_LINES} so that it can be reviewed closely`;
    report('warning', 'Pull request size', [`${lines} lines changed: ${advice}`]);
  }
  process.exitCode = titleProblems.length + branchProblems.length > 0 ? 1 : 0;
}

main();
