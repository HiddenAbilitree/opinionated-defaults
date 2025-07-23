import type { ConfigArray } from 'typescript-eslint';

import eslintPluginAstro from 'eslint-plugin-astro';

import baseConfig from './base';
import relativeConfig from './relative';
import tsConfig from './typescript';

const astroConfig: ConfigArray = [
  ...baseConfig,
  ...tsConfig,
  ...relativeConfig,
  ...eslintPluginAstro.configs.recommended,
];

export default astroConfig;
