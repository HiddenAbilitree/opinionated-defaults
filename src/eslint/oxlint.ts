import eslintPluginOxlint from 'eslint-plugin-oxlint';
import type { ConfigArray } from 'typescript-eslint';

const eslintConfigOxlint: ConfigArray = [...eslintPluginOxlint.configs[`flat/recommended`]];

export default eslintConfigOxlint;
