/** Tailwind config mirrored from the CDN inline config previously in ui/index.html.
 *  Used only at build time (npx tailwindcss) to emit a static stylesheet so the
 *  production UI has zero runtime CDN dependency. */
/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "class",
  content: ["./index.html", "./app.js"],
  theme: {
    extend: {
      colors: {
        "inverse-surface": "#2d3037",
        "tertiary-container": "#bd5700",
        "outline": "#717785",
        "on-secondary-fixed-variant": "#2d476f",
        "inverse-on-surface": "#eef0fa",
        "secondary-container": "#b6d0ff",
        "on-primary-container": "#fefcff",
        "on-secondary-container": "#3f5881",
        "on-error-container": "#93000a",
        "surface-bright": "#f9f9ff",
        "on-primary": "#ffffff",
        "on-tertiary": "#ffffff",
        "surface-container-lowest": "#ffffff",
        "primary-fixed": "#d6e3ff",
        "on-secondary-fixed": "#001b3d",
        "on-tertiary-container": "#fffbff",
        "on-surface": "#181c22",
        "on-background": "#181c22",
        "surface-container": "#ebedf7",
        "surface-container-highest": "#e0e2eb",
        "on-error": "#ffffff",
        "on-tertiary-fixed": "#321200",
        "on-secondary": "#ffffff",
        "surface-variant": "#e0e2eb",
        "surface": "#f9f9ff",
        "primary": "#005ab4",
        "on-surface-variant": "#414753",
        "error": "#ba1a1a",
        "surface-tint": "#005db8",
        "background": "#f9f9ff",
        "secondary": "#465f88",
        "inverse-primary": "#aac7ff",
        "tertiary-fixed": "#ffdbc9",
        "on-primary-fixed-variant": "#00458d",
        "secondary-fixed": "#d6e3ff",
        "surface-container-high": "#e6e8f1",
        "primary-fixed-dim": "#aac7ff",
        "outline-variant": "#c1c6d5",
        "on-tertiary-fixed-variant": "#763400",
        "tertiary": "#964400",
        "secondary-fixed-dim": "#aec7f7",
        "surface-container-low": "#f1f3fc",
        "primary-container": "#0a73e0",
        "on-primary-fixed": "#001b3e",
        "tertiary-fixed-dim": "#ffb68c",
        "surface-dim": "#d7dae3",
        "error-container": "#ffdad6"
      },
      borderRadius: {
        DEFAULT: "0.25rem",
        lg: "0.5rem",
        xl: "0.75rem",
        full: "9999px"
      },
      spacing: {
        xs: "4px",
        sm: "8px",
        md: "16px",
        lg: "24px",
        unit: "2px",
        xl: "32px",
        "container-margin-mobile": "20px",
        "container-margin-desktop": "40px",
        "glass-padding": "24px"
      },
      fontFamily: {
        "headline-md": ["Plus Jakarta Sans", "Inter"],
        "headline-lg": ["Plus Jakarta Sans", "Inter"],
        "display-lg-mobile": ["Plus Jakarta Sans"],
        "display-lg": ["Plus Jakarta Sans"],
        "body-md": ["Plus Jakarta Sans", "Inter"],
        "body-lg": ["Plus Jakarta Sans", "Inter"],
        "label-md": ["Inter"],
        "label-sm": ["Inter"]
      },
      fontSize: {
        "headline-md": ["24px", { lineHeight: "32px", fontWeight: "600" }],
        "headline-lg": ["32px", { lineHeight: "40px", fontWeight: "600" }],
        "body-md": ["14px", { lineHeight: "20px", fontWeight: "400" }],
        "body-lg": ["16px", { lineHeight: "24px", fontWeight: "400" }],
        "display-lg-mobile": ["32px", { lineHeight: "40px", letterSpacing: "-0.02em", fontWeight: "700" }],
        "display-lg": ["48px", { lineHeight: "56px", letterSpacing: "-0.02em", fontWeight: "700" }],
        "label-md": ["12px", { lineHeight: "16px", fontWeight: "500" }],
        "label-sm": ["12px", { lineHeight: "16px", letterSpacing: "0.05em", fontWeight: "600" }]
      }
    }
  }
};
