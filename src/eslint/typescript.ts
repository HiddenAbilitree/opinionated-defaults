import parser from '@typescript-eslint/parser';
import tseslint, { type ConfigArray } from 'typescript-eslint';

const typescriptConfig: ConfigArray = [
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parser,
      parserOptions: {
        projectService: true,
      },
    },
  },
  {
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
    },
  },
];

export default typescriptConfig;
