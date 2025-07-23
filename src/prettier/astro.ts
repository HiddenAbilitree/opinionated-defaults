import lodash from 'lodash';
import { type Config } from 'prettier';

import baseConfig from './base';

const { merge } = lodash;

const astroConfig: Config = merge(
  {
    overrides: [
      {
        files: `*.astro`,
        options: { parser: `astro` },
      },
    ],
    plugins: [`prettier-plugin-astro`],
  },
  baseConfig,
);

export default astroConfig;
