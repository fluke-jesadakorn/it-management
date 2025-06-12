/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  darkMode: 'class',
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {
      colors: {
        primary: '#0052CC',
        secondary: '#7A8FFF',
        accent: '#00C7F4',
        dark: {
          primary: '#0A1929',
          secondary: '#132F4C',
          accent: '#173A5E'
        },
        pancake: {
          gradient1: 'from-primary via-secondary to-accent',
          gradient2: 'from-accent via-secondary to-primary'
        }
      },
      borderRadius: {
        'pancake': '24px',
      },
      boxShadow: {
        'pancake': '0px 4px 8px rgba(0, 0, 0, 0.1)',
        'pancake-hover': '0px 8px 16px rgba(0, 0, 0, 0.2)',
      },
      animation: {
        'gradient': 'gradient 8s linear infinite',
      },
      keyframes: {
        gradient: {
          '0%, 100%': {
            'background-size': '200% 200%',
            'background-position': 'left center'
          },
          '50%': {
            'background-size': '200% 200%',
            'background-position': 'right center'
          },
        },
      },
    },
  },
  plugins: [],
};
