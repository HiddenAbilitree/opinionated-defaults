import type { OxlintConfig } from 'oxlint';

const config: OxlintConfig = {
  ignorePatterns: [`**/.output/`, `**/.nitro/`, `**/.tanstack/`],
};

export default config;
