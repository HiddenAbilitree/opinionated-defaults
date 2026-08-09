import type { OxfmtConfig } from 'oxfmt';

const config: OxfmtConfig = {
  arrowParens: `always`,
  bracketSameLine: false,
  bracketSpacing: true,
  ignorePatterns: [`**/*.gen.ts`],
  jsxSingleQuote: true,
  semi: true,
  singleQuote: true,
  sortImports: {
    newlinesBetween: true,
  },
  sortTailwindcss: {
    functions: [`cva`, `clsx`, `cn`],
  },
  tabWidth: 2,
  trailingComma: `all`,
  useTabs: false,
};

export default config;
