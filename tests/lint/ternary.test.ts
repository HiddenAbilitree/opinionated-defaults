import { test } from 'bun:test';

import { testEslint } from './utils';

test(`ternary`, async () => {
  const source = [
    `const condition = true;`,
    `const _thing = condition ? \`loooooooooooooooooooooooooooooooooooooooooooooong\` : \`looooooooooooooooooooooooooooooooooooooooooong\`;`,
  ].join(`\n`);

  const expected = [
    `const condition = true;`,
    `const _thing =`,
    `  condition ?`,
    `    \`loooooooooooooooooooooooooooooooooooooooooooooong\``,
    `  : \`looooooooooooooooooooooooooooooooooooooooooong\`;`,
    ``,
  ].join(`\n`);

  await testEslint(source, expected);
});
