import { useEffect, useRef, useState } from 'react';
import { LuCheck, LuCircleAlert, LuUpload } from 'react-icons/lu';
import { Button, Callout, CopyButton, Spinner, TextField } from '../../components/ui';
import { StoreAvatar, withStores } from '../../components/stores/StoreBits';
import { ConfirmEmail, SellerAccountForm } from './SellerLogin';
import { useSeller } from '../../context/Seller';
import { useToast } from '../../context/Toast';
import { usePageTitle } from '../../hooks/usePageTitle';
import { useTintColor } from '../../hooks/useTintColor';
import { Link } from '../../router';
import { ACCENTS, DEFAULT_ACCENT } from '../../lib/color';
import { digitsOnly } from '../../lib/phone';
import { shopPath, storeUrl } from '../../lib/links';
import { toSlug } from '../../lib/slug';
import { createStore, fetchStoreByOwner, isSlugTaken, uploadStoreLogo } from '../../services/stores';

const STEPS = ['Account', 'Your store', 'Look', 'Launch'];
const MAX_LOGO = 2 * 1024 * 1024;

function StoreBuilder() {
  usePageTitle('Open your store');
  const { seller, ready, authed } = useSeller();
  const toast = useToast();
  const [step, setStep] = useState(0);
  const [existing, setExisting] = useState(undefined); // undefined: checking, null: none
  const [busy, setBusy] = useState(false);
  const [info, setInfo] = useState({ name: '', description: '', whatsapp: '' });
  const [logo, setLogo] = useState({ file: null, url: null });
  const [accent, setAccent] = useState(DEFAULT_ACCENT);
  const [created, setCreated] = useState(null);
  const [touched, setTouched] = useState(false);
  const [pending, setPending] = useState(null);
  const fileRef = useRef(null);
  useTintColor(step >= 2 ? accent : null);

  // A signed-in seller can only have one store.
  useEffect(() => {
    if (!ready) return undefined;
    if (!seller) {
      setExisting(null);
      return undefined;
    }
    let off = false;
    authed((t) => fetchStoreByOwner(seller.id, t))
      .then((s) => !off && setExisting(s || null))
      .catch(() => !off && setExisting(null));
    return () => {
      off = true;
    };
  }, [ready, seller, authed]);

  // Skip the account step once there is an account.
  useEffect(() => {
    if (seller && step === 0 && existing === null) setStep(1);
  }, [seller, step, existing]);

  useEffect(() => () => logo.url && URL.revokeObjectURL(logo.url), [logo.url]);

  const slug = toSlug(info.name);
  const nameError = touched && (info.name.trim().length < 3 || slug.length < 3) ? 'Use a name of at least 3 letters or numbers' : undefined;

  const chooseLogo = (e) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (!/^image\/(png|jpe?g|webp|gif)$/.test(file.type)) return toast.err('Choose a PNG, JPG, WebP or GIF image.');
    if (file.size > MAX_LOGO) return toast.err('That image is over 2 MB. Choose a smaller one.');
    setLogo({ file, url: URL.createObjectURL(file) });
  };

  const toLook = () => {
    setTouched(true);
    if (info.name.trim().length < 3 || slug.length < 3) return;
    setStep(2);
  };

  const launch = async () => {
    setBusy(true);
    try {
      let finalSlug = slug;
      if (await isSlugTaken(finalSlug)) finalSlug = `${finalSlug.slice(0, 36)}-${Math.random().toString(36).slice(2, 5)}`;
      let logoUrl = null;
      let logoFailed = false;
      if (logo.file) {
        try {
          logoUrl = await authed((t) => uploadStoreLogo(logo.file, seller.id, t));
        } catch {
          logoFailed = true;
        }
      }
      await authed((t) =>
        createStore(
          { ownerId: seller.id, name: info.name.trim(), slug: finalSlug, description: info.description.trim(), whatsapp: digitsOnly(info.whatsapp), accent, logoUrl },
          t,
        ),
      );
      setCreated({ slug: finalSlug, name: info.name.trim(), logo: logoUrl, accent });
      setStep(3);
      toast.ok('Your store is live.');
      if (logoFailed) toast.info('Your logo could not be uploaded, so the store opened without one.');
    } catch (err) {
      toast.err(err.message || 'We could not create your store.');
    } finally {
      setBusy(false);
    }
  };

  if (!ready || (seller && existing === undefined)) {
    return <div className="pageloading" role="status" aria-label="Loading"><Spinner size={28} /></div>;
  }

  if (seller && existing && step !== 3) {
    return (
      <div className="container authpage">
        <div className="panel authcard stack">
          <h1 className="authcard__title">You already have a store</h1>
          <p className="authcard__lead">{existing.name} is live. Each seller account has one store.</p>
          <Button to="/seller" block>Go to my dashboard</Button>
          <Button to={shopPath(existing.slug)} variant="secondary" block>View my store</Button>
        </div>
      </div>
    );
  }

  const preview = { name: info.name || 'Your store', accent, logo: logo.url };

  return (
    <div className="container page page--narrow">
      <header className="pagehead">
        <div className="pagehead__text">
          <h1 className="pagehead__title">Open your store</h1>
          <p className="pagehead__lead">Sell data under your own name. Set your own prices, share one link, get paid through Paystack.</p>
        </div>
      </header>

      <ol className="wizard" aria-label="Progress">
        {STEPS.map((label, i) => (
          <li key={label} className={`wizard__step${i === step ? ' is-on' : ''}${i < step ? ' is-done' : ''}`} aria-current={i === step ? 'step' : undefined}>
            <span className="wizard__n">{i < step ? <LuCheck aria-hidden="true" /> : i + 1}</span>
            <span className="wizard__label">{label}</span>
          </li>
        ))}
      </ol>

      <section className="panel wizard__body">
        {step === 0 && pending && <ConfirmEmail email={pending} />}
        {step === 0 && !pending && (
          <div className="stack">
            <h2 className="wizard__title">Create your seller account</h2>
            <p className="authcard__lead">You will use it to manage bundles and see orders.</p>
            <SellerAccountForm mode="signup" submitLabel="Create account and continue" onConfirm={setPending} />
            <p className="authcard__foot">Already a seller? <Link to="/seller/login?next=/store-builder">Log in</Link></p>
          </div>
        )}

        {step === 1 && (
          <div className="stack">
            <h2 className="wizard__title">About your store</h2>
            <TextField label="Store name" placeholder="Kwame’s Data Hub" maxLength={40} value={info.name} onChange={(e) => setInfo({ ...info, name: e.target.value })} error={nameError} hint={slug ? `Your link ends with /s/${slug}` : undefined} />
            <TextField label="Description (optional)" as="textarea" rows={3} maxLength={160} placeholder="What makes your store special?" value={info.description} onChange={(e) => setInfo({ ...info, description: e.target.value })} />
            <TextField label="WhatsApp number (optional)" type="tel" inputMode="numeric" prefix="+" placeholder="233541234567" value={info.whatsapp} onChange={(e) => setInfo({ ...info, whatsapp: digitsOnly(e.target.value).slice(0, 15) })} hint="With the country code. Customers use it to message you about an order." />
            <div className="field">
              <span className="field__label">Logo (optional)</span>
              <input ref={fileRef} type="file" accept="image/png,image/jpeg,image/webp,image/gif" className="sr-only" onChange={chooseLogo} aria-label="Choose a logo image" />
              <button type="button" className="logodrop" onClick={() => fileRef.current?.click()}>
                {logo.url ? <img src={logo.url} alt="Your chosen logo" /> : <LuUpload aria-hidden="true" />}
                <span>{logo.file ? logo.file.name : 'Choose an image'}</span>
              </button>
              <p className="field__hint">PNG, JPG, WebP or GIF, up to 2 MB.</p>
            </div>
            <div className="wizard__nav">
              <Button onClick={toLook}>Next</Button>
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="stack">
            <h2 className="wizard__title">Choose your colour</h2>
            <p className="authcard__lead">The whole page takes on this colour when customers visit your store.</p>
            <div className="swatches" role="radiogroup" aria-label="Accent colour">
              {ACCENTS.map((c) => (
                <button key={c.value} type="button" role="radio" aria-checked={accent === c.value} className={`swatch${accent === c.value ? ' is-on' : ''}`} onClick={() => setAccent(c.value)}>
                  <span className="swatch__dot" style={{ background: c.value }} aria-hidden="true" />
                  {c.name}
                </button>
              ))}
            </div>
            <div className="panel preview">
              <StoreAvatar store={{ ...preview, logo: preview.logo }} size="lg" />
              <div>
                <strong>{preview.name}</strong>
                <p>{info.description || 'Data bundles at fair prices.'}</p>
              </div>
            </div>
            <div className="wizard__nav">
              <Button variant="ghost" onClick={() => setStep(1)}>Back</Button>
              <Button onClick={launch} loading={busy}>Launch my store</Button>
            </div>
          </div>
        )}

        {step === 3 && created && (
          <div className="stack launched">
            <StoreAvatar store={created} size="xl" />
            <h2 className="wizard__title">{created.name} is live</h2>
            <p className="authcard__lead">Add your first bundles, then share your link with customers. They see only your store and their order tracker, nothing else.</p>
            <div className="receipt__ref"><code>{storeUrl(created.slug)}</code><CopyButton text={storeUrl(created.slug)} label="Copy link" /></div>
            <Callout tone="info" icon={LuCircleAlert}>Your store shows no bundles until you add some. That takes a minute.</Callout>
            <div className="status__actions status__actions--left">
              <Button to="/seller/bundles">Add bundles</Button>
              <Button to={shopPath(created.slug)} variant="secondary">View my store</Button>
            </div>
          </div>
        )}
      </section>
    </div>
  );
}

export default withStores(StoreBuilder);
