import { useEffect, useRef, useState } from 'react';
import { LuCircleAlert, LuCircleCheck, LuShieldAlert } from 'react-icons/lu';
import { Button, Callout, PageHeader, TextField } from '../../components/ui';
import { usePageTitle } from '../../hooks/usePageTitle';
import { sendDataTransfer, dataTransferEnabled } from '../../services/webhooks';
import { transferRef } from '../../lib/ids';
import { cleanPhone } from '../../lib/phone';

const COOLDOWN_MS = 4000;

/**
 * Internal tool: send data units to a number through the transfer webhook.
 * It is not linked from the public navigation. The webhook itself must check `admin_key`,
 * because anything compiled into a browser bundle can be read by visitors.
 */
export default function Transfer() {
  usePageTitle('Data transfer');
  const [adminKey, setAdminKey] = useState('');
  const [phone, setPhone] = useState('');
  const [gb, setGb] = useState('');
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState(null);
  const last = useRef(0);

  useEffect(() => {
    let meta = document.querySelector('meta[name="robots"]');
    if (!meta) {
      meta = document.createElement('meta');
      meta.name = 'robots';
      document.head.appendChild(meta);
    }
    meta.content = 'noindex';
    return () => meta.remove();
  }, []);

  const submit = async (e) => {
    e.preventDefault();
    const wait = COOLDOWN_MS - (Date.now() - last.current);
    if (wait > 0) return setResult({ ok: false, text: `Wait ${Math.ceil(wait / 1000)}s before sending another request.` });
    if (!adminKey.trim()) return setResult({ ok: false, text: 'Enter the admin key.' });
    if (!/^\d{10}$/.test(phone)) return setResult({ ok: false, text: 'The receiver number must be 10 digits.' });
    const amount = parseFloat(gb);
    if (!Number.isFinite(amount) || amount <= 0) return setResult({ ok: false, text: 'The amount must be greater than 0.' });

    last.current = Date.now();
    setBusy(true);
    setResult(null);
    try {
      const data = await sendDataTransfer({ phone, amount, reference: transferRef(), admin_key: adminKey.trim() });
      setResult({ ok: true, text: 'Transfer sent.', data });
    } catch (err) {
      setResult({ ok: false, text: err.message });
    } finally {
      setBusy(false);
    }
  };

  if (!dataTransferEnabled) {
    return (
      <div className="container page page--narrow">
        <PageHeader title="Data transfer" />
        <Callout tone="warn" icon={LuShieldAlert}>
          <p><strong>This tool is switched off.</strong> Set <code>VITE_DATA_TRANSFER_URL</code> to your transfer webhook and rebuild to enable it.</p>
          <p>Make sure the webhook rejects requests that do not carry the correct <code>admin_key</code>. Anything in the browser bundle is public.</p>
        </Callout>
      </div>
    );
  }

  return (
    <div className="container page page--narrow">
      <PageHeader title="Data transfer" lead="Send data units to any receiver. For staff use." />
      <form className="panel settingsform" onSubmit={submit}>
        <TextField label="Admin key" type="password" autoComplete="off" value={adminKey} onChange={(e) => setAdminKey(e.target.value)} hint="Sent with the request so the webhook can verify you." />
        <TextField label="Receiver number" type="tel" inputMode="numeric" maxLength={16} prefix="+233" placeholder="0541234567" value={phone} onChange={(e) => setPhone(cleanPhone(e.target.value))} />
        <TextField label="Amount (GB)" type="number" min="0" step="0.1" placeholder="1" value={gb} onChange={(e) => setGb(e.target.value)} />
        <div><Button type="submit" loading={busy}>Send data</Button></div>
        {result && (
          <Callout tone={result.ok ? 'ok' : 'bad'} icon={result.ok ? LuCircleCheck : LuCircleAlert}>
            <p>{result.text}</p>
            {result.data && <pre className="codeblock" tabIndex={0}><code>{JSON.stringify(result.data, null, 2)}</code></pre>}
          </Callout>
        )}
      </form>
    </div>
  );
}
