import { defineBuildConfig } from 'unbuild';

export default defineBuildConfig({
  declaration: 'compatible',
  entries: [
    {
      builder: 'mkdist',
      declaration: true,
      input: './src/',
      outDir: './dist',
    },
  ],
  failOnWarn: true,
  sourcemap: true,
});
