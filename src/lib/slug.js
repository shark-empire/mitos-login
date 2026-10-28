/** "Kwame’s Data Hub!" -> "kwame-s-data-hub". Letters and digits only, at most 40 characters. */
export const toSlug = (name) =>
  String(name ?? '')
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 40)
    .replace(/-+$/g, '');
