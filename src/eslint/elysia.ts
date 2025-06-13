import baseConfig from './base.ts';
import tsConfig from './typescript.ts';
import functionalConfig from './functional.ts';

const elysiaConfig = [...baseConfig, ...tsConfig, ...functionalConfig];

export default elysiaConfig;
