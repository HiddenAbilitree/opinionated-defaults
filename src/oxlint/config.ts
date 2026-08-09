import type { OxlintConfig, OxlintOverride } from 'oxlint';

type OxlintConfigs = readonly [OxlintConfig, ...OxlintConfig[]];

const mergeArrays = <T>(
  config: T[] | undefined,
  source: T[] | undefined,
): T[] | undefined => {
  const merged = [...(config ?? []), ...(source ?? [])];
  return merged.length === 0 ? undefined : merged;
};

const mergeObjects = <T extends object>(
  config: T | undefined,
  source: T | undefined,
): T | undefined => {
  if (config === undefined) {
    return source === undefined ? undefined : { ...source };
  }
  if (source === undefined) return { ...config };
  return { ...config, ...source };
};

const mergeUniqueArrays = <T>(
  config: T[] | undefined,
  source: T[] | undefined,
): T[] | undefined => {
  const merged = [...new Set([...(config ?? []), ...(source ?? [])])];
  return merged.length === 0 ? undefined : merged;
};

const mergeOxlintConfig = (
  config: OxlintConfig,
  source: OxlintConfig,
): OxlintConfig => ({
  ...config,
  ...source,
  categories: mergeObjects(config.categories, source.categories),
  env: mergeObjects(config.env, source.env),
  globals: mergeObjects(config.globals, source.globals),
  ignorePatterns: mergeUniqueArrays(
    config.ignorePatterns,
    source.ignorePatterns,
  ),
  options: mergeObjects(config.options, source.options),
  overrides: mergeArrays(config.overrides, source.overrides),
  plugins: mergeUniqueArrays(config.plugins, source.plugins),
  rules: mergeObjects(config.rules, source.rules),
  settings: mergeObjects(config.settings, source.settings),
});

const mergeOxlintConfigs = (configs: readonly OxlintConfig[]): OxlintConfig =>
  configs.reduce((config, source) => mergeOxlintConfig(config, source), {});

export const oxlintIgnorePatterns = (configs: OxlintConfigs): string[] => [
  ...new Set(configs.flatMap(({ ignorePatterns = [] }) => ignorePatterns)),
];

export const oxlintConfig = (
  sharedConfigs: OxlintConfigs,
  config?: OxlintConfig,
): OxlintConfig =>
  mergeOxlintConfigs(
    config === undefined ? sharedConfigs : [...sharedConfigs, config],
  );

export const oxlintOverride = (
  files: OxlintOverride[`files`],
  configs: OxlintConfigs,
): OxlintOverride => {
  const { env, globals, jsPlugins, plugins, rules } =
    mergeOxlintConfigs(configs);

  return {
    env,
    files,
    globals,
    jsPlugins,
    plugins,
    rules,
  };
};
