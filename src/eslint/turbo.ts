import type { ConfigArray } from 'typescript-eslint';

import turboPlugin from 'eslint-plugin-turbo';

const turborepoConfig: ConfigArray = [turboPlugin.configs[`flat/recommended`]];

export default turborepoConfig;
