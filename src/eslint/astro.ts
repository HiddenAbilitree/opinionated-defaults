import baseConfig from './base';
import eslintPluginAstro from 'eslint-plugin-astro';
import tsConfig from './typescript';
import relativeConfig from './relative';

const astroConfig = [
  ...baseConfig,
  ...tsConfig,
  ...relativeConfig,
  eslintPluginAstro.configs.recommended,
];

export default astroConfig;
