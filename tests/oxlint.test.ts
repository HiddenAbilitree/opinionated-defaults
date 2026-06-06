import { expect, test } from 'bun:test';

import {
  oxlintConfig,
  oxlintConfigNext,
  oxlintConfigReact,
} from '../src/oxlint';

const plugins = (config: { plugins: string[] }) => config.plugins;

test(`oxlint default config excludes framework-specific plugins`, () => {
  expect(plugins(oxlintConfig)).not.toContain(`react`);
  expect(plugins(oxlintConfig)).not.toContain(`react-perf`);
  expect(plugins(oxlintConfig)).not.toContain(`nextjs`);
});

test(`oxlint react config enables react plugins without nextjs`, () => {
  expect(plugins(oxlintConfigReact)).toContain(`react`);
  expect(plugins(oxlintConfigReact)).toContain(`react-perf`);
  expect(plugins(oxlintConfigReact)).not.toContain(`nextjs`);
});

test(`oxlint next config enables nextjs and react plugins`, () => {
  expect(plugins(oxlintConfigNext)).toContain(`react`);
  expect(plugins(oxlintConfigNext)).toContain(`react-perf`);
  expect(plugins(oxlintConfigNext)).toContain(`nextjs`);
});
