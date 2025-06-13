import { type Config } from 'prettier';

import baseConfig from './base';

const astroConfig: Config = {
  ...baseConfig,
  plugins: ['prettier-plugin-astro', 'prettier-plugin-tailwindcss'],
  tailwindStylesheet: './src/styles/global.css',
  tailwindFunctions: ['cva', 'clsx', 'cn'],
  overrides: [
    {
      files: '*.astro',
      options: { parser: 'astro' },
    },
  ],
};

export default astroConfig;
