import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLlmsTxt from 'starlight-llms-txt';
import starlightPageActions from 'starlight-page-actions';
import gruvbox from 'starlight-theme-gruvbox';

const site = process.env.SITE_URL || process.env.CF_PAGES_URL || 'https://micro-wakeword.pages.dev';
const base = process.env.BASE_PATH || '/';

export default defineConfig({
  site,
  base,
  integrations: [
    starlight({
      title: 'micro-wakeword / guide',
      description: 'Local, streaming wake-word detection for Rust.',
      favicon: '/favicon.svg',
      lastUpdated: true,
      editLink: {
        baseUrl: 'https://github.com/akash-kamat/microwakeword-rs/edit/main/website/',
      },
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/akash-kamat/microwakeword-rs' },
      ],
      customCss: ['./src/styles/custom.css'],
      head: [
        { tag: 'meta', attrs: { name: 'theme-color', content: '#282828' } },
        { tag: 'meta', attrs: { property: 'og:type', content: 'website' } },
        { tag: 'meta', attrs: { property: 'og:title', content: 'micro-wakeword handbook' } },
        { tag: 'meta', attrs: { property: 'og:description', content: 'Build local wake-word detection into Rust apps.' } },
      ],
      plugins: [
        gruvbox(),
        starlightLlmsTxt({
          projectName: 'micro-wakeword',
          description: 'A Rust library for local microWakeWord-compatible wake-word detection.',
          details: 'Covers the high-level microphone Listener, low-level PCM Detector, models, tuning, runtimes, errors, and deployment patterns.',
        }),
        starlightPageActions({
          position: 'page-title',
          actions: {
            markdown: true,
            chatgpt: true,
            claude: true,
            perplexity: true,
          },
          share: true,
        }),
      ],
      sidebar: [
        {
          label: 'Start here',
          items: [
            { label: 'Welcome', slug: 'index' },
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'Your first listener', slug: 'getting-started/first-listener' },
            { label: 'Choose your API', slug: 'getting-started/choose-api' },
          ],
        },
        {
          label: 'Microphone listener',
          items: [
            { label: 'Select a microphone', slug: 'listener/microphones' },
            { label: 'Cooldowns', slug: 'listener/cooldowns' },
            { label: 'Disconnect & reconnect', slug: 'listener/reconnection' },
            { label: 'Performance', slug: 'listener/performance' },
          ],
        },
        {
          label: 'Low-level detector',
          items: [
            { label: 'Bring your own audio', slug: 'detector/custom-audio' },
            { label: 'PCM files', slug: 'detector/pcm-files' },
            { label: 'Reset & stream boundaries', slug: 'detector/reset' },
          ],
        },
        {
          label: 'Models & tuning',
          items: [
            { label: 'Model JSON', slug: 'models/config-json' },
            { label: 'Model without JSON', slug: 'models/model-only' },
            { label: 'Threshold & sliding window', slug: 'models/tuning' },
            { label: 'Compatibility', slug: 'models/compatibility' },
          ],
        },
        {
          label: 'Library reference',
          items: [
            { label: 'Configuration', slug: 'reference/configuration' },
            { label: 'TensorFlow Lite runtime', slug: 'reference/runtime' },
            { label: 'Errors', slug: 'reference/errors' },
            { label: 'Feature flags & platforms', slug: 'reference/platforms' },
            { label: 'Runnable examples', slug: 'reference/examples' },
            { label: 'Troubleshooting', slug: 'reference/troubleshooting' },
          ],
        },
      ],
    }),
  ],
});
