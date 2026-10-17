# sitemap

Parsing sitemap XML and assembling the files of one crawl into a single tree.

- `parse`: turns one sitemap file into `Parsed`, either a `<sitemapindex>`'s
  child references or a `<urlset>`'s page entries.
- `build_tree`: builds one `SitemapDocument` tree from a crawl's
  `RawSitemapDocument`s, starting at the retailer's root sitemaps. It leaves
  out each child sitemap its `greylisted` function rejects, since the crawl
  doesn't download those.
  `build_tree_with` does the same but asks a function for each file by URL, for
  callers that load files on demand.
- `GroupedLinks`: a tree's URLs sorted by kind of page, the consolidated
  sitemap.
- `SitemapDocument`: the tree, serialized as the processed sitemap that
  `retailer_sourcing` stores and the development tools write to disk.

`retailer_sourcing` crawls, processes and consolidates sitemaps with it, and
`process-sitemap-sources` and `consolidate-sitemap-sources` in
`apps/development` process and consolidate dumped files with it.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.
