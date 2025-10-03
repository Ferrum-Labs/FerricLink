import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docs: [
    {
      type: 'category',
      label: 'Getting Started',
      collapsible: false,
      items: ['intro', 'install', 'quickstart']
    },
    {
      type: 'category',
      label: 'Guides',
      items: ['guides/pipelines', 'guides/state', 'guides/rag']
    },
    {
      type: 'category',
      label: 'Reference',
      items: ['reference/config', 'reference/cli']
    }
  ]
};

export default sidebars;
