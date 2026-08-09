import type { OxfmtConfig, OxfmtOverrideConfig } from 'oxfmt';

export const oxfmtConfig = (
  source: OxfmtConfig,
  ...sources: OxfmtConfig[]
): OxfmtConfig => {
  const config: OxfmtConfig = {};
  const ignorePatterns = new Set<string>();
  const overrides: OxfmtOverrideConfig[] = [];
  let sortImports: OxfmtConfig[`sortImports`];
  let sortTailwindcss: OxfmtConfig[`sortTailwindcss`];

  for (const current of [source, ...sources]) {
    Object.assign(config, current);
    for (const pattern of current.ignorePatterns ?? []) {
      ignorePatterns.add(pattern);
    }
    overrides.push(...(current.overrides ?? []));

    if (
      typeof current.sortImports === `object` &&
      current.sortImports !== null
    ) {
      sortImports = {
        ...(typeof sortImports === `object` ? sortImports : {}),
        ...current.sortImports,
      };
    } else if (current.sortImports !== undefined) {
      sortImports = current.sortImports;
    }

    if (
      typeof current.sortTailwindcss === `object` &&
      current.sortTailwindcss !== null
    ) {
      sortTailwindcss = {
        ...(typeof sortTailwindcss === `object` ? sortTailwindcss : {}),
        ...current.sortTailwindcss,
      };
    } else if (current.sortTailwindcss !== undefined) {
      sortTailwindcss = current.sortTailwindcss;
    }
  }

  if (ignorePatterns.size > 0) config.ignorePatterns = [...ignorePatterns];
  if (overrides.length > 0) config.overrides = overrides;
  if (sortImports !== undefined) config.sortImports = sortImports;
  if (sortTailwindcss !== undefined) {
    config.sortTailwindcss = sortTailwindcss;
  }

  return config;
};
