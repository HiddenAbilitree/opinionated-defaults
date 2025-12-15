import eslintConfigBase from '../../src/eslint/base';
import { eslintConfig } from '../../src/eslint/config';
import eslintConfigPerfectionist from '../../src/eslint/perfectionist';
import eslintConfigPrettier from '../../src/eslint/prettier';

export default eslintConfig([
  ...eslintConfigBase,
  ...eslintConfigPerfectionist,
  ...eslintConfigPrettier,
]);
