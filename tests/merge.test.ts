import { merge } from '../src/prettier/index';
import type { Config } from 'prettier';
import { expect, test } from 'vitest';

const obj1: Config = {
  trailingComma: 'all',
  singleQuote: true,
  jsxSingleQuote: true,
  tabWidth: 2,
  arrowParens: 'always',
  bracketSpacing: true,
  useTabs: false,
  semi: true,
  experimentalTernaries: true,
  bracketSameLine: false,
  plugins: ['@trivago/prettier-plugin-sort-imports'],
  importOrderSeparation: true,
  importOrderSortSpecifiers: true,
};

const obj2: Config = {
  plugins: ['prettier-plugin-tailwindcss'],
  tailwindFunctions: ['cva', 'clsx', 'cn'],
};

const obj3: Config = merge(
  {
    plugins: ['prettier-plugin-astro'],
    overrides: [
      {
        files: '*.astro',
        options: { parser: 'astro' },
      },
    ],
  },
  obj1,
);

const expected1 = {
  arrowParens: 'always',
  bracketSameLine: false,
  bracketSpacing: true,
  experimentalTernaries: true,
  importOrderSeparation: true,
  importOrderSortSpecifiers: true,
  jsxSingleQuote: true,
  plugins: [
    '@trivago/prettier-plugin-sort-imports',
    'prettier-plugin-tailwindcss',
  ],
  semi: true,
  singleQuote: true,
  tabWidth: 2,
  tailwindFunctions: ['cva', 'clsx', 'cn'],
  trailingComma: 'all',
  useTabs: false,
};

const expected2 = {
  arrowParens: 'always',
  bracketSameLine: false,
  bracketSpacing: true,
  experimentalTernaries: true,
  importOrderSeparation: true,
  importOrderSortSpecifiers: true,
  jsxSingleQuote: true,
  plugins: [
    '@trivago/prettier-plugin-sort-imports',
    'prettier-plugin-astro',
    'prettier-plugin-tailwindcss',
  ],
  semi: true,
  singleQuote: true,
  tabWidth: 2,
  tailwindFunctions: ['cva', 'clsx', 'cn'],
  trailingComma: 'all',
  useTabs: false,
  overrides: [
    {
      files: '*.astro',
      options: { parser: 'astro' },
    },
  ],
};

test('merge handles normal input correctly', () => {
  expect(merge(obj1, obj2)).toStrictEqual(expected1);
  expect(merge(obj2, obj1)).toStrictEqual(expected1);
});

test('merge dedups', () => {
  expect(merge(obj3, obj1)).toStrictEqual(obj3);
});

test('merge multiple configs', () => {
  expect(merge(obj1, obj2, obj3)).toStrictEqual(expected2);
});
