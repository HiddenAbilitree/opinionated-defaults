import { expect } from 'bun:test';
import { ESLint } from 'eslint';
import { fileURLToPath } from 'node:url';

const configPath = fileURLToPath(new URL(`./eslint.config.ts`, import.meta.url));

export const testEslint = async (source: string, expected: string) => {
  const eslint = new ESLint({ fix: true, overrideConfigFile: configPath });
  const results = await eslint.lintText(source, {
    filePath: fileURLToPath(import.meta.url),
  });

  const formatted = results[0]?.output ?? source;

  expect(formatted).toBe(expected);
};
