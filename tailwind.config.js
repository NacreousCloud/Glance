/** @type {import('\''tailwindcss'\'').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        ios: {
          bg: {
            light: "rgba(255, 255, 255, 0.7)",
            dark: "rgba(28, 28, 30, 0.7)",
          },
          label: {
            primary: "#000000",
            secondary: "#3c3c4399",
            primaryDark: "#FFFFFF",
            secondaryDark: "#ebebf599",
          },
          system: {
            blue: "#007aff",
            green: "#34c759",
            red: "#ff3b30",
            gray: "#8e8e93",
          },
          separator: {
            light: "#3c3c434d",
            dark: "#54545899",
          }
        }
      },
      borderRadius: {
        "ios": "12px",
        "ios-lg": "16px",
      },
      backdropBlur: {
        "ios": "20px",
      }
    },
  },
  plugins: [],
};

