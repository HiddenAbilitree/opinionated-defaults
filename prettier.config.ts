import { type Config } from 'prettier';
import { prettierBaseConfig } from './src/index.ts';

const config: Config = { ...prettierBaseConfig };

export default config;
