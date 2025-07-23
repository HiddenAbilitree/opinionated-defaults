import {
  eslintConfig,
  eslintConfigBase,
  eslintConfigPerfectionist,
  eslintConfigPrettier,
  eslintConfigTypescript,
} from '@hiddenability/opinionated-defaults/eslint';

export default eslintConfig([
  ...eslintConfigBase,
  ...eslintConfigPrettier,
  ...eslintConfigTypescript,
  ...eslintConfigPerfectionist,
]);
