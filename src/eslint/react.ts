import type { ConfigArray } from 'typescript-eslint';

import pluginReact from 'eslint-plugin-react';
import pluginReactHooks from 'eslint-plugin-react-hooks';
import { defineConfig } from 'eslint/config';

const reactConfig: ConfigArray = defineConfig([
  pluginReact.configs.flat[`recommended`]!,
  pluginReact.configs.flat[`jsx-runtime`]!,
  {
    files: [
      `**/*.ts`,
      `**/*.js`,
      `**/*.mjs`,
      `**/*.mts`,
      `**/*.tsx`,
      `**/*.jsx`,
    ],
  },
  {
    extends: [`react-hooks/recommended`],
    plugins: {
      'react-hooks': pluginReactHooks,
    },
  },
  {
    settings: {
      react: {
        version: `detect`,
      },
    },
  },
]);

export default reactConfig;
