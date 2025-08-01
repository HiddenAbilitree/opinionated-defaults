#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import process from 'node:process';

const getExePath = () => {
  const arch = process.arch;

  let os = process.platform as string;
  let extension = ``;
  if ([`cygwin`, `win32`].includes(process.platform)) {
    os = `windows`;
    extension = `.exe`;
  }

  try {
    return import.meta.resolve(
      `opinionated-defaults-${os}-${arch}/bin/bin${extension}`,
    );
  } catch {
    throw new Error(
      `Couldn't find application binary inside node_modules for ${os}-${arch}`,
    );
  }
};

const run = () => {
  const args = process.argv.slice(2);
  const processResult = spawnSync(getExePath(), args, { stdio: `inherit` });
  process.exit(processResult.status ?? 0);
};

run();
