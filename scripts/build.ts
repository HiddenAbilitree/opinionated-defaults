import { build, type BunPlugin, file, pathToFileURL, write } from 'bun';
import { consola } from 'consola';
import { colorize } from 'consola/utils';
import { rm } from 'node:fs/promises';
import { performance } from 'node:perf_hooks';
import { isolatedDeclaration } from 'oxc-transform';

await rm(`./dist/cli`, { force: true, recursive: true });
await rm(`./dist/eslint`, { force: true, recursive: true });
await rm(`./dist/prettier`, { force: true, recursive: true });

// courtesy of:
// https://github.com/oven-sh/bun/issues/5141#issuecomment-2595032410
const dts: BunPlugin = {
  name: `bun-plugin-oxc-transform-dts`,
  setup: (builder) => {
    const written = new Set<string>();
    if (!builder.config.root || !builder.config.outdir) return;

    const rootPath = pathToFileURL(builder.config.root).pathname;
    const outPath = pathToFileURL(builder.config.outdir).pathname;

    builder.onStart(() => written.clear());
    builder.onLoad({ filter: /\.ts$/ }, async (args) => {
      if (!args.path.startsWith(rootPath) || written.has(args.path)) return;
      written.add(args.path);
      const { code } = isolatedDeclaration(
        args.path,
        await file(args.path).text(),
      );
      await write(
        args.path
          .replace(new RegExp(`^${rootPath}`), outPath)
          .replace(/\.ts$/, `.d.ts`),
        code,
      );
    });
  },
};

performance.mark(`build_start`);

await build({
  entrypoints: [
    `./src/eslint/index.ts`,
    `./src/prettier/index.ts`,
    `./src/cli/index.ts`,
  ],
  minify: true,
  naming: `[dir]/[name].mjs`,
  outdir: `./dist`,
  packages: `external`,
  plugins: [dts],
  root: `src`,
  target: `node`,
});

performance.mark(`build_end`);

consola.success(
  `built in`,
  colorize(
    `bold`,
    `${performance.measure(`build`, `build_start`, `build_end`).duration.toFixed(2)}ms`,
  ) + `.`,
);
