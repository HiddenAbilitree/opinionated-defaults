import { type Config } from 'prettier';

import baseConfig from './base';
import { prettierConfig } from './config';

const astroConfig: Config = prettierConfig(
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
