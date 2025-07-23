import type { ConfigArray } from 'typescript-eslint';

import baseConfig from './base';
import functionalConfig from './functional';
import relativeConfig from './relative';
import tsConfig from './typescript';

const elysiaConfig: ConfigArray = [
  ...baseConfig,
  ...tsConfig,
  ...functionalConfig,
  ...relativeConfig,
];

export default elysiaConfig;
