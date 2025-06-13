import {
  eslintConfigBase,
  eslintConfigPrettier,
  eslintConfigTypescript,
} from 'reasonable-defaults/eslint';

const config = [
  ...eslintConfigBase,
  ...eslintConfigPrettier,
  ...eslintConfigTypescript,
];

export default config;
