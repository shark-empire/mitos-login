import { createContext, useCallback, useContext, useMemo, useRef, useState } from 'react';
import { LuCircleAlert, LuCircleCheck, LuInfo, LuX } from 'react-icons/lu';

const ToastContext = createContext(null);
const ICONS = { ok: LuCircleCheck, err: LuCircleAlert, info: LuInfo };

export function ToastProvider({ children }) {
  const [items, setItems] = useState([]);
  const nextId = useRef(1);

  const dismiss = useCallback((id) => setItems((list) => list.filter((t) => t.id !== id)), []);

  const push = useCallback(
    (tone, message) => {
      const id = nextId.current++;
      setItems((list) => [...list.slice(-3), { id, tone, message }]);
      setTimeout(() => dismiss(id), tone === 'err' ? 6000 : 4000);
    },
    [dismiss],
  );

  const api = useMemo(
    () => ({
      ok: (m) => push('ok', m),
      err: (m) => push('err', m),
      info: (m) => push('info', m),
    }),
    [push],
  );

  return (
    <ToastContext.Provider value={api}>
      {children}
      <div className="toasts" role="region" aria-label="Notifications" aria-live="polite">
        {items.map((t) => {
          const Icon = ICONS[t.tone];
          return (
            <div key={t.id} className={`toast toast--${t.tone}`} role={t.tone === 'err' ? 'alert' : 'status'}>
              <Icon aria-hidden="true" className="toast__icon" />
              <span className="toast__msg">{t.message}</span>
              <button type="button" className="toast__close" onClick={() => dismiss(t.id)} aria-label="Dismiss">
                <LuX aria-hidden="true" />
              </button>
            </div>
          );
        })}
      </div>
    </ToastContext.Provider>
  );
}

export const useToast = () => useContext(ToastContext);
