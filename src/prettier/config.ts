import lodash from 'lodash';
import type { Config } from 'prettier';

const isArray = lodash.isArray.bind(lodash);
const mergeWith = lodash.mergeWith.bind(lodash);
const union = lodash.union.bind(lodash);

// function runs when object key matches. Returns the merge operation result
const arrayMerge = (arr1: unknown, arr2: unknown): unknown => {
  if (!isArray(arr1) || !isArray(arr2)) return undefined;

  const retArr: unknown[] = union(arr1, arr2);

  if (retArr.includes(`prettier-plugin-tailwindcss`)) {
    retArr.push(retArr.splice(retArr.indexOf(`prettier-plugin-tailwindcss`), 1)[0]); // cooked...
  }

  return retArr;
};

export const prettierConfig = (source: Config, ...sources: Config[]): Config => {
  const destination: Config = {};

  return mergeWith(destination, source, ...sources, arrayMerge);
};
