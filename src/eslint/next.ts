import pluginNext from '@next/eslint-plugin-next';
import type { ConfigArray } from 'typescript-eslint';

const nextJsConfig: ConfigArray = [
  {
    ignores: [`**/next-env.d.ts`],
  },
  {
    plugins: {
      '@next/next': pluginNext,
    },
    rules: {
      ...pluginNext.configs.recommended.rules,
      ...pluginNext.configs[`core-web-vitals`].rules,
    },
  },
];

export default nextJsConfig;
