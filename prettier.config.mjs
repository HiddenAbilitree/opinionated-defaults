import { prettierConfigBase } from '@hiddenability/opinionated-defaults/prettier';
import lodash from 'lodash';

const { merge } = lodash;

const config = merge(prettierConfigBase);

export default config;
