import { useState } from 'react';
import { Button, PageHeader, TextField } from '../../components/ui';
import { useAuth } from '../../context/Auth';
import { useToast } from '../../context/Toast';
import { usePageTitle } from '../../hooks/usePageTitle';
import { validatePhone, cleanPhone } from '../../lib/phone';

export default function Settings() {
  usePageTitle('Settings');
  const { user, updateProfile } = useAuth();
  const toast = useToast();
  const [f, setF] = useState({ first: user.first_name, last: user.last_name, phone: user.phone });
  const [busy, setBusy] = useState(false);

  const phoneCheck = f.phone ? validatePhone(f.phone) : { ok: true };

  const save = async (e) => {
    e.preventDefault();
    if (!phoneCheck.ok) return toast.err(phoneCheck.msg);
    setBusy(true);
    try {
      await updateProfile({ first_name: f.first.trim(), last_name: f.last.trim(), phone: f.phone });
      toast.ok('Your details have been saved.');
    } catch {
      toast.err('We couldn’t save your details. Try again.');
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <PageHeader title="Settings" lead="Your name and phone number. The phone number is used as the default recipient when you buy data." />
      <form className="panel settingsform" onSubmit={save}>
        <div className="fields2">
          <TextField label="First name" autoComplete="given-name" value={f.first} onChange={(e) => setF({ ...f, first: e.target.value })} />
          <TextField label="Last name" autoComplete="family-name" value={f.last} onChange={(e) => setF({ ...f, last: e.target.value })} />
        </div>
        <TextField label="Phone number" type="tel" inputMode="numeric" maxLength={16} prefix="+233" value={f.phone} onChange={(e) => setF({ ...f, phone: cleanPhone(e.target.value) })} error={f.phone && !phoneCheck.ok ? phoneCheck.msg : undefined} />
        <TextField label="Sign-in email" value={user.email || ''} disabled hint="This can’t be changed." />
        <div><Button type="submit" loading={busy}>Save changes</Button></div>
      </form>
    </>
  );
}
