/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        bg: {
          DEFAULT: "#0a0410",
          deep: "#06020a",
          card: "#1c1224",
          glass: "rgb(28 18 36 / 0.65)",
          elevated: "#241830",
        },
        accent: {
          DEFAULT: "#f4b942",
          light: "#ffd270",
          dark: "#c89530",
        },
        text: {
          primary: "#f5e8d4",
          secondary: "#c4b5cd",
          muted: "#7a6e87",
        },
        success: "#7dc580",
        danger: "#c8504a",
        border: {
          DEFAULT: "rgb(255 255 255 / 0.06)",
          light: "rgb(255 255 255 / 0.10)",
          accent: "rgb(244 185 66 / 0.3)",
        },
        discord: "#5865f2",
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'Consolas', 'monospace'],
        display: ['"Space Grotesk"', 'Inter', 'system-ui', 'sans-serif'],
      },
      boxShadow: {
        accent: '0 8px 32px -8px rgb(244 185 66 / 0.5)',
        'accent-lg': '0 16px 48px -12px rgb(244 185 66 / 0.6)',
        glass: '0 16px 48px -8px rgb(0 0 0 / 0.4)',
      },
      animation: {
        'fade-in': 'fadeIn 0.4s ease-out',
        'slide-up': 'slideUp 0.5s cubic-bezier(0.34, 1.56, 0.64, 1)',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        fadeIn: { '0%': { opacity: 0 }, '100%': { opacity: 1 } },
        slideUp: {
          '0%': { opacity: 0, transform: 'translateY(20px)' },
          '100%': { opacity: 1, transform: 'translateY(0)' },
        },
      },
    },
  },
  plugins: [],
}
