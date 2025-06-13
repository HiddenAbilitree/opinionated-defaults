import functional from 'eslint-plugin-functional';

const functionalConfig = [
  functional.configs.externalTypeScriptRecommended,
  functional.configs.recommended,
  functional.configs.stylistic,
  {
    rules: {
      'functional/no-expression-statements': ['off'],
      'functional/no-return-void': ['off'],
    },
  },
];

export default functionalConfig;
