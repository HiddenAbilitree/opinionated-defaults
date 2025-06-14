import { defineBuildConfig } from 'unbuild';

export default defineBuildConfig({
  entries: [
    {
      builder: 'mkdist',
      input: './src/',
      outDir: './dist',
      declaration: true,
    },
  ],
  declaration: 'compatible',
  sourcemap: true,
  failOnWarn: true,
});
