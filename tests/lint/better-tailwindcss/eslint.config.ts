import eslintConfigBase from '../../../src/eslint/base';
import eslintConfigBetterTailwindcss from '../../../src/eslint/better-tailwindcss';
import { eslintConfig } from '../../../src/eslint/config';
import eslintConfigDefaultProject from '../../../src/eslint/default-project';
import eslintConfigPerfectionist from '../../../src/eslint/perfectionist';
import eslintConfigPrettier from '../../../src/eslint/prettier';
import eslintConfigReact from '../../../src/eslint/react';

export default eslintConfig([
  ...eslintConfigBase,
  ...eslintConfigReact,
  ...eslintConfigBetterTailwindcss,
  ...eslintConfigPerfectionist,
  ...eslintConfigPrettier,
  ...eslintConfigDefaultProject([`eslint.config.ts`]),
]);
