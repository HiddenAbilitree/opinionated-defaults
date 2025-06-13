import eslintPluginAstro from 'eslint-plugin-astro';
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';
import eslintPluginUnicorn from 'eslint-plugin-unicorn';
import noRelativeImportPaths from "eslint-plugin-no-relative-import-paths";
import preferArrowFunctions from 'eslint-plugin-prefer-arrow-functions';

/**
 * @type {import("eslint").Linter.Config[]}
 * */
const astroConfig = [
  ...eslintPluginAstro.configs.recommended,
  eslintConfigPrettier,
  eslintPluginPrettierRecommended,
  eslintPluginUnicorn.configs.recommended,
  {
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
    },
  },
  {
    plugins: {
      'no-relative-import-paths': noRelativeImportPaths,
      'prefer-arrow-functions': preferArrowFunctions,
    },
    rules: {
      'prefer-arrow-functions/prefer-arrow-functions': [
        'error',
        { returnStyle: 'unchanged' },
      ],
      'no-relative-import-paths/no-relative-import-paths': [
        'error',
        { prefix: '@' },
      ],
      'unicorn/prevent-abbreviations': ['off'],
      'unicorn/filename-case': [
        'error',
        {
          cases: {
            pascalCase: true,
            kebabCase: true,
          },
        },
      ],
    }
  }
];

export default astroConfig
