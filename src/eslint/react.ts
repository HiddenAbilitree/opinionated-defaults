import pluginReactHooks from 'eslint-plugin-react-hooks';
import pluginReact from 'eslint-plugin-react';

const reactConfig = [
  pluginReact.configs.flat.all,
  pluginReactHooks.configs['recommended-latest'],
];

export default reactConfig;
