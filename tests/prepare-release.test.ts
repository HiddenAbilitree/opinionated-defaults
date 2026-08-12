import { expect, test } from 'bun:test';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { prepareRelease } from '../scripts/prepare-release';

test(`prepares the release manifest from platform package artifacts`, async () => {
  const directory = await mkdtemp(join(tmpdir(), `opinionated-defaults-`));
  const packagePath = join(directory, `package.json`);
  const artifactsPath = join(directory, `artifacts`);
  const platformPackages = [
    `@hiddenability/opinionated-defaults-linux-x64`,
    `@hiddenability/opinionated-defaults-darwin-arm64`,
  ];

  await Promise.all(
    platformPackages.map(async (name, index) => {
      const artifactPath = join(artifactsPath, `${index}`);
      await mkdir(artifactPath, { recursive: true });
      await writeFile(join(artifactPath, `package.json`), JSON.stringify({ name }));
    }),
  );
  await writeFile(
    packagePath,
    JSON.stringify({
      name: `@hiddenability/opinionated-defaults`,
      optionalDependencies: { stale: `0.0.1` },
      version: `0.0.0`,
    }),
  );

  await prepareRelease(packagePath, artifactsPath, `1.2.3`);

  expect(JSON.parse(await readFile(packagePath, `utf8`))).toEqual({
    name: `@hiddenability/opinionated-defaults`,
    optionalDependencies: {
      '@hiddenability/opinionated-defaults-darwin-arm64': `1.2.3`,
      '@hiddenability/opinionated-defaults-linux-x64': `1.2.3`,
    },
    version: `1.2.3`,
  });

  await rm(directory, { force: true, recursive: true });
});
