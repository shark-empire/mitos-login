import { useEffect, useState } from 'react';
import { LuEye, LuEyeOff } from 'react-icons/lu';
import { Button, Segmented, TextField } from '../../components/ui';
import { withStores } from '../../components/stores/StoreBits';
import { useSeller } from '../../context/Seller';
import { useToast } from '../../context/Toast';
import { usePageTitle } from '../../hooks/usePageTitle';
import { Link, useLocation, useNavigate } from '../../router';
import { checkRate } from '../../lib/rateLimit';
import { cleanPhone } from '../../lib/phone';

export function SellerPassword({ label, value, onChange, ...rest }) {
  const [show, setShow] = useState(false);
  return (
    <TextField
      label={label}
      type={show ? 'text' : 'password'}
      value={value}
      onChange={onChange}
      suffix={
        <button type="button" className="field__action field__action--icon" onClick={() => setShow((s) => !s)} aria-label={show ? 'Hide password' : 'Show password'}>
          {show ? <LuEyeOff aria-hidden="true" /> : <LuEye aria-hidden="true" />}
        </button>
      }
      {...rest}
    />
  );
}

/** Shown after sign-up when the project wants the email address confirmed before there is a session. */
export function ConfirmEmail({ email }) {
  return (
    <div className="stack">
      <h2 className="authcard__title">Check your email</h2>
      <p className="authcard__lead">We sent a confirmation link to {email}. Open it, then come back and log in.</p>
      <Button to="/seller/login" block>Go to log in</Button>
    </div>
  );
}

/** The account form for store owners, used here and as the first step of the store builder. */
export function SellerAccountForm({ mode, onDone, onConfirm, submitLabel }) {
  const { signIn, signUp } = useSeller();
  const toast = useToast();
  const [f, setF] = useState({ name: '', phone: '', email: '', pw: '' });
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);
  const set = (k) => (e) => setF((p) => ({ ...p, [k]: k === 'phone' ? cleanPhone(e.target.value) : e.target.value }));

  const signingUp = mode === 'signup';
  const errors = {
    name: signingUp && !f.name.trim() && 'Enter your name',
    email: !/^\S+@\S+\.\S+$/.test(f.email.trim()) && 'Enter a valid email address',
    pw: (signingUp ? f.pw.length < 8 : !f.pw) && (signingUp ? 'Use at least 8 characters' : 'Enter your password'),
  };
  const show = (k) => (touched && errors[k]) || undefined;

  const submit = async (e) => {
    e.preventDefault();
    setTouched(true);
    if (Object.values(errors).some(Boolean)) return;
    const rate = checkRate(signingUp ? 'seller-signup' : 'seller-login', signingUp ? 3 : 5, signingUp ? 300000 : 60000);
    if (!rate.ok) return toast.err(`Too many attempts. Wait ${rate.waitSeconds} seconds and try again.`);
    setBusy(true);
    try {
      if (signingUp) {
        const res = await signUp({ fullName: f.name.trim(), phone: f.phone, email: f.email, password: f.pw });
        if (res.needsConfirmation) onConfirm?.(f.email.trim());
        else {
          toast.ok('Your seller account is ready.');
          onDone?.(res.seller);
        }
      } else {
        const s = await signIn(f.email, f.pw);
        toast.ok(`Welcome back${s.name ? `, ${s.name.split(' ')[0]}` : ''}.`);
        onDone?.(s);
      }
    } catch (err) {
      toast.err(err.message || 'Something went wrong. Try again.');
    } finally {
      setBusy(false);
    }
  };

  return (
    <form className="stack" onSubmit={submit} noValidate>
      {signingUp && <TextField label="Your name" autoComplete="name" value={f.name} onChange={set('name')} error={show('name')} />}
      <TextField label="Email address" type="email" autoComplete="email" placeholder="you@example.com" value={f.email} onChange={set('email')} error={show('email')} />
      {signingUp && (
        <TextField label="Phone number (optional)" type="tel" inputMode="numeric" maxLength={16} prefix="+233" autoComplete="tel-national" placeholder="0541234567" value={f.phone} onChange={set('phone')} />
      )}
      <SellerPassword label="Password" autoComplete={signingUp ? 'new-password' : 'current-password'} value={f.pw} onChange={set('pw')} error={show('pw')} hint={signingUp && !show('pw') ? 'At least 8 characters.' : undefined} />
      <Button type="submit" block size="lg" loading={busy}>{submitLabel || (signingUp ? 'Create seller account' : 'Log in')}</Button>
    </form>
  );
}

const safeNext = (next) => (next && next.startsWith('/') && !next.startsWith('//') ? next : '/seller');

function SellerLogin() {
  usePageTitle('Seller log in');
  const { seller } = useSeller();
  const navigate = useNavigate();
  const { search } = useLocation();
  const next = safeNext(search.get('next'));
  const [mode, setMode] = useState(search.get('mode') === 'signup' ? 'signup' : 'login');
  const [pending, setPending] = useState(null);

  useEffect(() => {
    if (seller) navigate(next, { replace: true });
  }, [seller, next, navigate]);

  if (pending) {
    return (
      <div className="container authpage">
        <div className="panel authcard"><ConfirmEmail email={pending} /></div>
      </div>
    );
  }

  return (
    <div className="container authpage">
      <div className="panel authcard">
        <h1 className="authcard__title">{mode === 'login' ? 'Seller log in' : 'Open a seller account'}</h1>
        <p className="authcard__lead">
          For store owners. This is separate from a customer account, which you do not need to buy data.
        </p>
        <Segmented label="Account" value={mode} onChange={setMode} options={[{ value: 'login', label: 'Log in' }, { value: 'signup', label: 'Create account' }]} />
        <SellerAccountForm key={mode} mode={mode} onDone={() => navigate(next, { replace: true })} onConfirm={setPending} />
        <p className="authcard__foot">Buying data? <Link to="/data">You don’t need an account</Link>.</p>
      </div>
    </div>
  );
}

export default withStores(SellerLogin);
