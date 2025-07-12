import eslintPluginAstro from 'eslint-plugin-astro';

import baseConfig from './base';
import relativeConfig from './relative';
import tsConfig from './typescript';

const astroConfig = [
  ...baseConfig,
  ...tsConfig,
  ...relativeConfig,
  eslintPluginAstro.configs.recommended,
];

export default astroConfig;
