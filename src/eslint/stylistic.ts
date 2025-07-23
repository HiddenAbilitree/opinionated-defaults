import type { ConfigArray } from 'typescript-eslint';

import stylistic from '@stylistic/eslint-plugin';

const stylisticConfig: ConfigArray = [
  stylistic.configs.all,
  {
    plugins: {
      '@stylistic': stylistic,
    },
  },
];

export default stylisticConfig;
