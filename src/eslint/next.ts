import type { ConfigArray } from 'typescript-eslint';

import pluginNext from '@next/eslint-plugin-next';

type Rules = Record<string, undefined>;

const nextJsConfig: ConfigArray = [
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
