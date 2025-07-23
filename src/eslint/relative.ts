import type { ConfigArray } from 'typescript-eslint';

import noRelativeImportPaths from 'eslint-plugin-no-relative-import-paths';

const relativeConfig: ConfigArray = [
  {
    plugins: {
      'no-relative-import-paths': noRelativeImportPaths,
    },
    rules: {
      'no-relative-import-paths/no-relative-import-paths': [
        `error`,
        { prefix: `@` },
      ],
    },
  },
];

export default relativeConfig;
