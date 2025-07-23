import type { ConfigArray } from 'typescript-eslint';

import pluginReact from 'eslint-plugin-react';
import pluginReactHooks from 'eslint-plugin-react-hooks';

const reactConfig: ConfigArray = [
  pluginReact.configs.flat[`recommended`]!,
  pluginReact.configs.flat[`jsx-runtime`]!,
  pluginReactHooks.configs[`recommended-latest`],
];

export default reactConfig;
