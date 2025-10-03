import type { Config } from '@docusaurus/types';
import { themes as prismThemes } from 'prism-react-renderer';

const config: Config = {
  // ——— Site basics ———
  title: 'FerricLink',
  tagline: 'Rust-first building blocks inspired by LangChain & LangGraph',
  url: 'https://ferrum-labs.github.io',
  baseUrl: '/FerricLink/', // GitHub Pages (project site)
  favicon: 'img/favicon.ico',
  organizationName: 'Ferrum-Labs',
  projectName: 'FerricLink',
  trailingSlash: false,
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  // ——— Content presets ———
  presets: [
    [
      'classic',
      {
        docs: {
          // Docusaurus content lives in docs/docs/
          path: 'docs',
          routeBasePath: 'docs',
          sidebarPath: require.resolve('./sidebars.ts'),
          editUrl: 'https://github.com/Ferrum-Labs/FerricLink/edit/main/docs/',
        },
        blog: {
          path: 'blog',
          routeBasePath: 'blog',
          showReadingTime: true,
        },
        theme: {
          // Custom CSS in docs/src/css/custom.css
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],

  // ——— Plugins / redirects ———
  plugins: [
    [
      '@docusaurus/plugin-client-redirects',
      {
        redirects: [
          // root → docs landing
          { from: '/', to: '/docs' },
        ],
      },
    ],
  ],

  // ——— Theme / UI ———
  themeConfig: {
    // Social share card (served from docs/static/img/og.png)
    image: 'img/og.png',

    metadata: [{ name: 'theme-color', content: '#0f172a' }],

    navbar: {
      title: 'FerricLink',
      // Logos placed in docs/static/img/
      logo: {
        alt: 'FerricLink',
        src: 'img/logo-light.svg',
        srcDark: 'img/logo-dark.svg',
      },
      items: [
        { to: '/docs', label: 'Docs', position: 'left' },
        // API (rustdoc) built into docs/static/api; /api/ shows version index; /api/latest/ points to latest
        { to: '/api/latest', label: 'API (Rust)', position: 'left' },
        { to: '/blog', label: 'Blog', position: 'left' },
        { href: 'https://github.com/Ferrum-Labs/FerricLink', label: 'GitHub', position: 'right' },
      ],
    },

    colorMode: {
      defaultMode: 'dark',
      respectPrefersColorScheme: true,
    },

    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash'],
    },

    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [{ label: 'Getting Started', to: '/docs' }],
        },
        {
          title: 'Community',
          items: [{ label: 'GitHub', href: 'https://github.com/Ferrum-Labs/FerricLink' }],
        },
      ],
      copyright: `© ${new Date().getFullYear()} Ferrum Labs`,
    },
  },
};

export default config;
