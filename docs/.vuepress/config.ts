import path from 'node:path';

import { viteBundler } from '@vuepress/bundler-vite';
import { defaultTheme } from '@vuepress/theme-default';
import { defineUserConfig } from 'vuepress';

export default defineUserConfig({
  lang: 'en-US',
  title: 'rquickjs',
  description: 'Main project docs and browser playground for rquickjs',
  bundler: viteBundler({
    viteOptions: {
      resolve: {
        alias: {
          env: path.resolve(__dirname, '../playground-app/src/host-env.ts')
        }
      }
    }
  }),
  theme: defaultTheme({
    logo: '/playground/mark.svg',
    navbar: [
      { text: 'Guide', link: '/guide/wasm.html' },
      { text: 'Playground', link: '/playground/' }
    ]
  }),
  head: [['meta', { name: 'theme-color', content: '#0f172a' }]]
});
