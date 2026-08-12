---
name: Modern Professional
colors:
  surface: '#f9f9ff'
  surface-dim: '#d7dae3'
  surface-bright: '#f9f9ff'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#f1f3fc'
  surface-container: '#ebedf7'
  surface-container-high: '#e6e8f1'
  surface-container-highest: '#e0e2eb'
  on-surface: '#181c22'
  on-surface-variant: '#414753'
  inverse-surface: '#2d3037'
  inverse-on-surface: '#eef0fa'
  outline: '#717785'
  outline-variant: '#c1c6d5'
  surface-tint: '#005db8'
  primary: '#005ab4'
  on-primary: '#ffffff'
  primary-container: '#0a73e0'
  on-primary-container: '#fefcff'
  inverse-primary: '#aac7ff'
  secondary: '#465f88'
  on-secondary: '#ffffff'
  secondary-container: '#b6d0ff'
  on-secondary-container: '#3f5881'
  tertiary: '#964400'
  on-tertiary: '#ffffff'
  tertiary-container: '#bd5700'
  on-tertiary-container: '#fffbff'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#d6e3ff'
  primary-fixed-dim: '#aac7ff'
  on-primary-fixed: '#001b3e'
  on-primary-fixed-variant: '#00458d'
  secondary-fixed: '#d6e3ff'
  secondary-fixed-dim: '#aec7f7'
  on-secondary-fixed: '#001b3d'
  on-secondary-fixed-variant: '#2d476f'
  tertiary-fixed: '#ffdbc9'
  tertiary-fixed-dim: '#ffb68c'
  on-tertiary-fixed: '#321200'
  on-tertiary-fixed-variant: '#763400'
  background: '#f9f9ff'
  on-background: '#181c22'
  surface-variant: '#e0e2eb'
typography:
  headline-lg:
    fontFamily: Inter
    fontSize: 32px
    fontWeight: '600'
    lineHeight: 40px
  headline-md:
    fontFamily: Inter
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
  body-lg:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  label-md:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  unit: 2px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
---

# Design System Document

## Brand & Style
The brand identity has shifted from a warm, high-energy aesthetic to a cool, professional, and reliable persona. The style is **Corporate / Modern**, emphasizing clarity, stability, and trust. It utilizes a balanced composition with ample whitespace and a focused color palette to ensure a productive and calm user experience. The target audience includes professional users who value efficiency and a clean, Inter-based typographic interface.

## Colors
The palette is anchored by a vibrant blue primary color (#1275e2), conveying technology and reliability. The secondary color is a muted, desaturated blue-grey (#5f78a3) used for supporting UI elements. A tertiary orange-brown (#c55b00) provides a controlled accent for calls to action or specific highlights without overwhelming the professional tone. The neutral palette is a balanced grey (#74777f) that ensures high legibility and clean structural separation.

## Typography
The system uses **Inter** across all levels to provide a highly legible, neutral, and modern appearance. Headlines use a semi-bold weight to establish hierarchy, while body text maintains a standard weight for long-form reading. The typography scales from 12px for labels up to 32px for large headlines, ensuring a clear information architecture.

## Layout & Spacing
The layout follows a fluid grid system with a spacing rhythm based on a 2px base unit. Standardized margins of 24px and gutters of 16px are used to align components. The interface adapts to mobile devices by reducing margins to 16px and stacking columns vertically where necessary.

## Elevation & Depth
Visual hierarchy is achieved through tonal layers and subtle ambient shadows. Surfaces use light grey backgrounds to separate the container from the canvas. Depth is conveyed through low-opacity, diffused shadows that lift active elements like cards and menus without creating excessive visual noise.

## Shapes
The design utilizes a **Rounded** shape language. Standard UI components like buttons and input fields feature a 0.5rem (8px) corner radius. Larger containers like cards use a 1rem (16px) radius, providing a friendly and approachable feel that avoids the starkness of sharp corners.

## Components
- **Buttons:** Use the primary blue (#1275e2) for main actions with rounded corners (8px). 
- **Input Fields:** Outlined with neutral grey (#74777f), utilizing Inter for placeholder and input text.
- **Cards:** Elevated with soft shadows and 16px rounded corners to group related content.
- **Chips:** Small, rounded elements using the secondary blue-grey for categorization.