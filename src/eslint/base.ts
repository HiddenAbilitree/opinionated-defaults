import eslintPluginUnicorn from 'eslint-plugin-unicorn';
import js from '@eslint/js';
import preferArrowFunctions from 'eslint-plugin-prefer-arrow-functions';
// Import noRelativeImportPaths from "eslint-plugin-no-relative-import-paths";
//
const config = [
  js.configs.recommended,
  eslintPluginUnicorn.configs.recommended,
  {
    rules: {
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
  // {
  //   Plugins: {
  //     'no-relative-import-paths': noRelativeImportPaths,
  //   },
  //   Rules: {
  //     'no-relative-import-paths/no-relative-import-paths': [
  //       'error',
  //       { prefix: '@' },
  //     ],
  //   },
  // },
  {
    files: ['**/*.ts', '**/*.js'],
    ignores: ['dist/**'],
  },
];

export default config;
