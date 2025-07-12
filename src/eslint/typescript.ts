import parser from '@typescript-eslint/parser';
import tseslint from 'typescript-eslint';

const typescriptConfig: object[] = [
  ...tseslint.configs.recommended,
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
    },
  },
];

export default typescriptConfig;
