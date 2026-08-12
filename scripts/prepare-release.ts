import assert from 'node:assert/strict';
import { readdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

const parseJsonObject = (contents: string) => {
  const value: unknown = JSON.parse(contents);
  assert(typeof value === `object` && value !== null && !Array.isArray(value));
  return value;
};

export const prepareRelease = async (
  packagePath: string,
  platformPackagesPath: string,
  version: string,
) => {
  const artifactDirectories = await readdir(platformPackagesPath);
  const packageNames = (
    await Promise.all(
      artifactDirectories.map(async (directory) => {
        const contents = await readFile(
          join(platformPackagesPath, directory, `package.json`),
          `utf8`,
        );
        const platformPackage = parseJsonObject(contents);
        const name = `name` in platformPackage ? platformPackage.name : undefined;
        assert(typeof name === `string`);
        return name;
      }),
    )
  ).toSorted();

  const packageJson = parseJsonObject(await readFile(packagePath, `utf8`));
  const releasePackageJson = {
    ...packageJson,
    optionalDependencies: Object.fromEntries(packageNames.map((name) => [name, version])),
    version,
  };

  await writeFile(packagePath, `${JSON.stringify(releasePackageJson, undefined, 2)}\n`);
};

if (import.meta.main) {
  const [, , packagePath, platformPackagesPath, version] = process.argv;
  assert(packagePath !== undefined);
  assert(platformPackagesPath !== undefined);
  assert(version !== undefined);
  await prepareRelease(packagePath, platformPackagesPath, version);
}
