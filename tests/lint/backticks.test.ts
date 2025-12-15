import { test } from 'bun:test';

import { testEslint } from './utils';

test.failing(`backticks`, async () => {
  const source = [
    `import _ from './backticks.json' with { type: 'json' };`,
    `type __shouldBeSingleQuotes = typeof import('fs');`,
    `const _shouldBeBackticks = 'yooo';`,
    ``,
  ].join(`\n`);

  const expected = [
    `import _ from './backticks.json' with { type: 'json' };`,
    `type __shouldBeSingleQuotes = typeof import('fs');`,
    `const _shouldBeBackticks = \`yooo\`;`,
    ``,
  ].join(`\n`);

  await testEslint(source, expected);
});
