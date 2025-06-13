import {
  eslintBaseConfig,
  eslintPrettierConfig,
  eslintTypescriptConfig,
} from 'reasonable-defaults/eslint';

const config = [
  ...eslintBaseConfig,
  ...eslintPrettierConfig,
  ...eslintTypescriptConfig,
];

export default config;
