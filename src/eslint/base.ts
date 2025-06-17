import js from '@eslint/js';
import preferArrowFunctions from 'eslint-plugin-prefer-arrow-functions';
import eslintPluginUnicorn from 'eslint-plugin-unicorn';

// eslint-plugin-no-relative-import-paths is not used here because
// jiti does not support path aliases and this project uses these rules.
// https://github.com/unjs/jiti/issues/373

const config = [
  js.configs.recommended,
  eslintPluginUnicorn.configs.recommended,
  {
    rules: {
      'unicorn/prevent-abbreviations': ['off'],
      'unicorn/filename-case': [
        'error',
        {
          cases: {
            kebabCase: true,
          },
        },
      ],
    },
  },
  {
    plugins: {
      'prefer-arrow-functions': preferArrowFunctions,
    },
    rules: {
      'prefer-arrow-functions/prefer-arrow-functions': [
        'error',
        { returnStyle: 'implicit' },
      ],
    },
  },
  {
    files: [
      '**/*.ts',
      '**/*.js',
      '**/*.mjs',
      '**/*.mts',
      '**/*.tsx',
      '**/*.jsx',
    ],
  },
  {
    ignores: ['dist/'],
  },
];

export default config;
