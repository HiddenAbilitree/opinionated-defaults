import js from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';
import turboPlugin from "eslint-plugin-turbo";
import tseslint from "typescript-eslint";
import eslintPluginUnicorn from 'eslint-plugin-unicorn';
import preferArrowFunctions from 'eslint-plugin-prefer-arrow-functions';
import noRelativeImportPaths from "eslint-plugin-no-relative-import-paths";
import stylisticJs from '@stylistic/eslint-plugin'

/**
 * @type {import("eslint").Linter.Config[]}
 * */
const config = [
  js.configs.recommended,
  eslintConfigPrettier,
  eslintPluginPrettierRecommended,
  ...tseslint.configs.recommended,
  eslintPluginUnicorn.configs.recommended,
  {
    plugins: {
      turbo: turboPlugin,
    },
    rules: {
      "turbo/no-undeclared-env-vars": "error",
    },
  },
  {
    plugins: {
      'no-relative-import-paths': noRelativeImportPaths,
      'prefer-arrow-functions': preferArrowFunctions,
      '@stylistic': stylisticJs,
    },
    rules: {
      // '@stylistic/quotes': ['error', 'single'],
      '@stylistic/jsx-curly-brace-presence': ["error", "never"],
      'prefer-arrow-functions/prefer-arrow-functions': [
        'error',
        { returnStyle: 'unchanged' },
      ],
      'no-relative-import-paths/no-relative-import-paths': [
        'error',
        { prefix: '@' },
      ],
      'unicorn/no-array-for-each': ['off'],
      'unicorn/prevent-abbreviations': ['off'],
      'unicorn/filename-case': [
        'error',
        {
          cases: {
            kebabCase: true,
          },
        },
      ],
    }
  },
  {
    ignores: ["dist/**"],
  },
];

export default config;
