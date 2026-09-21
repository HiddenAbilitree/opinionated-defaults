import type { OxlintConfig } from 'oxlint';

const config: OxlintConfig = {
  jsPlugins: [`eslint-plugin-solid`],
  rules: {
    'solid/jsx-no-duplicate-props': `error`,
    'solid/jsx-no-undef': [`error`, { typescriptEnabled: true }],
    'solid/jsx-uses-vars': `error`,
    'solid/no-unknown-namespaces': `off`,
    'solid/no-innerhtml': `error`,
    'solid/jsx-no-script-url': `error`,
    'solid/components-return-once': `warn`,
    'solid/no-destructure': `error`,
    'solid/prefer-for': `error`,
    'solid/reactivity': `warn`,
    'solid/event-handlers': `warn`,
    'solid/imports': `warn`,
    'solid/style-prop': `warn`,
    'solid/no-react-deps': `warn`,
    'solid/no-react-specific-props': `warn`,
    'solid/self-closing-comp': `warn`,
    'solid/no-array-handlers': `off`,
    'solid/prefer-show': `off`,
    'solid/no-proxy-apis': `off`,
    'solid/prefer-classlist': `off`,
  },
};

export default config;
