import { type Config } from 'prettier';
import baseConfig from './base';

const nextConfig: Config = {
  ...baseConfig,
  plugins: ['prettier-plugin-tailwindcss'],
  tailwindStylesheet: './src/globals.css',
  tailwindFunctions: ['cva', 'clsx', 'cn'],
};

export default nextConfig;
