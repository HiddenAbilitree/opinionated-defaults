import baseConfig from "./base.js";
import functional from "eslint-plugin-functional";
import tsConfig from './typescript.js';
const elysiaConfig = [
  ...baseConfig,
  ...tsConfig,
  functional.configs.externalTypeScriptRecommended,
  functional.configs.recommended,
  functional.configs.stylistic,
  {
    rules: {
      "functional/no-expression-statements": ["off"],
      "functional/no-return-void": ["off"],
    }
  }
]

export default elysiaConfig;
