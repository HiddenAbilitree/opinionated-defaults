import { type Config } from 'prettier';

const tailwindConfig: Config = {
  plugins: ['prettier-plugin-tailwindcss'],
  tailwindFunctions: ['cva', 'clsx', 'cn'],
};

export default tailwindConfig;
