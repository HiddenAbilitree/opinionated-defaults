import baseConfig from "./base.js";

/**
 * @type {import("prettier").Config}
 */
const astroConfig = {
  ...baseConfig,
  plugins: ["prettier-plugin-astro", "prettier-plugin-tailwindcss"],
  tailwindStylesheet: './src/styles/global.css',
  tailwindFunctions: ["cva", 'clsx', 'cn'],
  overrides: [
    {
      files: '*.astro',
      options: { parser: 'astro' },
    },
  ],
};

export default astroConfig;
