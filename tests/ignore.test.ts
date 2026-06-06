import { expect, test } from 'bun:test';
import { ESLint } from 'eslint';
import { fileURLToPath } from 'node:url';
import { getFileInfo } from 'prettier';

import eslintConfigBase from '../src/eslint/base';
import { oxfmtConfig } from '../src/oxfmt';
import { oxlintConfig } from '../src/oxlint';

const GENERATED_TS_PATTERN = `**/*.gen.ts`;

test(`eslint base ignores generated TypeScript files`, async () => {
  expect(
    eslintConfigBase.some(
      (config) =>
        `ignores` in config && config.ignores?.includes(GENERATED_TS_PATTERN),
    ),
  ).toBe(true);

  const eslint = new ESLint({
    overrideConfigFile: fileURLToPath(
      new URL(`lint/eslint.config.ts`, import.meta.url),
    ),
  });

  const isIgnored = await eslint.isPathIgnored(
    fileURLToPath(new URL(`lint/generated.gen.ts`, import.meta.url)),
  );

  expect(isIgnored).toBe(true);
});

test(`prettier ignores generated TypeScript files`, async () => {
  const fileInfo = await getFileInfo(
    fileURLToPath(new URL(`fixtures/generated.gen.ts`, import.meta.url)),
    { ignorePath: `.gitignore` },
  );

  expect(fileInfo).toMatchObject({ ignored: true });
});

test(`oxtools ignore generated TypeScript files`, () => {
  expect(oxlintConfig.ignorePatterns).toContain(GENERATED_TS_PATTERN);
  expect(oxfmtConfig.ignorePatterns).toContain(GENERATED_TS_PATTERN);
});
