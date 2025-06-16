import {
  prettierConfigBase,
  prettierConfigTailwind,
} from '@hiddenability/opinionated-defaults/prettier';
import lodash from 'lodash';

const { merge } = lodash;

const config = merge(prettierConfigTailwind, prettierConfigBase);

export default config;
