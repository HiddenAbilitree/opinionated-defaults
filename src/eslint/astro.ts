import baseConfig from './base';
import relativeConfig from './relative';
import tsConfig from './typescript';
import eslintPluginAstro from 'eslint-plugin-astro';

const astroConfig = [
  ...baseConfig,
  ...tsConfig,
  ...relativeConfig,
  eslintPluginAstro.configs.recommended,
];

export default astroConfig;
