import type { ConfigArray } from 'typescript-eslint';

import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';

const eslintPrettierConfig: ConfigArray = [
  eslintPluginPrettierRecommended,
  {
    rules: {
      'prettier/prettier': `warn`,
      quotes: [`warn`, `backtick`, { avoidEscape: true }],
    },
  },
];

export default eslintPrettierConfig;
