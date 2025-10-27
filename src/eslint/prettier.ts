import type { ConfigArray } from 'typescript-eslint';

import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';

const eslintPrettierConfig: ConfigArray = [
  eslintPluginPrettierRecommended,
  {
    rules: {
      'prettier/prettier': `warn`,
    },
  },
];

export default eslintPrettierConfig;
