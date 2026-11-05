/**
 * Canonical URL and file slugification utilities for Pico.
 * Guarantees consistent slug and path generation across the application.
 */

/**
 * Converts any arbitrary string (e.g. post title, tag, collection name)
 * into a safe, lowercase, hyphen-delimited URL slug.
 */
export function slugify(input: string): string {
  if (!input) return '';
  return input
    .trim()
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '') // strip diacritics
    .replace(/[^a-z0-9]+/g, '-')     // replace non-alphanumeric with hyphens
    .replace(/^-+|-+$/g, '')         // trim leading/trailing hyphens
    .replace(/--+/g, '-');           // collapse multiple hyphens
}

/**
 * Normalizes a relative filesystem path:
 * - Converts backslashes to forward slashes
 * - Strips leading `./` and `/`
 * - Collapses consecutive forward slashes
 */
export function normalizeRelativePath(path: string): string {
  if (!path) return '';
  return path
    .trim()
    .replace(/\\/g, '/')
    .replace(/^\.?\/+/, '')
    .replace(/\/+/g, '/');
}

/**
 * Normalizes a collection model path to canonical `models/<name>.yaml` form.
 */
export function normalizeModelPath(pathOrName: string): string {
  let clean = normalizeRelativePath(pathOrName);
  let name = clean.replace(/^models\//, '');
  if (!name.endsWith('.yaml') && !name.endsWith('.yml')) {
    name = `${name}.yaml`;
  }
  return `models/${name}`;
}

/**
 * Derives a canonical content document path from collection and slug.
 */
export function formatPostPath(collection: string, slug: string): string {
  const cleanCol = slugify(collection) || 'blog';
  const cleanSlug = slugify(slug) || 'post';
  return `content/${cleanCol}/${cleanSlug}.md`;
}
