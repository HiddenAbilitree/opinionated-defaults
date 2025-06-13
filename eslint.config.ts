import {
  eslintBaseConfig,
  eslintPrettierConfig,
  eslintTypescriptConfig,
} from './dist/index.mjs';

const config = [
  ...eslintBaseConfig,
  ...eslintPrettierConfig,
  ...eslintTypescriptConfig,
];

export default config;
