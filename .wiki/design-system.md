# Design System - 2026 Trends

## Design Philosophy

Modern, premium feel with dark mode as default. Glassmorphism for depth and layering. Clean, minimal interface that lets content (TV) be the focus.

## Core Trends for 2026

### 1. Dark Mode Excellence

Dark mode is now the **default**, not an afterthought.

```css
:root {
  /* Dark mode palette - never pure black */
  --bg-primary: #0a0a0a;
  --bg-secondary: #141414;
  --bg-tertiary: #1a1a1a;

  /* Text - off-white, not pure white */
  --text-primary: #f5f5f5;
  --text-secondary: #a3a3a3;
  --text-muted: #737373;

  /* Accents - slightly desaturated for dark mode */
  --accent-primary: #6366f1;  /* Indigo */
  --accent-secondary: #8b5cf6; /* Violet */

  /* Shadows - subtle or none in dark mode */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
}
```

**Key Rules:**
- Never use pure black (#000) - use dark grays (#0a0a0a)
- Text minimum 4.5:1 contrast ratio (WCAG 2.1)
- Accent colors slightly desaturated
- Shadows darker than surface, not lighter

### 2. Advanced Glassmorphism

Refined frosted glass effect - subtle, not garish.

```css
.glass-panel {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
}

/* Dark glass for dark mode */
.glass-panel-dark {
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.08);
}
```

**2026 Refinements:**
- Variable blur based on depth
- Subtle noise textures
- Gradient borders (soft glow)
- Less saturation than 2021-2023 era

### 3. Bento Grid Layouts

Content organized in card-based grid system.

```css
.bento-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.bento-card {
  border-radius: 12px;
  overflow: hidden;
  transition: transform 0.2s ease;
}

.bento-card:hover {
  transform: translateY(-2px);
}
```

### 4. Spatial Design & Depth

Layered interfaces with clear depth hierarchy.

```css
/* Surface elevation system */
.surface-0 { background: var(--bg-primary); }
.surface-1 { background: var(--bg-secondary); }
.surface-2 { background: var(--bg-tertiary); }

/* Floating elements */
.floating {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}
```

## Component Patterns

### Navigation
- Sidebar with glassmorphic background
- Icon + text labels
- Active state with accent glow

### Cards (Channel/Recording)
- 16:9 aspect ratio thumbnails
- Glassmorphic overlay for metadata
- Hover reveals actions

### Video Player
- Minimal controls, auto-hide
- Progress bar with buffer indicator
- Glassmorphic control bar

### Guide Grid
- Timeline header (sticky)
- Channel column (sticky)
- Program blocks with color-coded categories

## Color Palette

```css
/* Primary Colors */
--indigo-500: #6366f1;
--violet-500: #8b5cf6;

/* Status Colors */
--success: #22c55e;
--warning: #f59e0b;
--error: #ef4444;
--info: #3b82f6;

/* Category Colors (for guide) */
--category-movie: #ef4444;
--category-sports: #22c55e;
--category-news: #3b82f6;
--category-series: #8b5cf6;
```

## Typography

```css
/* Font Stack */
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
--font-mono: 'JetBrains Mono', monospace;

/* Scale */
--text-xs: 0.75rem;   /* 12px */
--text-sm: 0.875rem;  /* 14px */
--text-base: 1rem;    /* 16px */
--text-lg: 1.125rem;  /* 18px */
--text-xl: 1.25rem;   /* 20px */
--text-2xl: 1.5rem;   /* 24px */
--text-3xl: 1.875rem; /* 30px */
```

## Animation Guidelines

- Transitions: 150-300ms ease
- Use `transform` and `opacity` for performance
- Subtle micro-interactions on hover/focus
- No jarring animations

## Accessibility

- WCAG 2.1 AA compliance minimum
- Focus visible indicators
- Keyboard navigation support
- Screen reader labels for icons
- Reduced motion preference support

## References

- [Apple Liquid Glass](https://developer.apple.com/design/) - iOS/macOS design language
- [Tailwind CSS](https://tailwindcss.com) - Utility-first CSS framework
- [Radix UI](https://www.radix-ui.com/) - Accessible component primitives
