import baseConfig from './base.ts';
import eslintPluginAstro from 'eslint-plugin-astro';
import tsConfig from './typescript.ts';

const astroConfig = [
  ...baseConfig,
  ...tsConfig,
  eslintPluginAstro.configs.recommended,
];

export default astroConfig;
