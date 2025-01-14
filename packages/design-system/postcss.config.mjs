/** @type {import('postcss-load-config').Config} */
const config = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};

/** @type {import('postcss-load-config').Config} */
export const configWithoutAutoprefixer = {
  plugins: {
    tailwindcss: {},
  },
}

export default config;
