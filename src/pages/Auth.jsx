import { useEffect, useState } from 'react';
import { LuCircleAlert, LuEye, LuEyeOff, LuMailCheck, LuMessageCircle } from 'react-icons/lu';
import { Button, Callout, Segmented, TextField } from '../components/ui';
import { useAuth } from '../context/Auth';
import { useToast } from '../context/Toast';
import { usePageTitle } from '../hooks/usePageTitle';
import { Link, useLocation, useNavigate } from '../router';
import { site } from '../config/site';
import { checkRate } from '../lib/rateLimit';
import { validatePhone, cleanPhone } from '../lib/phone';

function PasswordField({ label, value, onChange, ...rest }) {
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

/** Only follow in-app paths after sign-in. */
const safeNext = (next) => (next && next.startsWith('/') && !next.startsWith('//') ? next : '/account');

export function Login() {
  usePageTitle('Log in');
  const { signIn, resendConfirmation, user } = useAuth();
  const toast = useToast();
  const navigate = useNavigate();
  const { search } = useLocation();
  const next = safeNext(search.get('next'));
  const [id, setId] = useState('');
  const [pw, setPw] = useState('');
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState(null); // { message, unconfirmedEmail? }

  useEffect(() => {
    if (user) navigate(next, { replace: true });
  }, [user, next, navigate]);

  const submit = async (e) => {
    e.preventDefault();
    setProblem(null);
    if (!id.trim() || !pw) return setProblem({ message: 'Enter your email or phone number and your password.' });
    const rate = checkRate('login', 5, 60000);
    if (!rate.ok) return setProblem({ message: `Too many attempts. Wait ${rate.waitSeconds} seconds and try again.` });
    setBusy(true);
    try {
      const u = await signIn(id, pw);
      toast.ok(`Welcome back${u.first_name ? `, ${u.first_name}` : ''}.`);
      navigate(next, { replace: true });
    } catch (err) {
      setProblem({ message: err.message, unconfirmedEmail: err.unconfirmed && err.email.includes('@') && !err.email.endsWith('@sci-fidata.com') ? err.email : '' });
    } finally {
      setBusy(false);
    }
  };

  const resend = async () => {
    try {
      await resendConfirmation(problem.unconfirmedEmail);
      toast.ok('Confirmation email sent. Check your inbox and spam folder.');
    } catch (err) {
      toast.err(err.message);
    }
  };

  return (
    <div className="container authpage">
      <form className="panel authcard" onSubmit={submit}>
        <h1 className="authcard__title">Log in</h1>
        <p className="authcard__lead">Use your email or the phone number you signed up with.</p>
        <TextField label="Email or phone number" autoComplete="username" placeholder="you@example.com or 0541234567" value={id} onChange={(e) => setId(e.target.value)} />
        <PasswordField label="Password" autoComplete="current-password" value={pw} onChange={(e) => setPw(e.target.value)} />
        {problem && (
          <Callout tone="bad" icon={LuCircleAlert}>
            {problem.message}
            {problem.unconfirmedEmail && (
              <> <button type="button" className="linkbtn" onClick={resend}>Send the email again</button></>
            )}
          </Callout>
        )}
        <Button type="submit" block size="lg" loading={busy}>Log in</Button>
        <p className="authcard__foot">New here? <Link to="/signup">Create an account</Link></p>
        <p className="authcard__foot">You don’t need an account to <Link to="/data">buy data</Link>.</p>
      </form>
    </div>
  );
}

function strength(p) {
  if (!p) return null;
  let s = 0;
  if (p.length >= 8) s++;
  if (p.length >= 12) s++;
  if (/[A-Z]/.test(p)) s++;
  if (/[0-9]/.test(p)) s++;
  if (/[^A-Za-z0-9]/.test(p)) s++;
  return { pct: (s / 5) * 100, label: ['Too short', 'Weak', 'Fair', 'Good', 'Strong', 'Very strong'][s], tone: s <= 1 ? 'bad' : s <= 3 ? 'warn' : 'ok' };
}

/** Shown after an email sign-up when the project wants the address confirmed before the first login. */
function CheckEmail({ email, resend }) {
  const toast = useToast();
  const [busy, setBusy] = useState(false);
  const again = async () => {
    setBusy(true);
    try {
      await resend(email);
      toast.ok('Confirmation email sent again.');
    } catch (err) {
      toast.err(err.message);
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="container authpage">
      <div className="panel authcard">
        <h1 className="authcard__title">Check your email</h1>
        <p className="authcard__lead">We sent a confirmation link to <strong>{email}</strong>. Open it, then come back and log in. If you can’t find it, look in your spam folder.</p>
        <Button to="/login" block><LuMailCheck aria-hidden="true" /> Go to log in</Button>
        <Button variant="secondary" block onClick={again} loading={busy}>Send the email again</Button>
      </div>
    </div>
  );
}

/** Phone accounts have no real inbox, so a project that insists on email confirmation cannot activate them by itself. */
function ActivationNeeded({ phone }) {
  return (
    <div className="container authpage">
      <div className="panel authcard">
        <h1 className="authcard__title">One more step</h1>
        <p className="authcard__lead">Your account for <strong>{phone}</strong> was created, but it has to be activated before you can log in. Message us and we will switch it on.</p>
        <Button href={site.whatsappUrl(`Hi, please activate my account for ${phone}.`)} target="_blank" rel="noreferrer" block><LuMessageCircle aria-hidden="true" /> Message support on WhatsApp</Button>
        <Button to="/login" variant="secondary" block>Go to log in</Button>
      </div>
    </div>
  );
}

export function Signup() {
  usePageTitle('Create account');
  const { signUp, resendConfirmation, user } = useAuth();
  const toast = useToast();
  const navigate = useNavigate();
  const [method, setMethod] = useState('phone');
  const [f, setF] = useState({ first: '', last: '', phone: '', email: '', pw: '', pw2: '' });
  const [busy, setBusy] = useState(false);
  const [confirm, setConfirm] = useState(null); // { email, method } once the project asks for confirmation first
  const [problem, setProblem] = useState(null);
  const [touched, setTouched] = useState(false);
  const set = (k) => (e) => setF((p) => ({ ...p, [k]: k === 'phone' ? cleanPhone(e.target.value) : e.target.value }));

  useEffect(() => {
    if (user) navigate('/account', { replace: true });
  }, [user, navigate]);

  const phoneCheck = validatePhone(f.phone, { strict: true });
  const pwStrength = strength(f.pw);
  const errors = {
    first: !f.first.trim() && 'Enter your first name',
    last: !f.last.trim() && 'Enter your last name',
    contact: method === 'phone' ? !phoneCheck.ok && phoneCheck.msg : !/^\S+@\S+\.\S+$/.test(f.email.trim()) && 'Enter a valid email address',
    pw: (f.pw.length < 8 && 'Use at least 8 characters') || ((!/[A-Z]/.test(f.pw) || !/[0-9]/.test(f.pw)) && 'Include a capital letter and a number'),
    pw2: f.pw !== f.pw2 && 'The passwords don’t match',
  };
  const show = (k) => (touched && errors[k]) || undefined;

  const submit = async (e) => {
    e.preventDefault();
    setTouched(true);
    setProblem(null);
    if (Object.values(errors).some(Boolean)) return;
    const rate = checkRate('signup', 3, 300000);
    if (!rate.ok) return setProblem(`Too many attempts. Wait ${rate.waitSeconds} seconds and try again.`);
    setBusy(true);
    try {
      const res = await signUp({ firstName: f.first.trim(), lastName: f.last.trim(), phone: f.phone, email: f.email, password: f.pw, method });
      if (res.needsConfirmation) setConfirm({ email: res.email, method: res.method });
      else {
        toast.ok('Your account is ready.');
        navigate('/account', { replace: true });
      }
    } catch (err) {
      setProblem(err.message || 'We couldn’t create your account.');
    } finally {
      setBusy(false);
    }
  };

  if (confirm) {
    return confirm.method === 'phone' ? (
      <ActivationNeeded phone={f.phone} />
    ) : (
      <CheckEmail email={confirm.email} resend={resendConfirmation} />
    );
  }

  return (
    <div className="container authpage">
      <form className="panel authcard authcard--wide" onSubmit={submit} noValidate>
        <h1 className="authcard__title">Create your account</h1>
        <p className="authcard__lead">Keep a wallet balance and see all your orders in one place.</p>
        <Segmented label="Sign up with" value={method} onChange={setMethod} options={[{ value: 'phone', label: 'Phone number' }, { value: 'email', label: 'Email' }]} />
        <div className="fields2">
          <TextField label="First name" autoComplete="given-name" value={f.first} onChange={set('first')} error={show('first')} />
          <TextField label="Last name" autoComplete="family-name" value={f.last} onChange={set('last')} error={show('last')} />
        </div>
        {method === 'phone' ? (
          <TextField label="Ghana phone number" type="tel" inputMode="numeric" maxLength={16} prefix="+233" autoComplete="tel-national" placeholder="0541234567" value={f.phone} onChange={set('phone')} error={show('contact')} hint="You will use this number to log in." />
        ) : (
          <TextField label="Email address" type="email" autoComplete="email" placeholder="you@example.com" value={f.email} onChange={set('email')} error={show('contact')} />
        )}
        <PasswordField label="Password" autoComplete="new-password" value={f.pw} onChange={set('pw')} error={show('pw')} hint={show('pw') ? undefined : 'At least 8 characters, with a capital letter and a number.'} />
        {pwStrength && (
          <div className="strength" aria-live="polite">
            <div className="strength__bar"><span className={`strength__fill strength__fill--${pwStrength.tone}`} style={{ width: `${pwStrength.pct}%` }} /></div>
            <span className="strength__label">{pwStrength.label}</span>
          </div>
        )}
        <PasswordField label="Confirm password" autoComplete="new-password" value={f.pw2} onChange={set('pw2')} error={show('pw2')} />
        <p className="authcard__fine">By creating an account you agree to our <Link to="/terms">terms</Link> and <Link to="/privacy">privacy policy</Link>.</p>
        {problem && (
          <Callout tone="bad" icon={LuCircleAlert}>
            {problem}{' '}
            {/already has an account/.test(problem) && <Link to="/login">Go to log in</Link>}
          </Callout>
        )}
        <Button type="submit" block size="lg" loading={busy}>Create account</Button>
        <p className="authcard__foot">Already registered? <Link to="/login">Log in</Link></p>
      </form>
    </div>
  );
}
