import { type Config } from 'prettier';

import baseConfig from './base';

const astroConfig: Config = {
  ...baseConfig,
  plugins: ['prettier-plugin-astro'],
  overrides: [
    {
      files: '*.astro',
      options: { parser: 'astro' },
    },
  ],
};

export default astroConfig;
