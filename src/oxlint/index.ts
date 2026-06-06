import oxlintConfigBase from '../oxlintrc.json';

type OxlintConfig = typeof oxlintConfigBase;

export { default as oxlintConfig } from '../oxlintrc.json';

const withPlugins = (...plugins: string[]): OxlintConfig => ({
  ...oxlintConfigBase,
  plugins: [...new Set([...oxlintConfigBase.plugins, ...plugins])],
});

export const oxlintConfigReact: OxlintConfig = withPlugins(
  `react`,
  `react-perf`,
);
export const oxlintConfigNext: OxlintConfig = withPlugins(
  `react`,
  `react-perf`,
  `nextjs`,
);
