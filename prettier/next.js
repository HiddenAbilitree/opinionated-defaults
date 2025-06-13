import baseConfig from "./base.js";

/**
 * @type {import("prettier").Config}
 */
const nextConfig = {
  ...baseConfig,
  plugins: ["prettier-plugin-tailwindcss"],
  tailwindStylesheet: './src/globals.css',
  tailwindFunctions: ["cva", 'clsx', 'cn'],
};

export default nextConfig;
