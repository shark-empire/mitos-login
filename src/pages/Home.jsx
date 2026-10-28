import { useEffect, useMemo, useState } from 'react';
import { LuCode, LuCreditCard, LuGraduationCap, LuLandmark, LuLayoutDashboard, LuLink, LuPackage, LuSmartphone, LuStore, LuWallet } from 'react-icons/lu';
import { Button, NetworkChip } from '../components/ui';
import RateCard from '../components/RateCard';
import { Link } from '../router';
import { usePageTitle } from '../hooks/usePageTitle';
import { useTint } from '../hooks/useTint';
import { useBundles } from '../hooks/useBundles';
import { site, FAQ, NETWORKS, RESULTS_BULK_PRICE, RESULTS_BULK_THRESHOLD, RESULTS_PRODUCTS, RESULTS_REGULAR_PRICE } from '../config/site';
import { money } from '../lib/format';
import { config } from '../config/env';

const STEPS = [
  { verb: 'choose.', text: 'Pick MTN, Telecel or AirtelTigo, then the size that suits you.' },
  { verb: 'check.', text: 'We show the registered owner’s name, so you know it is the right line.' },
  { verb: 'pay.', text: 'Mobile Money, card or bank transfer through Paystack. We never see your PIN or card details.' },
  { verb: 'receive.', text: 'Most bundles arrive within 1 to 2 minutes. Track the order with its reference.' },
];

const PAYMENTS = [
  { icon: LuSmartphone, label: 'Mobile Money' },
  { icon: LuCreditCard, label: 'Card' },
  { icon: LuLandmark, label: 'Bank transfer' },
];
const SHORTCUTS = [
  { icon: LuGraduationCap, label: 'Results checkers', to: '/results' },
  ...(config.stores.enabled ? [{ icon: LuStore, label: 'Marketplace', to: '/marketplace' }] : []),
  { icon: LuPackage, label: 'Track an order', to: '/track' },
  { icon: LuWallet, label: 'Wallet', to: '/account/wallet' },
];

/** A price in the stat band: small currency, big amount. */
const Price = ({ value }) => (
  <>
    <small>GH₵</small>
    {value.toFixed(2)}
  </>
);

const reducedMotion = () => typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

export default function Home() {
  usePageTitle(null);
  const { bundles } = useBundles();

  // The headline names a network. It steps through all three once, then stays where the visitor leaves it.
  // Whatever network is showing also sets the colour of the glass.
  const [network, setNetwork] = useState('MTN');
  const [auto, setAuto] = useState(() => !reducedMotion());
  const current = NETWORKS.find((n) => n.id === network) || NETWORKS[0];
  useTint(current.meta);

  useEffect(() => {
    if (!auto) return undefined;
    let ticks = 0;
    const timer = setInterval(() => {
      ticks += 1;
      setNetwork((cur) => NETWORKS[(NETWORKS.findIndex((n) => n.id === cur) + 1) % NETWORKS.length].id);
      if (ticks >= NETWORKS.length) setAuto(false);
    }, 3200);
    return () => clearInterval(timer);
  }, [auto]);

  const choose = (id) => {
    setAuto(false);
    setNetwork(id);
  };
  const next = () => choose(NETWORKS[(NETWORKS.findIndex((n) => n.id === network) + 1) % NETWORKS.length].id);

  const cheapest = useMemo(() => {
    const prices = bundles.filter((b) => b.available).map((b) => b.price);
    return prices.length ? Math.min(...prices) : null;
  }, [bundles]);

  return (
    <>
      <section className="hero">
        <div className="container hero__stack">
          <h1 className="hero__title" style={{ '--i': 0 }}>
            Data for{' '}
            <span className="rot" onClick={next} title="Show the next network">
              {NETWORKS.map((n) => (
                <span key={n.id} className={`rot__word${n.id === network ? ' is-on' : ''}`} aria-hidden={n.id === network ? undefined : 'true'}>
                  {n.label}
                </span>
              ))}
            </span>
            .
          </h1>
          <p className="hero__lead" style={{ '--i': 1 }}>
            Pay with Mobile Money or card and the bundle lands on the number you choose. WASSCE, NOVDEC and BECE results checkers appear on
            screen as soon as you have paid.
          </p>
          <div className="hero__cta" style={{ '--i': 2 }}>
            <Button to={`/data?network=${encodeURIComponent(network)}`} size="lg" data-magnetic>Buy data</Button>
            <Button to="/results" size="lg" variant="secondary" data-magnetic>Get a results checker</Button>
          </div>
          <ul className="chips" style={{ '--i': 3 }} aria-label="Payment methods and shortcuts">
            {PAYMENTS.map(({ icon: Icon, label }) => (
              <li key={label} className="chip">
                <Icon aria-hidden="true" /> {label}
              </li>
            ))}
            {SHORTCUTS.map(({ icon: Icon, label, to }) => (
              <li key={label}>
                <Link to={to} className="chip">
                  <Icon aria-hidden="true" /> {label}
                </Link>
              </li>
            ))}
          </ul>
          <div className="hero__lens" style={{ '--i': 4 }}>
            <RateCard network={network} onNetwork={choose} />
          </div>
        </div>
      </section>

      <section className="section section--tight">
        <div className="container">
          <ul className="statband">
            <li className="statband__item">
              <span className="statband__num">{NETWORKS.length}</span>
              <span className="statband__label">networks: MTN, Telecel and AirtelTigo</span>
            </li>
            <li className="statband__item">
              <span className="statband__num">{cheapest == null ? '–' : <Price value={cheapest} />}</span>
              <span className="statband__label">for the smallest bundle on sale today</span>
            </li>
            <li className="statband__item">
              <span className="statband__num">1–2<small>min</small></span>
              <span className="statband__label">is how long most bundles take to arrive</span>
            </li>
            <li className="statband__item">
              <span className="statband__num"><Price value={RESULTS_BULK_PRICE} /></span>
              <span className="statband__label">per results checker when you buy more than {RESULTS_BULK_THRESHOLD}</span>
            </li>
          </ul>
        </div>
      </section>

      <section className="section">
        <div className="container">
          <h2 className="section__title">From choice to data in four steps.</h2>
          <ol className="how">
            {STEPS.map((s, i) => (
              <li key={s.verb} className="how__item">
                <span className="how__n" aria-hidden="true">{i + 1}</span>
                <h3 className="how__verb">{s.verb}</h3>
                <p className="how__text">{s.text}</p>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="section">
        <div className="container">
          <h2 className="section__title">What you can buy.</h2>
          <div className="offers">
            <article className="offer offer--data">
              <h3 className="offer__title">Data bundles</h3>
              <p className="offer__text">Bundles for any MTN, Telecel or AirtelTigo number, yours or someone else’s.</p>
              <table className="deliverytable">
                <caption className="sr-only">Typical delivery time by network</caption>
                <thead>
                  <tr><th scope="col">Network</th><th scope="col">Usually delivered in</th></tr>
                </thead>
                <tbody>
                  {NETWORKS.map((n) => (
                    <tr key={n.id}>
                      <th scope="row"><NetworkChip network={n.id} long /></th>
                      <td>{n.delivery}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <Button to="/data" variant="primary">Browse bundles</Button>
            </article>

            <article className="offer offer--results" data-tint="results">
              <h3 className="offer__title">Results checkers</h3>
              <p className="offer__text">A PIN and serial number to check results online. Delivered on screen and by email.</p>
              <ul className="pricelist">
                {RESULTS_PRODUCTS.map((p) => (
                  <li key={p.id}>
                    <span>{p.name}</span>
                    <span className="pricelist__price">{money(RESULTS_REGULAR_PRICE)}</span>
                  </li>
                ))}
              </ul>
              <p className="offer__note">
                Buying more than {RESULTS_BULK_THRESHOLD}? Each one costs <strong>{money(RESULTS_BULK_PRICE)}</strong>.
              </p>
              <Button to="/results" variant="primary">Choose a checker</Button>
            </article>
          </div>
        </div>
      </section>

      {config.stores.enabled && (
        <section className="section">
          <div className="container split">
            <div className="split__text">
              <h2 className="section__title">Sell data under your own name.</h2>
              <ul className="checklist">
                <li><LuStore aria-hidden="true" /> Open a store in a couple of minutes: a name, a colour and your own prices.</li>
                <li><LuLink aria-hidden="true" /> Share one link. Customers pay by Mobile Money or card through Paystack.</li>
                <li><LuLayoutDashboard aria-hidden="true" /> Follow every order from your seller dashboard.</li>
              </ul>
              <div className="split__cta">
                <Button to="/store-builder" data-magnetic>Open your store</Button>
                <Button to="/marketplace" variant="secondary" data-magnetic>Browse the marketplace</Button>
              </div>
            </div>
            <figure className="codecard">
              <figcaption className="codecard__cap">One link for your customers</figcaption>
              <pre className="codecard__pre" tabIndex={0}><code>{`https://your-site.com/#/store/your-store-name

  choose a bundle
  enter the number to top up
  pay with Mobile Money or card
  your store delivers it`}</code></pre>
            </figure>
          </div>
        </section>
      )}

      <section className="section">
        <div className="container split">
          <div className="split__text">
            <h2 className="section__title">Buying often? Open an account.</h2>
            <ul className="checklist">
              <li><LuWallet aria-hidden="true" /> Keep a wallet balance and pay for bundles in one tap.</li>
              <li><LuPackage aria-hidden="true" /> See every order you have placed in one place.</li>
              <li><LuCode aria-hidden="true" /> Building an app or reselling? Use our REST API to place orders from your own system.</li>
            </ul>
            <div className="split__cta">
              <Button to="/signup">Create an account</Button>
              <Button to="/developers" variant="secondary">Read the API docs</Button>
            </div>
          </div>
          <figure className="codecard">
            <figcaption className="codecard__cap">Place an order from your own code</figcaption>
            <pre className="codecard__pre" tabIndex={0}><code>{`curl -X POST https://api.sci-fidata.com/v1/orders \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "bundle_id": "b1",
    "phone": "0244123456",
    "payment_method": "wallet"
  }'`}</code></pre>
          </figure>
        </div>
      </section>

      <section className="section">
        <div className="container faq">
          <h2 className="section__title">Questions people ask.</h2>
          <div className="faq__list panel">
            {FAQ.slice(0, 5).map((f) => (
              <details key={f.q} className="faq__item">
                <summary>{f.q}</summary>
                <p>{f.a}</p>
              </details>
            ))}
          </div>
          <p className="faq__more">More answers are in the <Link to="/how-it-works">buying guide</Link>.</p>
        </div>
      </section>

      <section className="cta">
        <div className="container">
          <div className="cta__inner">
            <div>
              <h2 className="cta__title">Top up your data now.</h2>
              <p className="cta__text">Most orders are delivered within two minutes.</p>
            </div>
            <div className="cta__actions">
              <Button to="/data" size="lg" data-magnetic>Buy data</Button>
              <Button href={site.whatsappUrl('Hi, I need help with an order')} target="_blank" rel="noreferrer" size="lg" variant="secondary" data-magnetic>
                Chat on WhatsApp
              </Button>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
