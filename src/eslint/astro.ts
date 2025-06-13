import baseConfig from './base';
import eslintPluginAstro from 'eslint-plugin-astro';
import tsConfig from './typescript';

const astroConfig = [
  ...baseConfig,
  ...tsConfig,
  eslintPluginAstro.configs.recommended,
];

export default astroConfig;
