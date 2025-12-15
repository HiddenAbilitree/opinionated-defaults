import type { ConfigArray } from 'typescript-eslint';

type DefaultProjectOptions = {
  allowDefaultProject: string[];
  defaultProject?: string;
};

const eslintConfigDefaultProject = (
  filesOrOptions: DefaultProjectOptions | string[],
): ConfigArray => {
  const options: DefaultProjectOptions =
    Array.isArray(filesOrOptions) ?
      { allowDefaultProject: filesOrOptions }
    : filesOrOptions;

  return options.allowDefaultProject.length === 0 ?
      []
    : [
        {
          languageOptions: {
            parserOptions: {
              projectService: {
                allowDefaultProject: options.allowDefaultProject,
                ...(options.defaultProject && {
                  defaultProject: options.defaultProject,
                }),
              },
            },
          },
        },
      ];
};

export default eslintConfigDefaultProject;
