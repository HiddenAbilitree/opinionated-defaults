import pluginNext from '@next/eslint-plugin-next';
import baseConfig from './base.ts';
import tsConfig from './typescript.ts';
import reactConfig from './react.ts';

const nextJsConfig = [
  ...baseConfig,
  ...reactConfig,
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
