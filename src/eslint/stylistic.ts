import stylistic from '@stylistic/eslint-plugin';
const stylisticConfig = [
  stylistic.configs.all,
  {
    plugins: {
      '@stylistic': stylistic,
    },
  },
];

export default stylisticConfig;
