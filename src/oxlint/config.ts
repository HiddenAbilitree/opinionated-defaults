import type { DummyRule, OxlintConfig, OxlintOverride } from 'oxlint';

type OxlintConfigs = readonly [OxlintConfig, ...OxlintConfig[]];

const mergeArrays = <T>(config: T[] | undefined, source: T[] | undefined): T[] | undefined => {
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

type OxlintRules = NonNullable<OxlintConfig[`rules`]>;

const mergeRuleOption = (config: unknown, source: unknown): unknown => {
  if (Array.isArray(config) && Array.isArray(source)) {
    return [...new Set([...config, ...source])];
  }
  if (
    typeof config === `object` &&
    config !== null &&
    !Array.isArray(config) &&
    typeof source === `object` &&
    source !== null &&
    !Array.isArray(source)
  ) {
    const configEntries = new Map<string, unknown>(Object.entries(config));
    return {
      ...config,
      ...Object.fromEntries(
        Object.entries(source).map(([key, value]) => [
          key,
          configEntries.has(key) ? mergeRuleOption(configEntries.get(key), value) : value,
        ]),
      ),
    };
  }
  return source;
};

const mergeRule = (
  config: DummyRule | undefined,
  source: DummyRule | undefined,
): DummyRule | undefined =>
  Array.isArray(config) && Array.isArray(source)
    ? [
        source[0],
        ...source.slice(1).map((value, index) => mergeRuleOption(config[index + 1], value)),
      ]
    : source;

const mergeRules = (
  config: OxlintRules | undefined,
  source: OxlintRules | undefined,
): OxlintRules | undefined => {
  if (config === undefined) {
    return source === undefined ? undefined : { ...source };
  }
  if (source === undefined) return { ...config };
  return Object.entries(source).reduce<OxlintRules>(
    (rules, [rule, value]) => ({
      ...rules,
      [rule]: mergeRule(config[rule], value),
    }),
    { ...config },
  );
};

const mergeInheritedRules = (
  config: OxlintRules | undefined,
  source: OxlintRules | undefined,
): OxlintRules | undefined =>
  source === undefined
    ? source
    : Object.entries(source).reduce<OxlintRules>(
        (rules, [rule, value]) => ({
          ...rules,
          [rule]: mergeRule(config?.[rule], value),
        }),
        {},
      );

const mergeInheritedOverrideRules = (config: OxlintConfig): OxlintConfig => ({
  ...config,
  overrides: config.overrides?.map((override) => ({
    ...override,
    rules: mergeInheritedRules(config.rules, override.rules),
  })),
});

const mergeOxlintConfig = (config: OxlintConfig, source: OxlintConfig): OxlintConfig => ({
  ...config,
  ...source,
  categories: mergeObjects(config.categories, source.categories),
  env: mergeObjects(config.env, source.env),
  globals: mergeObjects(config.globals, source.globals),
  ignorePatterns: mergeUniqueArrays(config.ignorePatterns, source.ignorePatterns),
  options: mergeObjects(config.options, source.options),
  overrides: mergeArrays(config.overrides, source.overrides),
  plugins: mergeUniqueArrays(config.plugins, source.plugins),
  rules: mergeRules(config.rules, source.rules),
  settings: mergeObjects(config.settings, source.settings),
});

const mergeOxlintConfigs = (configs: readonly OxlintConfig[]): OxlintConfig =>
  mergeInheritedOverrideRules(
    configs.reduce((config, source) => mergeOxlintConfig(config, source), {}),
  );

export const oxlintIgnorePatterns = (configs: OxlintConfigs): string[] => [
  ...new Set(configs.flatMap(({ ignorePatterns = [] }) => ignorePatterns)),
];

export const oxlintConfig = (sharedConfigs: OxlintConfigs, config?: OxlintConfig): OxlintConfig =>
  mergeOxlintConfigs(config === undefined ? sharedConfigs : [...sharedConfigs, config]);

export const oxlintOverride = (
  files: OxlintOverride[`files`],
  configs: OxlintConfigs,
): OxlintOverride => {
  const { env, globals, jsPlugins, plugins, rules } = mergeOxlintConfigs(configs);

  return {
    env,
    files,
    globals,
    jsPlugins,
    plugins,
    rules,
  };
};
