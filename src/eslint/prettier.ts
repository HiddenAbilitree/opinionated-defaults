import type { ConfigArray } from 'typescript-eslint';

import stylistic from '@stylistic/eslint-plugin';
import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';

const eslintPrettierConfig: ConfigArray = [
  eslintPluginPrettierRecommended,
  {
    plugins: {
      '@stylistic': stylistic,
    },
    rules: {
      '@stylistic/quotes': ['warn', 'backtick', { avoidEscape: true }],
      'prettier/prettier': `warn`,
    },
  },
];

export default eslintPrettierConfig;
