import { defineClientConfig } from 'vuepress/client';

import PlaygroundShell from './components/PlaygroundShell.vue';

export default defineClientConfig({
  enhance({ app }) {
    app.component('PlaygroundShell', PlaygroundShell);
  }
});
