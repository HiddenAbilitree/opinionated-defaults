import type { ConfigArray } from 'typescript-eslint';

import eslintPluginOxlint from 'eslint-plugin-oxlint';

const eslintConfigOxlint: ConfigArray = [
  ...eslintPluginOxlint.configs[`flat/recommended`],
];

export default eslintConfigOxlint;
