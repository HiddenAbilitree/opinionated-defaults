import noRelativeImportPaths from 'eslint-plugin-no-relative-import-paths';

const relativeConfig = [
  {
    plugins: {
      'no-relative-import-paths': noRelativeImportPaths,
    },
    rules: {
      'no-relative-import-paths/no-relative-import-paths': [
        'error',
        { prefix: '@' },
      ],
    },
  },
];

export default relativeConfig;
