import { type Config } from 'prettier';

const sortConfig: Config = {
  importOrderSeparation: true,
  importOrderSortSpecifiers: true,
  plugins: [`@trivago/prettier-plugin-sort-imports`],
};

export default sortConfig;
