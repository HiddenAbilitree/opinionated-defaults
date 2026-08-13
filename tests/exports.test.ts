import { expect, test } from 'bun:test';

import * as eslintConfigs from '@hiddenability/opinionated-defaults/eslint';
import eslintConfigAstro from '@hiddenability/opinionated-defaults/eslint/astro';
import eslintConfigBetterTailwindcss from '@hiddenability/opinionated-defaults/eslint/better-tailwindcss';
import eslintConfigFunctional from '@hiddenability/opinionated-defaults/eslint/functional';
import eslintConfigNext from '@hiddenability/opinionated-defaults/eslint/next';
import eslintConfigOxlint from '@hiddenability/opinionated-defaults/eslint/oxlint';
import eslintConfigPerfectionist from '@hiddenability/opinionated-defaults/eslint/perfectionist';
import eslintConfigPrettier from '@hiddenability/opinionated-defaults/eslint/prettier';
import eslintConfigReact from '@hiddenability/opinionated-defaults/eslint/react';
import eslintConfigRelative from '@hiddenability/opinionated-defaults/eslint/relative';
import eslintConfigSolid from '@hiddenability/opinionated-defaults/eslint/solid';
import eslintConfigTurbo from '@hiddenability/opinionated-defaults/eslint/turbo';

test(`isolates optional ESLint configurations behind subpath exports`, () => {
  expect(
    [
      eslintConfigAstro,
      eslintConfigBetterTailwindcss,
      eslintConfigFunctional,
      eslintConfigNext,
      eslintConfigOxlint,
      eslintConfigPerfectionist,
      eslintConfigPrettier,
      eslintConfigReact,
      eslintConfigRelative,
      eslintConfigSolid,
      eslintConfigTurbo,
    ].every(Array.isArray),
  ).toBeTrue();
  expect(Object.keys(eslintConfigs).toSorted()).toEqual([
    `eslintConfig`,
    `eslintConfigBase`,
    `eslintConfigDefaultProject`,
  ]);
  expect(eslintConfigs.eslintConfigBase).toBeArray();
  expect(eslintConfigs.eslintConfigDefaultProject).toBeFunction();
  expect(eslintConfigs.eslintConfig([])).toEqual([]);
});
