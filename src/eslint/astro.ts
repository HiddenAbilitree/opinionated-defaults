import type { ConfigArray } from 'typescript-eslint';

import eslintPluginAstro from 'eslint-plugin-astro';

const astroConfig: ConfigArray = [...eslintPluginAstro.configs.recommended];

export default astroConfig;
