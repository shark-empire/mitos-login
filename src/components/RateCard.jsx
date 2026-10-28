import { useMemo, useState } from 'react';
import { LuRefreshCw } from 'react-icons/lu';
import { Link } from '../router';
import { useBundles } from '../hooks/useBundles';
import { NETWORKS } from '../config/site';
import { splitSize } from '../lib/format';
import { Ticket } from './ui';

const ROWS = 6;

/**
 * The hero's purchase module: pick a network, tap a bundle, go straight to checkout.
 * The network can be controlled by the parent (the home page cycles it); it falls back to its own state otherwise.
 */
export default function RateCard({ network: controlled, onNetwork }) {
  const { bundles, status, reload } = useBundles();
  const [own, setOwn] = useState('MTN');
  const network = controlled ?? own;
  const setNetwork = (id) => (onNetwork ? onNetwork(id) : setOwn(id));

  const list = useMemo(() => bundles.filter((b) => b.network === network), [bundles, network]);
  const shown = list.slice(0, ROWS);

  return (
    <Ticket
      className="ratecard"
      top={
        <>
          <div className="ratecard__head">
            <h2 className="ratecard__title">Today’s bundles</h2>
            <div className="ratecard__tabs" role="tablist" aria-label="Network">
              {NETWORKS.map((n) => (
                <button
                  key={n.id}
                  role="tab"
                  type="button"
                  aria-selected={network === n.id}
                  className={`ratecard__tab ratecard__tab--${n.meta}${network === n.id ? ' is-on' : ''}`}
                  onClick={() => setNetwork(n.id)}
                >
                  {n.label}
                </button>
              ))}
            </div>
          </div>

          <ul className="ratecard__rows" key={network}>
            {status === 'loading' &&
              Array.from({ length: ROWS }).map((_, i) => <li key={i} className="ratecard__row ratecard__row--skeleton" aria-hidden="true" />)}

            {status === 'error' && (
              <li className="ratecard__msg">
                <p>We couldn’t load the bundle list.</p>
                <button type="button" className="btn btn--secondary btn--sm" onClick={reload}>
                  <LuRefreshCw aria-hidden="true" /> Try again
                </button>
              </li>
            )}

            {status === 'ready' && shown.length === 0 && (
              <li className="ratecard__msg">
                <p>No {network} bundles are listed right now.</p>
              </li>
            )}

            {shown.map((b, i) => {
              const { num, unit } = splitSize(b.size);
              const inner = (
                <>
                  <span className="ratecard__size">
                    {num}
                    <span className="ratecard__unit">{unit}</span>
                  </span>
                  <span className="ratecard__leader" aria-hidden="true" />
                  <span className="ratecard__price">
                    <span className="ratecard__cur">GH₵</span>
                    {b.price.toFixed(2)}
                  </span>
                  <span className="ratecard__buy">{b.available ? 'Buy' : 'Sold out'}</span>
                </>
              );
              return (
                <li key={b.id} className="ratecard__row" style={{ '--i': i }}>
                  {b.available ? (
                    <Link to={`/data?network=${encodeURIComponent(b.network)}&bundle=${encodeURIComponent(b.id)}`} className="ratecard__link" aria-label={`Buy ${b.size} ${b.network} for GH₵ ${b.price.toFixed(2)}`}>
                      {inner}
                    </Link>
                  ) : (
                    <span className="ratecard__link is-off">{inner}</span>
                  )}
                </li>
              );
            })}
          </ul>
        </>
      }
      bottom={
        <Link to={`/data?network=${encodeURIComponent(network)}`} className="ratecard__more">
          {status === 'ready' && list.length > ROWS ? `See all ${list.length} ${network} bundles` : 'Open the full bundle list'}
        </Link>
      }
    />
  );
}
