import syntaxHighlight from "@11ty/eleventy-plugin-syntaxhighlight";

export default function (eleventyConfig) {
  eleventyConfig.addPlugin(syntaxHighlight);

  // Static assets (CSS, etc.) pass through untouched.
  eleventyConfig.addPassthroughCopy({ "src/assets": "assets" });
  eleventyConfig.addPassthroughCopy({ "src/.nojekyll": ".nojekyll" });
  eleventyConfig.addPassthroughCopy({ "src/CNAME": "CNAME" });
  eleventyConfig.addWatchTarget("src/assets");

  // Ordered navigation: pages declare `order` in front matter.
  eleventyConfig.addCollection("docs", (collectionApi) =>
    collectionApi
      .getFilteredByGlob("src/pages/*.md")
      .sort((a, b) => (a.data.order ?? 0) - (b.data.order ?? 0)),
  );

  // Zero-padded section index for the spec-sheet nav (01, 02, ...).
  eleventyConfig.addFilter("pad2", (n) => String(n).padStart(2, "0"));

  // Extract <h2>/<h3> headings from rendered HTML for the on-page TOC.
  // Relies on the markdown-it anchor slugs added below.
  eleventyConfig.addFilter("toc", (html) => {
    if (!html) return [];
    const re = /<h([23])[^>]*\sid="([^"]+)"[^>]*>(.*?)<\/h\1>/g;
    return [...html.matchAll(re)].map((m) => ({
      level: Number(m[1]),
      id: m[2],
      text: m[3]
        .replace(/<[^>]+>/g, "")
        .replace(/&amp;/g, "&")
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/&#39;|&apos;/g, "'")
        .replace(/&quot;/g, '"')
        .trim(),
    }));
  });

  // Render GitHub-style alerts (> [!NOTE], > [!IMPORTANT], ...) as styled callouts.
  const ALERT_RE = /^\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*/i;
  const slugify = (s) =>
    s
      .toLowerCase()
      .replace(/<[^>]+>/g, "")
      .replace(/[^\w\s-]/g, "")
      .trim()
      .replace(/\s+/g, "-");
  eleventyConfig.amendLibrary("md", (md) => {
    // Add id slugs to h2/h3 so the on-page TOC can link to them.
    md.core.ruler.push("heading_anchors", (state) => {
      const slugSeen = {}; // reset per render so slugs don't leak across pages
      for (let i = 0; i < state.tokens.length; i++) {
        const t = state.tokens[i];
        if (t.type !== "heading_open") continue;
        if (t.tag !== "h2" && t.tag !== "h3") continue;
        const inline = state.tokens[i + 1];
        if (inline?.type !== "inline") continue;
        let slug = slugify(inline.content);
        if (!slug) continue;
        if (slugSeen[slug] != null) slug = `${slug}-${++slugSeen[slug]}`;
        else slugSeen[slug] = 0;
        t.attrSet("id", slug);
      }
    });

    const defaultOpen =
      md.renderer.rules.blockquote_open ||
      ((tokens, idx, options, _env, self) =>
        self.renderToken(tokens, idx, options));

    md.core.ruler.push("github_alerts", (state) => {
      const tokens = state.tokens;
      for (let i = 0; i < tokens.length; i++) {
        if (tokens[i].type !== "blockquote_open") continue;
        // First inline token inside the blockquote carries the [!TYPE] marker.
        const inline = tokens[i + 2];
        if (inline?.type !== "inline") continue;
        const match = inline.content.match(ALERT_RE);
        if (!match) continue;
        const kind = match[1].toLowerCase();
        tokens[i].attrSet("class", `callout callout-${kind}`);
        tokens[i].meta = { alert: kind };
        // Strip the marker and its children from the rendered text.
        inline.content = inline.content.replace(ALERT_RE, "");
        if (inline.children?.length) {
          inline.children = inline.children.filter(
            (c, ci) => !(ci < 2 && ALERT_RE.test(c.content)),
          );
          if (inline.children[0]) {
            inline.children[0].content = inline.children[0].content.replace(
              ALERT_RE,
              "",
            );
          }
        }
      }
    });

    md.renderer.rules.blockquote_open = (tokens, idx, options, env, self) => {
      const meta = tokens[idx].meta;
      if (meta?.alert) {
        const label = meta.alert.charAt(0).toUpperCase() + meta.alert.slice(1);
        return `${defaultOpen(tokens, idx, options, env, self)}<p class="callout-title">${label}</p>`;
      }
      return defaultOpen(tokens, idx, options, env, self);
    };
  });

  return {
    // Custom domain (ntgcallsjs.arnabxd.me) serves from root.
    pathPrefix: process.env.PATH_PREFIX || "/",
    dir: {
      input: "src",
      output: "_site",
      includes: "_includes",
      data: "_data",
    },
    markdownTemplateEngine: "njk",
    htmlTemplateEngine: "njk",
  };
}
