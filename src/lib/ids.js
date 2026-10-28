const rand = (len) => Math.random().toString(36).slice(2, 2 + len).toUpperCase().padEnd(len, '0');

export const uuid = () =>
  typeof crypto !== 'undefined' && crypto.randomUUID
    ? crypto.randomUUID()
    : 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
        const r = (Math.random() * 16) | 0;
        return (c === 'x' ? r : (r & 0x3) | 0x8).toString(16);
      });

// Reference prefixes are relied on by the order lookup and by support, so keep them stable.
export const dataRef = () => `SF-${Date.now()}-${Math.floor(100000 + Math.random() * 900000)}`;
export const walletRef = () => `SCIFI-${Date.now()}-${rand(6)}`;
export const resultsRef = () => `RC-${Date.now()}-${rand(5)}`;
export const transferRef = () => `SFapi_${Date.now()}_${Math.random().toString(36).slice(2, 11)}`;
// Store orders keep the DP- prefix the Store project used, in case its payment automation matches on it.
export const storeRef = () => `DP-${Date.now()}-${rand(4)}`;
