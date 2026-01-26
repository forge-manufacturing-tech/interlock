/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                // Factory Industrial Dark Theme
                industrial: {
                    // Copper/Rust accent - primary brand color
                    copper: {
                        50: '#fef5f1',
                        100: '#fde8df',
                        200: '#fbd1be',
                        300: '#f7ae8d',
                        400: '#f28a5c',
                        500: '#cc6731',  // Main accent
                        600: '#b85525',
                        700: '#9a4520',
                        800: '#7d3a1e',
                        900: '#66321d',
                    },
                    // Steel/Metal grays - for backgrounds and surfaces
                    steel: {
                        50: '#f8f9fa',
                        100: '#e9ecef',
                        200: '#c8cdd2',
                        300: '#a8b0b8',
                        400: '#6c757d',
                        500: '#495057',
                        600: '#343a40',
                        700: '#2b2f35',
                        800: '#1f2228',  // Main background
                        900: '#13151a',  // Darker elements
                        950: '#0a0b0e',  // Deepest black
                    },
                    // Concrete - for borders and dividers
                    concrete: {
                        DEFAULT: '#3a3d42',
                        light: '#4a4e54',
                        dark: '#2a2d32',
                    },
                    // Warning/Caution yellow
                    caution: {
                        DEFAULT: '#ffc107',
                        dark: '#ff9800',
                    },
                    // Success/Active green
                    active: {
                        DEFAULT: '#4caf50',
                        dark: '#388e3c',
                    },
                    // Error/Alert red
                    alert: {
                        DEFAULT: '#f44336',
                        dark: '#d32f2f',
                    },
                }
            },
            fontFamily: {
                mono: ['JetBrains Mono', 'Consolas', 'Monaco', 'monospace'],
                sans: ['Inter', 'system-ui', 'sans-serif'],
            },
            boxShadow: {
                'glow-copper': '0 0 20px rgba(204, 103, 49, 0.3)',
                'glow-copper-lg': '0 0 30px rgba(204, 103, 49, 0.5)',
                'inset-dark': 'inset 0 2px 4px 0 rgba(0, 0, 0, 0.3)',
            },
        },
    },
    plugins: [],
}
