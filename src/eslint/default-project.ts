import type { ConfigArray } from 'typescript-eslint';

const eslintConfigDefaultProject = (files: string[]): ConfigArray =>
  files.length === 0 ?
    []
  : [
      {
        languageOptions: {
          parserOptions: {
            projectService: {
              allowDefaultProject: files,
            },
          },
        },
      },
    ];

export default eslintConfigDefaultProject;
