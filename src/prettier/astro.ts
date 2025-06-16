import baseConfig from './base';
import lodash from 'lodash';
import { type Config } from 'prettier';

const { merge } = lodash;

const astroConfig: Config = merge(
  {
    plugins: ['prettier-plugin-astro'],
    overrides: [
      {
        files: '*.astro',
        options: { parser: 'astro' },
      },
    ],
  },
  baseConfig,
);

export default astroConfig;
