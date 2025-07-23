import type { Config } from 'prettier';

import lodash from 'lodash';

const { isArray, mergeWith, union } = lodash;

// function runs when object key matches. Returns the merge operation result
const arrayMerge = (arr1: object, arr2: object) => {
  if (!isArray(arr1) || !isArray(arr2)) return;

  const retArr = union(arr1, arr2);

  if (retArr.includes(`prettier-plugin-tailwindcss`)) {
    retArr.push(
      retArr.splice(retArr.indexOf(`prettier-plugin-tailwindcss`), 1)[0],
    ); // cooked...
  }

  return retArr;
};

export const merge = (source: Config, ...sources: Config[]): Config =>
  mergeWith({}, source, ...sources, arrayMerge);
