import lodash from 'lodash';

export { default as prettierConfigBase } from './base';
export { default as prettierConfigAstro } from './astro';
export { default as prettierConfigTailwind } from './tailwind';
export const { merge } = lodash;
