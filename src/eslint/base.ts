import type { ConfigArray } from 'typescript-eslint';

import js from '@eslint/js';
import preferArrowFunctions from 'eslint-plugin-prefer-arrow-functions';
import eslintPluginUnicorn from 'eslint-plugin-unicorn';

// eslint-plugin-no-relative-import-paths is not used here because
// jiti does not support path aliases and this project uses these rules.
// https://github.com/unjs/jiti/issues/373

const config: ConfigArray = [
  {
    ignores: [`**/dist/`, `**/node_modules/`, `**/.direnv/`, `**/.git/`],
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
  js.configs.recommended,
  eslintPluginUnicorn.configs.recommended,
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
    rules: {
      'no-console': `warn`,
      quotes: [`warn`, `backtick`, { avoidEscape: true }],
    },
  },
];

export default config;
