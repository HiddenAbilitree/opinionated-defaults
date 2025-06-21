import parser from '@typescript-eslint/parser';
import tseslint from 'typescript-eslint';
import { type Config } from 'typescript-eslint';

const typescriptConfig: Config = [
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
