import { expect, test } from 'bun:test';

import {
  oxlintConfig,
  oxlintConfigBase,
  oxlintConfigNext,
  oxlintConfigReact,
  oxlintConfigTanstackStart,
} from '../src/oxlint';

test(`oxlint base config excludes framework-specific plugins`, () => {
  expect(oxlintConfigBase.plugins).not.toContain(`react`);
  expect(oxlintConfigBase.plugins).not.toContain(`react-perf`);
  expect(oxlintConfigBase.plugins).not.toContain(`nextjs`);
});

test(`oxlint framework configs contain only their project differences`, () => {
  expect(oxlintConfigReact.plugins).toEqual([`react`, `react-perf`]);
  expect(oxlintConfigReact.rules?.[`react/exhaustive-deps`]).toBe(`error`);
  expect(oxlintConfigNext.plugins).toEqual([`nextjs`]);
  expect(oxlintConfigNext.rules?.[`nextjs/no-img-element`]).toBe(`error`);
  expect(oxlintConfigNext.plugins).not.toContain(`react`);
});

test(`oxlint config composes base, React, and Next options`, () => {
  const config = oxlintConfig(
    oxlintConfigBase,
    oxlintConfigReact,
    oxlintConfigNext,
  );

  expect(config.plugins).toContain(`typescript`);
  expect(config.plugins).toContain(`react`);
  expect(config.plugins).toContain(`react-perf`);
  expect(config.plugins).toContain(`nextjs`);
  expect(config.rules?.[`react/exhaustive-deps`]).toBe(`error`);
  expect(config.rules?.[`nextjs/no-img-element`]).toBe(`error`);
});

test(`oxlint base allows stylesheet side effects without package exceptions`, () => {
  expect(oxlintConfigBase.rules?.[`no-unassigned-import`]).toEqual([
    `warn`,
    {
      allow: [`**/*.css`, `**/*.scss`, `**/*.less`],
    },
  ]);
});

test(`framework modules own their generated-file ignore options`, () => {
  expect(oxlintConfigBase.ignorePatterns).toContain(`**/*.gen.ts`);
  expect(oxlintConfigBase.ignorePatterns).not.toContain(`**/.next/`);
  for (const pattern of [`**/.next/`, `**/out/`, `**/next-env.d.ts`]) {
    expect(oxlintConfigNext.ignorePatterns).toContain(pattern);
  }
  for (const pattern of [`**/.output/`, `**/.nitro/`, `**/.tanstack/`]) {
    expect(oxlintConfigTanstackStart.ignorePatterns).toContain(pattern);
  }
});
