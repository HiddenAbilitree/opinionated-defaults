import { expect, test } from 'bun:test';

import type { OxlintConfig } from 'oxlint';

import {
  oxlintConfig,
  oxlintConfigBase,
  oxlintConfigNext,
  oxlintConfigReact,
  oxlintConfigSolid,
  oxlintConfigTanstackStart,
  oxlintIgnorePatterns,
  oxlintOverride,
} from '../src/oxlint';

test(`oxlint base config excludes framework-specific plugins`, () => {
  expect(oxlintConfigBase.plugins).not.toContain(`react`);
  expect(oxlintConfigBase.plugins).not.toContain(`react-perf`);
  expect(oxlintConfigBase.plugins).not.toContain(`nextjs`);
});

test(`oxlint framework configs contain only their project differences`, () => {
  expect(oxlintConfigReact.plugins).toEqual([`react`, `react-perf`]);
  expect(oxlintConfigReact.rules?.[`react/exhaustive-deps`]).toBe(`error`);
  expect(oxlintConfigReact.rules?.[`react/react-in-jsx-scope`]).toBe(`off`);
  expect(oxlintConfigNext.plugins).toEqual([`nextjs`]);
  expect(oxlintConfigNext.rules?.[`nextjs/no-img-element`]).toBe(`error`);
  expect(oxlintConfigNext.plugins).not.toContain(`react`);
});

test(`oxlint config composes an ordered array of shared configs`, () => {
  const config = oxlintConfig([oxlintConfigBase, oxlintConfigReact, oxlintConfigNext]);

  expect(config.plugins).toContain(`typescript`);
  expect(config.plugins).toContain(`react`);
  expect(config.plugins).toContain(`react-perf`);
  expect(config.plugins).toContain(`nextjs`);
  expect(config.rules?.[`react/exhaustive-deps`]).toBe(`error`);
  expect(config.rules?.[`nextjs/no-img-element`]).toBe(`error`);
});

test(`later configs win duplicate keys while list values deduplicate`, () => {
  const config = oxlintConfig([
    {
      plugins: [`react`],
      rules: { eqeqeq: `warn` },
    },
    {
      plugins: [`react`, `nextjs`],
      rules: { eqeqeq: `error` },
    },
  ]);

  expect(config.rules?.eqeqeq).toBe(`error`);
  expect(config.plugins?.filter((plugin) => plugin === `react`)).toHaveLength(1);
  expect(config.plugins).toEqual([`react`, `nextjs`]);
});

test(`JavaScript plugins survive composition with later framework configs`, () => {
  const config = oxlintConfig([
    oxlintConfigSolid,
    { jsPlugins: [`eslint-plugin-example`] },
    oxlintConfigSolid,
    oxlintConfigReact,
  ]);
  const override = oxlintOverride([`apps/solid/**/*`], [oxlintConfigSolid, { jsPlugins: [] }]);

  expect(config.jsPlugins).toEqual([`eslint-plugin-solid`, `eslint-plugin-example`]);
  expect(override.jsPlugins).toEqual([`eslint-plugin-solid`]);
});

test(`later rule options extend earlier rule options`, () => {
  const config = oxlintConfig([
    {
      rules: {
        'import/no-unassigned-import': [`warn`, { allow: [`**/*.css`, `**/*.scss`] }],
      },
    },
    {
      rules: {
        'import/no-unassigned-import': [`error`, { allow: [`server-only`, `*/css`] }],
      },
    },
  ]);

  expect(config.rules?.[`import/no-unassigned-import`]).toEqual([
    `error`,
    {
      allow: [`**/*.css`, `**/*.scss`, `server-only`, `*/css`],
    },
  ]);
});

test(`file overrides extend matching root rule options`, () => {
  const config = oxlintConfig([oxlintConfigBase], {
    overrides: [
      oxlintOverride(
        [`apps/nexus/**/*`],
        [
          {
            rules: {
              'import/no-unassigned-import': [`error`, { allow: [`server-only`, `*/css`] }],
            },
          },
        ],
      ),
    ],
  });

  expect(config.overrides?.[0]?.rules?.[`import/no-unassigned-import`]).toEqual([
    `error`,
    {
      allow: [`**/*.css`, `**/*.scss`, `**/*.less`, `server-only`, `*/css`],
    },
  ]);
});

test(`oxlint config does not alias source collections`, () => {
  const source = {
    plugins: [`react`],
    rules: { eqeqeq: `warn` },
  } satisfies OxlintConfig;

  const config = oxlintConfig([source]);

  expect(config.plugins).not.toBe(source.plugins);
  expect(config.rules).not.toBe(source.rules);
  expect(source).toEqual({
    plugins: [`react`],
    rules: { eqeqeq: `warn` },
  });
});

test(`oxlint override composes only its declared configs`, () => {
  const override = oxlintOverride([`apps/nexus/**/*`], [oxlintConfigReact, oxlintConfigNext]);

  for (const plugin of [`react`, `react-perf`, `nextjs`] as const) {
    expect(override.plugins).toContain(plugin);
  }
  expect(override.plugins).not.toContain(`typescript`);
  expect(override.rules?.[`react/exhaustive-deps`]).toBe(`error`);
  expect(override.rules?.[`nextjs/no-img-element`]).toBe(`error`);
  expect(override.rules?.[`import/no-unassigned-import`]).toBeUndefined();
  expect(override.files).toEqual([`apps/nexus/**/*`]);
});

test(`oxlint config preserves overlapping config-specific overrides`, () => {
  const config = oxlintConfig([oxlintConfigBase], {
    overrides: [
      oxlintOverride([`apps/project-1/**/*`, `apps/project-2/**/*`], [oxlintConfigReact]),
      oxlintOverride([`apps/project-1/**/*`], [oxlintConfigNext]),
    ],
  });

  expect(config.plugins).not.toContain(`react`);
  expect(config.plugins).not.toContain(`nextjs`);
  expect(config.overrides).toHaveLength(2);
  expect(config.overrides?.[0]?.plugins).toEqual([`react`, `react-perf`]);
  expect(config.overrides?.[1]?.plugins).toEqual([`nextjs`]);
});

test(`oxlint ignore patterns compose without generated config spreads`, () => {
  const patterns = oxlintIgnorePatterns([
    oxlintConfigNext,
    oxlintConfigTanstackStart,
    oxlintConfigNext,
  ]);

  expect(patterns.filter((pattern) => pattern === `**/.next/`)).toHaveLength(1);
  expect(patterns).toContain(`**/.tanstack/`);
});

test(`oxlint base allows stylesheet side effects without package exceptions`, () => {
  expect(oxlintConfigBase.rules?.[`import/no-unassigned-import`]).toEqual([
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
