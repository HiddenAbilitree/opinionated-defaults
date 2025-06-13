import parser from '@typescript-eslint/parser';
import tseslint from 'typescript-eslint';

const typescriptConfig = [
  ...tseslint.configs.recommended,
  {
    languageOptions: {
      parser,
      parserOptions: {
        projectService: true,
      },
    },
  },
];

export default typescriptConfig;
