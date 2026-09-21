import { expect, test } from 'bun:test';

import { oxfmtConfigBase } from '../src/oxfmt';
import { oxlintConfigBase } from '../src/oxlint';

const GENERATED_TS_PATTERN = `**/*.gen.ts`;

test(`oxtools ignore generated TypeScript files`, () => {
  expect(oxlintConfigBase.ignorePatterns).toContain(GENERATED_TS_PATTERN);
  expect(oxfmtConfigBase.ignorePatterns).toContain(GENERATED_TS_PATTERN);
});
