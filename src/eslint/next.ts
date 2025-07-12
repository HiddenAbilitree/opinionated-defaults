import pluginNext from '@next/eslint-plugin-next';

import baseConfig from './base';
import reactConfig from './react';
import relativeConfig from './relative';
import tsConfig from './typescript';

const nextJsConfig = [
  ...baseConfig,
  ...reactConfig,
  ...relativeConfig,
  ...tsConfig,
  {
    plugins: {
      '@next/next': pluginNext,
    },
    rules: {
      ...pluginNext.configs.recommended.rules,
      ...pluginNext.configs['core-web-vitals'].rules,
    },
  },
];

export default nextJsConfig;
