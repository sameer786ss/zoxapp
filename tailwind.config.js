/** @type {import('tailwindcss').Config} */
module.exports = {
    darkMode: 'class',
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                background: "hsl(var(--background))",
                foreground: "hsl(var(--foreground))",
                card: "hsl(var(--card))",
                "card-foreground": "hsl(var(--card-foreground))",
                popover: "hsl(var(--popover))",
                "popover-foreground": "hsl(var(--popover-foreground))",
                primary: "hsl(var(--primary))",
                "primary-foreground": "hsl(var(--primary-foreground))",
                secondary: "hsl(var(--secondary))",
                "secondary-foreground": "hsl(var(--secondary-foreground))",
                muted: "hsl(var(--muted))",
                "muted-foreground": "hsl(var(--muted-foreground))",
                accent: "hsl(var(--accent))",
                "accent-foreground": "hsl(var(--accent-foreground))",
                destructive: "hsl(var(--destructive))",
                "destructive-foreground": "hsl(var(--destructive-foreground))",
                border: "hsl(var(--border))",
                input: "hsl(var(--input))",
                ring: "hsl(var(--ring))",
                // Fluent Design Colors
                fluent: {
                    accent: "hsl(var(--fluent-accent))",
                    "accent-light": "hsl(var(--fluent-accent-light))",
                    surface: "hsl(var(--fluent-surface))",
                    layer: "hsl(var(--fluent-layer))",
                },
            },
            borderRadius: {
                lg: "var(--radius)",
                md: "calc(var(--radius) - 2px)",
                sm: "calc(var(--radius) - 4px)",
            },
            boxShadow: {
                "elevation-1": "0 2px 4px rgba(0,0,0,0.14), 0 0 2px rgba(0,0,0,0.12)",
                "elevation-2": "0 4px 8px rgba(0,0,0,0.14), 0 0 2px rgba(0,0,0,0.12)",
                "elevation-3": "0 8px 16px rgba(0,0,0,0.14), 0 0 2px rgba(0,0,0,0.12)",
                "elevation-4": "0 16px 32px rgba(0,0,0,0.14), 0 0 2px rgba(0,0,0,0.12)",
            },
            animation: {
                "accordion-down": "accordion-down 0.2s ease-out",
                "accordion-up": "accordion-up 0.2s ease-out",
                "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
                "fade-in": "fade-in 0.2s cubic-bezier(0.1, 0.9, 0.2, 1)",
                "slide-up": "slide-up 0.2s cubic-bezier(0.1, 0.9, 0.2, 1)",
                "scale-in": "scale-in 0.15s cubic-bezier(0.1, 0.9, 0.2, 1)",
                "shimmer": "shimmer 2s linear infinite",
            },
            keyframes: {
                "accordion-down": {
                    from: { height: "0" },
                    to: { height: "var(--radix-accordion-content-height)" },
                },
                "accordion-up": {
                    from: { height: "var(--radix-accordion-content-height)" },
                    to: { height: "0" },
                },
                "fade-in": {
                    from: { opacity: "0" },
                    to: { opacity: "1" },
                },
                "slide-up": {
                    from: { opacity: "0", transform: "translateY(8px)" },
                    to: { opacity: "1", transform: "translateY(0)" },
                },
                "scale-in": {
                    from: { opacity: "0", transform: "scale(0.95)" },
                    to: { opacity: "1", transform: "scale(1)" },
                },
                "shimmer": {
                    "0%": { backgroundPosition: "-200% 0" },
                    "100%": { backgroundPosition: "200% 0" },
                },
            },
            transitionTimingFunction: {
                fluent: "cubic-bezier(0.1, 0.9, 0.2, 1)",
                "fluent-decelerate": "cubic-bezier(0, 0, 0, 1)",
                "fluent-accelerate": "cubic-bezier(1, 0, 1, 1)",
            },
            transitionDuration: {
                "ultra-fast": "50ms",
                faster: "100ms",
                fast: "150ms",
                normal: "200ms",
                slow: "300ms",
                slower: "400ms",
            },
            fontFamily: {
                sans: ['"Segoe UI Variable"', '"Segoe UI"', '-apple-system', 'BlinkMacSystemFont', 'system-ui', 'sans-serif'],
            },
        },
    },
    plugins: [],
}
