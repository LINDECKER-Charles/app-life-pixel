// @ts-check
import eslint from '@eslint/js';
import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';

// The size and complexity limits of AGENTS.md.
const limits = {
  'max-lines': ['error', 400],
  'max-lines-per-function': ['error', { max: 30, skipBlankLines: true, skipComments: true }],
  'max-params': ['error', 3],
  'max-depth': ['error', 3],
  complexity: ['error', 10],
};

export default defineConfig(
  { ignores: ['node_modules/'] },
  {
    files: ['**/*.ts'],
    extends: [eslint.configs.recommended, tseslint.configs.strictTypeChecked],
    languageOptions: { parserOptions: { projectService: true } },
    rules: limits,
  },
  {
    files: ['**/*.js'],
    extends: [eslint.configs.recommended],
  },
);
