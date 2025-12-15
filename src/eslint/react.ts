import type { ConfigArray } from 'typescript-eslint';

import pluginReact from 'eslint-plugin-react';
import pluginReactHooks from 'eslint-plugin-react-hooks';
import { defineConfig } from 'eslint/config';

const reactConfig: ConfigArray = defineConfig([
  pluginReact.configs.flat[`recommended`]!,
  pluginReact.configs.flat[`jsx-runtime`]!,
  pluginReactHooks.configs.flat[`recommended-latest`],
  {
    settings: {
      react: {
        version: `detect`,
      },
    },
  },
]);

export default reactConfig;
