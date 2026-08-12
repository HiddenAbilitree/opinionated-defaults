import { configs } from 'eslint-plugin-perfectionist';
import type { ConfigArray } from 'typescript-eslint';

const perfectionistConfig: ConfigArray = [
  configs[`recommended-natural`],
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
