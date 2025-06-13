# ![Banner](/assets/banner.svg)
A collection of opinionated tooling configurations.

## Supported Framework Configurations:
### Eslint:
- Astro
- Elysia.js
- Next.js

#### :electric_plug: Included plugins :
- [eslint-plugin-prettier](https://github.com/prettier/eslint-plugin-prettier)
- [eslint-plugin-no-relative-import-paths](https://github.com/MelvinVermeer/eslint-plugin-no-relative-import-paths)
- [eslint-plugin-react](https://github.com/jsx-eslint/eslint-plugin-react)
- [eslint-plugin-react-hooks](https://github.com/facebook/react/tree/main/packages/eslint-plugin-react-hooks)
- [eslint-plugin-turbo](https://github.com/vercel/turborepo/tree/main/packages/eslint-plugin-turbo)
- [eslint-plugin-unicorn](https://github.com/sindresorhus/eslint-plugin-unicorn)
- [eslint-plugin-functional](https://github.com/eslint-functional/eslint-plugin-functional)
- [eslint-plugin-astro](https://github.com/ota-meshi/eslint-plugin-astro)
- [eslint-plugin-next](https://github.com/vercel/next.js/tree/canary/packages/eslint-plugin-next)
- [eslint-stylistic](https://github.com/eslint-stylistic/eslint-stylistic)

### Prettier:
- Astro
- Next.js
- +Opinionated defaults

## Usage:
#### Installation:
`bun add reasonable-defaults -d`
`npm i reasonable-defaults -D`
### Eslint:
```ts
import {eslintConfigConfigName} from "reasonable-defaults/eslint"

const eslintConfig = [...eslintConfigConfigName];

export default eslintConfig;
```

##### Supported Imports:
- eslintConfigAstro (Astro)
- eslintConfigElysia (Elysia.js)
- eslintConfigNext (Next.js)
- eslintConfigTurbo (Turborepo)
- eslintConfigBase (General rules for every project)
- eslintConfigFunctional (Enforces functional style)
- eslintConfigPrettier (Runs prettier as ESLint rules)
- eslintConfigReact (General rules for React)
- eslintConfigRelative (Forces the use of non-relative import paths using path aliases)
- eslintConfigStylistic (Code-style through ESLint rules)

### Prettier:
```ts
import {prettierConfigConfigName} from "reasonable-defaults/prettier"

const prettierConfig = {...prettierConfigConfigName};

export default prettierConfig;
```

##### Supported Imports:
- prettierConfigAstro (Astro prettier rules with Tailwind class ordering)
- prettierConfigNext (Rules for Next.js with Tailwind class ordering)
- prettierConfigBase (General rules for every project)

## TODO:
- Improve repository structure (How to manage configuration options within eslint dir?)
- Maybe convert into monorepo with one package per tool instead of multiple exports from one package.
- Prevent importing overlapping configurations (i.e., Next.js ESLint config contains base config)
