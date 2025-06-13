import {
  eslintConfigBase,
  eslintConfigPrettier,
  eslintConfigTypescript,
} from '@hiddenability/opinionated-defaults/eslint';

const config = [
  ...eslintConfigBase,
  ...eslintConfigPrettier,
  ...eslintConfigTypescript,
];

export default config;
