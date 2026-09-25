// @ts-check
const eslint = require('@eslint/js');
const { defineConfig } = require('eslint/config');
const tseslint = require('typescript-eslint');
const angular = require('angular-eslint');

// The size and complexity limits of AGENTS.md.
const sizeRules = {
  'max-lines': ['error', 400],
  'max-lines-per-function': ['error', { max: 30, skipBlankLines: true, skipComments: true }],
};
const complexityRules = {
  'max-params': ['error', 3],
  'max-depth': ['error', 3],
  complexity: ['error', 10],
};

module.exports = defineConfig([
  {
    files: ['**/*.ts'],
    extends: [
      eslint.configs.recommended,
      tseslint.configs.recommended,
      tseslint.configs.stylistic,
      angular.configs.tsRecommended,
    ],
    processor: angular.processInlineTemplates,
    rules: {
      ...sizeRules,
      ...complexityRules,
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@angular-eslint/directive-selector': [
        'error',
        { type: 'attribute', prefix: 'lp', style: 'camelCase' },
      ],
      '@angular-eslint/component-selector': [
        'error',
        { type: 'element', prefix: 'lp', style: 'kebab-case' },
      ],
      '@angular-eslint/prefer-on-push-component-change-detection': 'error',
    },
  },
  {
    // engine/testing/ is test code (editor.md, W0): production files get the engine through
    // EDITOR_ENGINE and EngineStore, never the mock directly. editor-engine.ts is the one
    // production file allowed in, since it is where EDITOR_ENGINE provides the mock until W1.
    files: ['**/*.ts'],
    ignores: ['**/*.spec.ts', '**/engine/testing/**', '**/engine/editor-engine.ts'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['**/engine/testing', '**/engine/testing/*'],
              message: 'engine/testing is test code: import EDITOR_ENGINE or EngineStore instead.',
            },
          ],
        },
      ],
    },
  },
  {
    files: ['**/*.html'],
    extends: [angular.configs.templateRecommended, angular.configs.templateAccessibility],
    rules: {
      '@angular-eslint/template/prefer-control-flow': 'error',
    },
  },
  {
    files: ['tools/**/*.mjs'],
    extends: [eslint.configs.recommended],
    languageOptions: {
      sourceType: 'module',
      globals: { console: 'readonly', process: 'readonly', URL: 'readonly' },
    },
    rules: { ...sizeRules, ...complexityRules },
  },
  {
    // engine-contract.ts defines the shared suite as nested `describe`/`it` blocks: like a spec
    // file, its size is the suite's, not a single function's.
    files: [
      '**/*.spec.ts',
      'e2e/**/*.ts',
      'tools/**/*.test.mjs',
      '**/engine/testing/engine-contract.ts',
    ],
    rules: { 'max-lines': 'off', 'max-lines-per-function': 'off' },
  },
]);
