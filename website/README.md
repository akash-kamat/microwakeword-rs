# micro-wakeword handbook

The Astro Starlight website for `micro-wakeword`.

## Local development

```bash
npm install
npm run dev
```

Run the production checks with `npm run build`. Output is written to `dist/`.

## Deployments

- GitHub Pages is deployed by `.github/workflows/docs.yml` with `BASE_PATH=/microwakeword-rs`.
- Cloudflare Pages is deployed by `.github/workflows/cloudflare-pages.yml` after its one-time credentials and project configuration are added. It publishes at the domain Cloudflare assigns with no base path.

For Cloudflare Pages, first create a Pages project in Cloudflare. Then add these GitHub repository settings:

| GitHub setting | Value |
| --- | --- |
| Actions secret `CLOUDFLARE_API_TOKEN` | Token with Cloudflare Pages edit permission |
| Actions secret `CLOUDFLARE_ACCOUNT_ID` | Cloudflare account ID |
| Actions variable `CLOUDFLARE_PAGES_PROJECT_NAME` | The Pages project name |
| Actions variable `CLOUDFLARE_PAGES_SITE_URL` | Production URL, such as `https://micro-wakeword.pages.dev` |

Until `CLOUDFLARE_PAGES_PROJECT_NAME` exists, the Cloudflare workflow skips cleanly instead of making pushes fail. Alternatively, Cloudflare's native Git integration can build with root directory `website`, command `npm run build`, output `dist`, and Node 24; in that case the Cloudflare workflow can remain disabled.
