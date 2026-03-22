import type { ConfigArray } from 'typescript-eslint';

import betterTailwindcss from 'eslint-plugin-better-tailwindcss';
import { defineConfig } from 'eslint/config';

const betterTailwindcssConfig: ConfigArray = defineConfig([
  {
    plugins: {
      'better-tailwindcss': betterTailwindcss,
    },
    rules: {
      ...betterTailwindcss.configs[`recommended-error`].rules,
    },
  },
]);

export default betterTailwindcssConfig;
