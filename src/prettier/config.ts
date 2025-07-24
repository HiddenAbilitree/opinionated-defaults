import type { Config } from 'prettier';

import lodash from 'lodash';

const isArray = lodash.isArray.bind(lodash);
const mergeWith = lodash.mergeWith.bind(lodash);
const union = lodash.union.bind(lodash);

// function runs when object key matches. Returns the merge operation result
const arrayMerge = (arr1: Config, arr2: Config) => {
  if (!isArray(arr1) || !isArray(arr2)) return;

  const retArr = union(arr1, arr2);

  if (retArr.includes(`prettier-plugin-tailwindcss`)) {
    retArr.push(
      retArr.splice(retArr.indexOf(`prettier-plugin-tailwindcss`), 1)[0],
    ); // cooked...
  }

  return retArr as unknown as Config;
};

export const prettierConfig = (source: Config, ...sources: Config[]): Config =>
  mergeWith({}, source, ...sources, arrayMerge) as Config;
