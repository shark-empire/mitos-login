import { useId, useState } from 'react';
import { LuCheck, LuCopy, LuLoaderCircle } from 'react-icons/lu';
import { Link } from '../router';
import { NETWORKS } from '../config/site';
import { statusInfo } from '../lib/format';

export function Spinner({ size = 16 }) {
  return <LuLoaderCircle className="spin" size={size} aria-hidden="true" />;
}

/** Renders a router link (`to`), a plain link (`href`) or a button. */
export function Button({ variant = 'primary', size, block, loading, to, href, className = '', disabled, children, ...rest }) {
  const cls = ['btn', `btn--${variant}`, size && `btn--${size}`, block && 'btn--block', className].filter(Boolean).join(' ');
  if (to) return <Link to={to} className={cls} {...rest}>{children}</Link>;
  if (href) return <a href={href} className={cls} {...rest}>{children}</a>;
  return (
    <button type="button" className={cls} disabled={loading || disabled} aria-busy={loading || undefined} {...rest}>
      {loading && <Spinner />}
      {children}
    </button>
  );
}

/** Label + control + hint/error, wired together for assistive tech. Use `as="select"` or `as="textarea"`. */
export function TextField({ label, hint, error, prefix, suffix, id, className = '', as: Tag = 'input', children, ...props }) {
  const auto = useId();
  const fid = id || auto;
  const hintId = `${fid}-hint`;
  const message = error || hint;
  return (
    <div className={`field ${className}`.trim()}>
      {label && (
        <label htmlFor={fid} className="field__label">
          {label}
        </label>
      )}
      <div className={`field__control${error ? ' has-error' : ''}`}>
        {prefix && <span className="field__affix">{prefix}</span>}
        <Tag
          id={fid}
          className="field__input"
          aria-invalid={error ? true : undefined}
          aria-describedby={message ? hintId : undefined}
          {...props}
        >
          {children}
        </Tag>
        {suffix}
      </div>
      {message && (
        <p id={hintId} className={`field__hint${error ? ' is-error' : ''}`}>
          {message}
        </p>
      )}
    </div>
  );
}

export function Badge({ tone = 'muted', children }) {
  return <span className={`badge badge--${tone}`}>{children}</span>;
}

export function StatusBadge({ status }) {
  const { tone, label } = statusInfo(status);
  return <Badge tone={tone}>{label}</Badge>;
}

export function NetworkChip({ network, long = false }) {
  const n = NETWORKS.find((x) => x.id === network);
  return <span className={`netchip netchip--${(n?.meta || 'other')}`}>{n ? (long ? n.long : n.label) : network}</span>;
}

export function PageHeader({ title, lead, actions }) {
  return (
    <header className="pagehead">
      <div className="pagehead__text">
        <h1 className="pagehead__title">{title}</h1>
        {lead && <p className="pagehead__lead">{lead}</p>}
      </div>
      {actions && <div className="pagehead__actions">{actions}</div>}
    </header>
  );
}

export function Empty({ icon: Icon, title, children, action }) {
  return (
    <div className="empty">
      {Icon && <Icon className="empty__icon" aria-hidden="true" />}
      <h3 className="empty__title">{title}</h3>
      {children && <p className="empty__text">{children}</p>}
      {action}
    </div>
  );
}

export function Callout({ tone = 'info', icon: Icon, children }) {
  return (
    <div className={`callout callout--${tone}`} role={tone === 'bad' ? 'alert' : undefined}>
      {Icon && <Icon className="callout__icon" aria-hidden="true" />}
      <div className="callout__body">{children}</div>
    </div>
  );
}

/** Segmented control (radio-style). */
export function Segmented({ label, options, value, onChange }) {
  return (
    <div className="segmented" role="group" aria-label={label}>
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          className={`segmented__item${value === o.value ? ' is-on' : ''}`}
          aria-pressed={value === o.value}
          disabled={o.disabled}
          onClick={() => onChange(o.value)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function CopyButton({ text, label = 'Copy', onCopied }) {
  const [done, setDone] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setDone(true);
      onCopied && onCopied();
      setTimeout(() => setDone(false), 1800);
    } catch {
      /* clipboard blocked: the text stays selectable */
    }
  };
  return (
    <button type="button" className="copybtn" onClick={copy}>
      {done ? <LuCheck aria-hidden="true" /> : <LuCopy aria-hidden="true" />}
      <span>{done ? 'Copied' : label}</span>
    </button>
  );
}

/** The voucher motif: a panel with a perforated tear line and punched edges. */
export function Ticket({ top, bottom, className = '' }) {
  return (
    <div className={`ticket ${className}`.trim()}>
      <div className="ticket__top">{top}</div>
      <div className="ticket__tear" aria-hidden="true" />
      <div className="ticket__bottom">{bottom}</div>
    </div>
  );
}
