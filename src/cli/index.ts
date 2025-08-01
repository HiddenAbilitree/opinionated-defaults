#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
const getExePath = () => {
  const arch = process.arch;

  let os = process.platform as string;
  let extension = ``;
  if ([`cygwin`, `win32`].includes(process.platform)) {
    os = `windows`;
    extension = `.exe`;
  }

  try {
    return fileURLToPath(
      import.meta.resolve(
        `@hiddenability/opinionated-defaults-${os}-${arch}/bin/bin${extension}`,
      ),
    );
  } catch {
    throw new Error(
      `Couldn't find application binary inside node_modules for ${os}-${arch}`,
    );
  }
};

const run = () => {
  const args = process.argv.slice(2);
  const processResult = spawnSync(getExePath(), args);

  process.exit(processResult.status ?? 0);
};

run();
