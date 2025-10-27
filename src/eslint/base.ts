import js from '@eslint/js';
import stylistic from '@stylistic/eslint-plugin';
import parser from '@typescript-eslint/parser';
import preferArrowFunctions from 'eslint-plugin-prefer-arrow-functions';
import eslintPluginUnicorn from 'eslint-plugin-unicorn';
import globals from 'globals';
import tseslint, { type ConfigArray } from 'typescript-eslint';

// eslint-plugin-no-relative-import-paths is not used here because
// jiti does not support path aliases and this project uses these rules.
// https://github.com/unjs/jiti/issues/373

const config: ConfigArray = [
  {
    ignores: [`**/dist/`, `**/node_modules/`, `**/.git/`],
  },
  {
    files: [
      `**/*.ts`,
      `**/*.js`,
      `**/*.mjs`,
      `**/*.mts`,
      `**/*.tsx`,
      `**/*.jsx`,
    ],
  },
  ...tseslint.configs.recommendedTypeChecked,
  js.configs.recommended,
  eslintPluginUnicorn.configs.recommended,
  {
    languageOptions: {
      globals: {
        ...globals.nodeBuiltin,
        ...globals.browser,
      },
      parser,
      parserOptions: {
        projectService: true,
      },
    },
    rules: {
      '@typescript-eslint/no-misused-promises': [
        `error`,
        {
          checksVoidReturn: false,
        },
      ],
      '@typescript-eslint/no-unused-vars': [
        `error`,
        {
          args: `all`,
          argsIgnorePattern: `^_`,
          caughtErrors: `all`,
          caughtErrorsIgnorePattern: `^_`,
          destructuredArrayIgnorePattern: `^_`,
          ignoreRestSiblings: true,
          varsIgnorePattern: `^_`,
        },
      ],
      '@typescript-eslint/prefer-nullish-coalescing': `error`,
      'no-unused-vars': [
        `error`,
        {
          args: `all`,
          argsIgnorePattern: `^_`,
          caughtErrors: `all`,
          caughtErrorsIgnorePattern: `^_`,
          destructuredArrayIgnorePattern: `^_`,
          ignoreRestSiblings: true,
          varsIgnorePattern: `^_`,
        },
      ],
    },
  },
  {
    rules: {
      'unicorn/filename-case': [
        `error`,
        {
          cases: {
            kebabCase: true,
          },
        },
      ],
      'unicorn/no-array-for-each': [`off`],
      'unicorn/no-array-reduce': [`off`],
      'unicorn/prevent-abbreviations': [`off`],
    },
  },
  {
    plugins: {
      'prefer-arrow-functions': preferArrowFunctions,
    },
    rules: {
      'prefer-arrow-functions/prefer-arrow-functions': [
        `error`,
        { returnStyle: `implicit` },
      ],
    },
  },
  {
    plugins: {
      '@stylistic': stylistic,
    },
    rules: {
      '@stylistic/quotes': [`error`, `backtick`],
    },
  },
];

export default config;
