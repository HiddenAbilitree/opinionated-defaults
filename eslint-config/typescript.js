import parser from '@typescript-eslint/parser';

/**
 *  @type {import("eslint").Linter.Config[]}
 * */
const tsConfig = [{
  languageOptions: {
    parser,
    parserOptions: {
      projectService: true,
    },
  },
}
];

export default tsConfig;
