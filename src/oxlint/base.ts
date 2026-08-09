import type { OxlintConfig } from 'oxlint';

const config: OxlintConfig = {
  categories: {
    correctness: `error`,
    suspicious: `warn`,
  },
  env: {
    browser: true,
    es6: true,
    node: true,
  },
  ignorePatterns: [`**/dist/`, `**/node_modules/`, `**/.git/`, `**/*.gen.ts`],
  options: {
    typeAware: true,
    typeCheck: true,
  },
  plugins: [
    `typescript`,
    `unicorn`,
    `oxc`,
    `import`,
    `promise`,
    `node`,
    `jsx-a11y`,
  ],
  rules: {
    'no-unassigned-import': [
      `warn`,
      {
        allow: [`**/*.css`, `**/*.scss`, `**/*.less`],
      },
    ],
    'no-unused-vars': [
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
    'react/react-in-jsx-scope': `off`,
    'typescript/no-misused-promises': [
      `error`,
      {
        checksVoidReturn: false,
      },
    ],
    'typescript/prefer-nullish-coalescing': `error`,
    'unicorn/filename-case': [
      `error`,
      {
        cases: {
          kebabCase: true,
        },
      },
    ],
    'unicorn/no-array-for-each': `off`,
    'unicorn/no-array-reduce': `off`,
  },
};

export default config;
