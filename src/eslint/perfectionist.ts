import type { ConfigArray } from 'typescript-eslint';

import perfectionist from 'eslint-plugin-perfectionist';

const perfectionistConfig: ConfigArray = [
  perfectionist.configs[`recommended-natural`],
  {
    rules: {
      'perfectionist/sort-object-types': [
        `warn`,
        {
          partitionByNewLine: true,
        },
      ],
      'perfectionist/sort-objects': [
        `warn`,
        {
          partitionByNewLine: true,
        },
      ],
    },
  },
];

export default perfectionistConfig;
