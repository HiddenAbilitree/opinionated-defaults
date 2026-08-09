import type {
  DummyRuleMap,
  OxlintConfig,
  OxlintEnv,
  OxlintGlobals,
  OxlintOverride,
  RuleCategories,
} from 'oxlint';

export const oxlintConfig = (
  source: OxlintConfig,
  ...sources: OxlintConfig[]
): OxlintConfig => {
  const config: OxlintConfig = {};
  const categories: RuleCategories = {};
  const env: OxlintEnv = {};
  const globals: OxlintGlobals = {};
  const options: NonNullable<OxlintConfig[`options`]> = {};
  const rules: DummyRuleMap = {};
  const settings: NonNullable<OxlintConfig[`settings`]> = {};
  const ignorePatterns = new Set<string>();
  const overrides: OxlintOverride[] = [];
  const plugins = new Set<NonNullable<OxlintConfig[`plugins`]>[number]>();

  for (const current of [source, ...sources]) {
    Object.assign(config, current);
    Object.assign(categories, current.categories ?? {});
    Object.assign(env, current.env ?? {});
    Object.assign(globals, current.globals ?? {});
    Object.assign(options, current.options ?? {});
    Object.assign(rules, current.rules ?? {});
    Object.assign(settings, current.settings ?? {});
    for (const pattern of current.ignorePatterns ?? []) {
      ignorePatterns.add(pattern);
    }
    overrides.push(...(current.overrides ?? []));
    for (const plugin of current.plugins ?? []) {
      plugins.add(plugin);
    }
  }

  if (Object.keys(categories).length > 0) config.categories = categories;
  if (Object.keys(env).length > 0) config.env = env;
  if (Object.keys(globals).length > 0) config.globals = globals;
  if (Object.keys(options).length > 0) config.options = options;
  if (Object.keys(rules).length > 0) config.rules = rules;
  if (Object.keys(settings).length > 0) config.settings = settings;
  if (ignorePatterns.size > 0) config.ignorePatterns = [...ignorePatterns];
  if (overrides.length > 0) config.overrides = overrides;
  if (plugins.size > 0) config.plugins = [...plugins];

  return config;
};
