import stylistic from '@stylistic/eslint-plugin';
import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';
import type { ConfigArray } from 'typescript-eslint';

import prettierConfigBase from '../prettier/base';

const eslintPrettierConfig: ConfigArray = [
  eslintPluginPrettierRecommended,
  {
    plugins: {
      '@stylistic': stylistic,
    },
    rules: {
      '@stylistic/quotes': [`warn`, `backtick`, { avoidEscape: true }],
      'prettier/prettier': [`warn`, prettierConfigBase],
    },
  },
];

export default eslintPrettierConfig;
