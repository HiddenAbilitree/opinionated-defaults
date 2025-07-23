import { type BunPlugin } from 'bun';
import { consola } from 'consola';
import { colorize } from 'consola/utils';
import { rm } from 'node:fs/promises';
import { isolatedDeclaration } from 'oxc-transform';

await rm(`./dist`, { force: true, recursive: true });

performance.mark(`build_start`);

// courtesy of:
// https://github.com/oven-sh/bun/issues/5141#issuecomment-2595032410
const dts: BunPlugin = {
  name: `bun-plugin-oxc-transform-dts`,
  setup: (builder) => {
    const written = new Set<string>();
    if (!builder.config.root || !builder.config.outdir) return;

    const rootPath = Bun.pathToFileURL(builder.config.root).pathname;
    const outPath = Bun.pathToFileURL(builder.config.outdir).pathname;

    builder.onStart(() => written.clear());
    builder.onLoad({ filter: /\.ts$/ }, async (args) => {
      if (!args.path.startsWith(rootPath) || written.has(args.path)) return;
      written.add(args.path);
      const { code } = isolatedDeclaration(
        args.path,
        await Bun.file(args.path).text(),
      );
      await Bun.write(
        args.path
          .replace(new RegExp(`^${rootPath}`), outPath)
          .replace(/\.ts$/, `.d.ts`),
        code,
      );
    });
  },
};

await Bun.build({
  entrypoints: [`./src/eslint/index.ts`, `./src/prettier/index.ts`],
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
