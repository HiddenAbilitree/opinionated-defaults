import {
  eslintConfigBase,
  eslintConfigPrettier,
  eslintConfigTypescript,
} from '@hiddenaility/opinionated-defaults/eslint';

const config = [
  ...eslintConfigBase,
  ...eslintConfigPrettier,
  ...eslintConfigTypescript,
];

export default config;
