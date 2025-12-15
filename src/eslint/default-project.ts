import type { ConfigArray } from 'typescript-eslint';

const config: ConfigArray = [
  {
    languageOptions: {
      parserOptions: {
        projectService: {
          allowDefaultProject: [`eslint.config.ts`, `prettier.config.mjs`],
        },
      },
    },
  },
];

export default config;
