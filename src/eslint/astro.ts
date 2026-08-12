import { configs } from 'eslint-plugin-astro';
import type { ConfigArray } from 'typescript-eslint';

const astroConfig: ConfigArray = [...configs.recommended];

export default astroConfig;
