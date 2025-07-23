import type { ConfigArray } from 'typescript-eslint';

import eslintPluginAstro from 'eslint-plugin-astro';

import baseConfig from './base';
import relativeConfig from './relative';
import tsConfig from './typescript';

type ArrayElementType<T> = T extends (infer U)[] ? U : never;

const astroConfig: ConfigArray = [
  ...baseConfig,
  ...tsConfig,
  ...relativeConfig,
  eslintPluginAstro.configs.recommended as ArrayElementType<ConfigArray>,
];

export default astroConfig;
