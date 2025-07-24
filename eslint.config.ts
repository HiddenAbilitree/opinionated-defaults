import { includeIgnoreFile } from '@eslint/compat';
import {
  eslintConfig,
  eslintConfigBase,
  eslintConfigOxlint,
  eslintConfigPerfectionist,
  eslintConfigPrettier,
  eslintConfigTypescript,
} from '@hiddenability/opinionated-defaults/eslint';
import { fileURLToPath } from 'node:url';

const gitignorePath = fileURLToPath(new URL(`.gitignore`, import.meta.url));

export default eslintConfig([
  includeIgnoreFile(gitignorePath, `Imported .gitignore patterns`),
  ...eslintConfigBase,
  ...eslintConfigPrettier,
  ...eslintConfigTypescript,
  ...eslintConfigPerfectionist,
  ...eslintConfigOxlint,
]);
