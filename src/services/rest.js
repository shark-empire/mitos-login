// Minimal Supabase client over fetch: PostgREST (tables) and GoTrue (auth).
// No SDK is needed for what this app does, which keeps the bundle small.
import { config } from '../config/env';

export class ApiError extends Error {
  constructor(message, status, code = '') {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code; // machine-readable reason when the server gives one (for example "email_not_confirmed")
  }
}

/** Pull the human message and machine code out of a PostgREST / GoTrue error body. */
async function readError(res) {
  const text = await res.text().catch(() => '');
  try {
    const j = JSON.parse(text);
    const message = j.msg || j.message || j.error_description || j.error || text || `Request failed (${res.status})`;
    const code = [j.error_code, j.code, j.error].find((c) => typeof c === 'string') || '';
    return { message: String(message), code };
  } catch {
    return { message: text || `Request failed (${res.status})`, code: '' };
  }
}

export function createClient({ url, anon }) {
  // The classic "anon" key is a JWT and also goes in Authorization when nobody is signed in. The newer
  // "publishable" key (sb_publishable_...) is not a JWT: Supabase wants it in `apikey` only.
  const anonIsJwt = String(anon).startsWith('eyJ');
  const headers = (token, extra) => {
    const bearer = token || (anonIsJwt ? anon : '');
    return {
      apikey: anon,
      ...(bearer ? { Authorization: `Bearer ${bearer}` } : {}),
      'Content-Type': 'application/json',
      ...extra,
    };
  };

  async function request(method, path, { token, body, prefer, signal, extraHeaders, raw } = {}) {
    let res;
    try {
      res = await fetch(`${url}${path}`, {
        method,
        headers: headers(token, { ...(prefer ? { Prefer: prefer } : {}), ...extraHeaders }),
        body: body === undefined ? undefined : raw ? body : JSON.stringify(body),
        signal,
      });
    } catch (err) {
      if (err?.name === 'AbortError') throw err;
      // fetch only rejects when no response came back at all (offline, blocked, DNS, CORS).
      throw new ApiError('Network error', 0, 'network');
    }
    if (!res.ok) {
      const { message, code } = await readError(res);
      throw new ApiError(message, res.status, code);
    }
    if (res.status === 204 || prefer === 'return=minimal') return null;
    return res.json();
  }

  return {
    select: (table, query = '', token, signal) => request('GET', `/rest/v1/${table}${query}`, { token, signal }),
    insert: (table, body, token) => request('POST', `/rest/v1/${table}`, { token, body, prefer: 'return=minimal' }),
    update: (table, query, body, token) =>
      request('PATCH', `/rest/v1/${table}${query}`, { token, body, prefer: 'return=minimal' }),
    /** Call a Postgres function (`select * from fn(args)`). */
    rpc: (fn, args, token) => request('POST', `/rest/v1/rpc/${fn}`, { token, body: args || {} }),
    /** Upload a file to a storage bucket and return its public URL. */
    upload: async (bucket, path, file, token) => {
      await request('POST', `/storage/v1/object/${bucket}/${path}`, {
        token,
        body: file,
        raw: true,
        extraHeaders: { 'Content-Type': file.type || 'application/octet-stream', 'x-upsert': 'true' },
      });
      return `${url}/storage/v1/object/public/${bucket}/${path}`;
    },

    auth: {
      signIn: (email, password) =>
        request('POST', '/auth/v1/token?grant_type=password', { body: { email, password } }),
      // `data` becomes the user's metadata (for example a full name).
      signUp: (email, password, data) =>
        request('POST', '/auth/v1/signup', { body: { email, password, ...(data ? { data } : {}) } }),
      /** Send the sign-up confirmation email again. */
      resend: (email) => request('POST', '/auth/v1/resend', { body: { type: 'signup', email } }),
      refresh: (refreshToken) =>
        request('POST', '/auth/v1/token?grant_type=refresh_token', { body: { refresh_token: refreshToken } }),
      user: (token) => request('GET', '/auth/v1/user', { token }),
    },
  };
}

/** Bundle catalogue + payment confirmations. */
export const dataApi = createClient(config.dataBackend);
/** Sign-in, profiles, wallet and account orders. */
export const accountsApi = createClient(config.accountsBackend);
/** Reseller stores, store bundles and store orders (the Store project). */
export const storesApi = createClient(config.stores);
