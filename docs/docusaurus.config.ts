import type { Config } from '@docusaurus/types';
import { themes as prismThemes } from 'prism-react-renderer';

const config: Config = {
  title: 'FerricLink',
  tagline: 'Rust-first building blocks inspired by LangChain & LangGraph',
  url: 'https://ferrum-labs.github.io',
  baseUrl: '/FerricLink/',
  favicon: 'img/favicon.ico',
  organizationName: 'Ferrum-Labs',
  projectName: 'FerricLink',
  trailingSlash: false,

  // keep strict link checking
  onBrokenLinks: 'throw',

  // deprecation fix: move markdown hook here
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },

  i18n: { defaultLocale: 'en', locales: ['en'] },

  presets: [
    [
      'classic',
      {
        docs: {
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
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],

  // no redirects plugin needed

  themeConfig: {
    image: 'img/og.png',
    metadata: [{ name: 'theme-color', content: '#0f172a' }],

    navbar: {
      title: 'FerricLink',
      logo: {
        alt: 'FerricLink',
        src: 'img/logo-light.svg',
        srcDark: 'img/logo-dark.svg',
      },
      items: [
        { to: '/docs', label: 'Docs', position: 'left' },
        { to: '/api/latest', label: 'API (Rust)', position: 'left' },
        { to: '/blog', label: 'Blog', position: 'left' },
        { href: 'https://github.com/Ferrum-Labs/FerricLink', label: 'GitHub', position: 'right' },
      ],
    },

    colorMode: { defaultMode: 'dark', respectPrefersColorScheme: true },

    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash'],
    },

    footer: {
      style: 'dark',
      links: [
        { title: 'Docs', items: [{ label: 'Getting Started', to: '/docs' }] },
        { title: 'Community', items: [{ label: 'GitHub', href: 'https://github.com/Ferrum-Labs/FerricLink' }] }
      ],
      copyright: `© ${new Date().getFullYear()} Ferrum Labs`
    }
  }
};

export default config;
