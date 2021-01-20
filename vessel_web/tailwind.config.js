module.exports = {
  purge: {
    content: ['./public/index.html', './src/**/*.svelte'],
    options: {
      safelist: [/svelte-/],
    },
  },
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
  },
  variants: {
    extend: {
      transitionTimingFunction: ['hover', 'focus'],
    },
  },
  plugins: [],
}