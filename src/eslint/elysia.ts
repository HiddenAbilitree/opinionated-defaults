import baseConfig from './base';
import tsConfig from './typescript';
import functionalConfig from './functional';
import relativeConfig from './relative';

const elysiaConfig = [
  ...baseConfig,
  ...tsConfig,
  ...functionalConfig,
  ...relativeConfig,
];

export default elysiaConfig;
