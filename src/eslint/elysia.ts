import baseConfig from './base';
import tsConfig from './typescript';
import functionalConfig from './functional';

const elysiaConfig = [...baseConfig, ...tsConfig, ...functionalConfig];

export default elysiaConfig;
