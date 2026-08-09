import type { OxlintConfig } from 'oxlint';

const config: OxlintConfig = {
  plugins: [`react`, `react-perf`],
  rules: {
    'react/exhaustive-deps': `error`,
    'react/forward-ref-uses-ref': `error`,
    'react/iframe-missing-sandbox': `warn`,
    'react/jsx-key': `error`,
    'react/jsx-no-comment-textnodes': `warn`,
    'react/jsx-no-duplicate-props': `error`,
    'react/jsx-no-script-url': `warn`,
    'react/jsx-no-undef': `error`,
    'react/jsx-props-no-spread-multi': `error`,
    'react/no-children-prop': `error`,
    'react/no-danger-with-children': `error`,
    'react/no-did-mount-set-state': `error`,
    'react/no-did-update-set-state': `error`,
    'react/no-direct-mutation-state': `error`,
    'react/no-find-dom-node': `error`,
    'react/no-is-mounted': `error`,
    'react/no-namespace': `warn`,
    'react/no-render-return-value': `error`,
    'react/no-string-refs': `error`,
    'react/no-this-in-sfc': `error`,
    'react/no-unsafe': `error`,
    'react/no-unstable-nested-components': `warn`,
    'react/no-will-update-set-state': `error`,
    'react/react-in-jsx-scope': `off`,
    'react/style-prop-object': `warn`,
    'react/void-dom-elements-no-children': `error`,
  },
};

export default config;
