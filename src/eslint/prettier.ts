import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';

const eslintPrettierConfig = [
  eslintPluginPrettierRecommended,
  {
    'prettier/prettier': ['error', { usePrettierrc: true }],
  },
];

export default eslintPrettierConfig;
