import type { ConfigArray } from 'typescript-eslint';

import pluginNext from '@next/eslint-plugin-next';

import baseConfig from './base';
import reactConfig from './react';
import relativeConfig from './relative';
import tsConfig from './typescript';

type Rules = Record<string, undefined>;

const nextJsConfig: ConfigArray = [
  ...baseConfig,
  ...reactConfig,
  ...relativeConfig,
  ...tsConfig,
  {
    plugins: {
      '@next/next': pluginNext,
    },
    rules: {
      ...(pluginNext.configs.recommended.rules as unknown as Rules),
      ...(pluginNext.configs[`core-web-vitals`].rules as unknown as Rules),
    },
  },
];

export default nextJsConfig;
