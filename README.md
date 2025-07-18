# ![Banner](/assets/banner.svg)

A collection of opinionated tooling configurations.

## Supported Framework Configurations:

### Eslint:

- Astro
- Elysia.js
- Next.js

#### Exports:

- eslintConfigAstro (Astro)
- eslintConfigBase (General rules for every project)
- eslintConfigElysia (Elysia.js)
- eslintConfigFunctional (Enforces functional style)
- eslintConfigNext (Next.js)
- eslintConfigPrettier (Runs Prettier as ESLint rules)
- eslintConfigReact (General rules for React)
- eslintConfigRelative (Enforces the use of absolute import paths using path aliases)
- eslintConfigStylistic (Enforces code-style through ESLint rules)
- eslintConfigTurbo (Turborepo)

#### Included plugins:

- [eslint-plugin-astro](https://github.com/ota-meshi/eslint-plugin-astro)
- [eslint-plugin-prettier](https://github.com/prettier/eslint-plugin-prettier)
- [eslint-plugin-no-relative-import-paths](https://github.com/MelvinVermeer/eslint-plugin-no-relative-import-paths)
- [eslint-plugin-react](https://github.com/jsx-eslint/eslint-plugin-react)
- [eslint-plugin-react-hooks](https://github.com/facebook/react/tree/main/packages/eslint-plugin-react-hooks)
- [eslint-plugin-turbo](https://github.com/vercel/turborepo/tree/main/packages/eslint-plugin-turbo)
- [eslint-plugin-unicorn](https://github.com/sindresorhus/eslint-plugin-unicorn)
- [eslint-plugin-functional](https://github.com/eslint-functional/eslint-plugin-functional)
- [eslint-plugin-next](https://github.com/vercel/next.js/tree/canary/packages/eslint-plugin-next)
- [eslint-plugin-perfectionist](https://github.com/azat-io/eslint-plugin-perfectionist)
- [eslint-stylistic](https://github.com/eslint-stylistic/eslint-stylistic)

### Prettier:

- Astro
- Next.js
- +Opinionated defaults

#### Exports:

- prettierConfigAstro (Astro prettier rules with Tailwind class ordering)
- prettierConfigNext (Rules for Next.js with Tailwind class ordering)
- prettierConfigBase (General rules for every project)
- prettierConfigSortImports (Prettier-based import sorting)
- configMerge (used to merge configurations)

#### Included plugins:

- [prettier-plugin-astro](https://github.com/withastro/prettier-plugin-astro)
- [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss)
- [prettier-plugin-sort-imports](https://github.com/trivago/prettier-plugin-sort-imports)

## Installation:

```
bun add @hiddenability/opinionated-defaults -d
```

```
npm i @hiddenability/opinionated-defaults -D
```

## Usage:

### Eslint:

```ts
// eslint.config.ts
import { eslintConfigConfigName } from '@hiddenability/opinionated-defaults/eslint';

const eslintConfig = [...eslintConfigConfigName];

export default eslintConfig;
```

### Prettier:

```ts
// prettier.config.mjs
import { prettierConfigConfigName } from '@hiddenability/opinionated-defaults/prettier';

const prettierConfig = { ...prettierConfigConfigName };

export default prettierConfig;
```

#### Extending/Combining Prettier Configs:

Since prettier uses a configuration object instead of a flat config like ESLint, to extend or combine configurations, you need to use the provided merge function.

```ts
// prettier.config.mjs
import {
  merge,
  prettierConfig1,
  prettierConfig2,
} from '@hiddenability/opinionated-defaults/prettier';

const prettierConfig = merge(
  prettierConfig1,
  prettierConfig2,
  {
    /* your custom rules */
  },
 /*...*/
);

export default prettierConfig;
```

#### TailwindCSS Plugin:

When using `prettier-config-tailwind`, make sure to specify the CSS file that contains the `@import "tailwindcss"` directive.

For example, given the following css file:
```css
// /app/styles.css
@import 'tailwindcss';
```

This should be a minimal version of your Prettier config:

```ts
// prettier.config.mjs
import { prettierConfigTailwind } from '@hiddenability/opinionated-defaults/prettier';

const prettierConfig = merge(
  prettierConfigTailwind,
  { tailwindStylesheet: './app/styles.css' }
);
```


## TODO:

- Improve repository structure (How to manage configuration options within eslint dir?).
- Maybe convert into monorepo with one package per tool instead of multiple exports from one package.
- Prevent importing overlapping configurations (i.e., Next.js ESLint config contains base config).
- Support node module resolution.
- Maybe make declarative configurations instead of just providing wrapped config.
