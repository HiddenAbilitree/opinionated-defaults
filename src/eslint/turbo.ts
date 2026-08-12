import type { Linter } from 'eslint';
import turboPlugin from 'eslint-plugin-turbo';
import type { ConfigArray } from 'typescript-eslint';

const isFlatConfig = (
  config: Linter.Config | Linter.Config[] | Linter.LegacyConfig | undefined,
): config is Linter.Config =>
  !Array.isArray(config) && typeof config?.plugins === `object` && config.plugins !== null;

const turboRecommendedConfig = turboPlugin.configs?.[`flat/recommended`];

if (!isFlatConfig(turboRecommendedConfig)) {
  throw new TypeError(`eslint-plugin-turbo did not provide a flat config`);
}

const turborepoConfig: ConfigArray = [turboRecommendedConfig];

export default turborepoConfig;
