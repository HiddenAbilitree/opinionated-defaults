import baseConfig from './base.ts';

const nextConfig = {
  ...baseConfig,
  plugins: ['prettier-plugin-tailwindcss'],
  tailwindStylesheet: './src/globals.css',
  tailwindFunctions: ['cva', 'clsx', 'cn'],
};

export default nextConfig;
