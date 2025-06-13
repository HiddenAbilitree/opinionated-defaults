import {
  eslintBaseConfig,
  eslintPrettierConfig,
  eslintTypescriptConfig,
} from './src/index.ts';

const config = [
  ...eslintBaseConfig,
  ...eslintPrettierConfig,
  ...eslintTypescriptConfig,
];

export default config;
