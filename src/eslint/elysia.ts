import type { ConfigArray } from 'typescript-eslint';

import baseConfig from './base';
import functionalConfig from './functional';
import relativeConfig from './relative';

const elysiaConfig: ConfigArray = [
  ...baseConfig,
  ...functionalConfig,
  ...relativeConfig,
];

export default elysiaConfig;
