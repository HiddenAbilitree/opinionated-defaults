import { expect, test } from 'bun:test';

import {
  oxfmtConfig,
  oxfmtConfigBase,
  oxfmtConfigNext,
  oxfmtConfigTanstackStart,
} from '../src/oxfmt';

test(`oxfmt config composes typed config modules`, () => {
  const config = oxfmtConfig(oxfmtConfigBase, {
    printWidth: 100,
    sortTailwindcss: {
      functions: [`cx`],
    },
  });

  expect(config.printWidth).toBe(100);
  expect(config.sortImports).toEqual({ newlinesBetween: true });
  expect(config.sortTailwindcss).toEqual({ functions: [`cx`] });
});

test(`oxfmt framework modules own generated-file ignore options`, () => {
  expect(oxfmtConfigBase.ignorePatterns).toContain(`**/*.gen.ts`);
  expect(oxfmtConfigBase.ignorePatterns).not.toContain(`**/.next/`);
  for (const pattern of [`**/.next/`, `**/out/`, `**/next-env.d.ts`]) {
    expect(oxfmtConfigNext.ignorePatterns).toContain(pattern);
  }
  for (const pattern of [`**/.output/`, `**/.nitro/`, `**/.tanstack/`]) {
    expect(oxfmtConfigTanstackStart.ignorePatterns).toContain(pattern);
  }
});
