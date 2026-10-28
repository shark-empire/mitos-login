// Placeholder mark: replace public/logo.svg (and this wordmark) with your own brand artwork.
export default function Logo({ compact = false }) {
  return (
    <span className="logo">
      <svg className="logo__mark" viewBox="0 0 32 32" aria-hidden="true">
        <rect width="32" height="32" rx="9" fill="currentColor" />
        <circle cx="10" cy="22" r="2.6" fill="#fff" />
        <path d="M10 14.6a7.4 7.4 0 0 1 7.4 7.4" fill="none" stroke="#fff" strokeWidth="2.4" strokeLinecap="round" />
        <path d="M10 8a14 14 0 0 1 14 14" fill="none" stroke="#FFC62E" strokeWidth="2.4" strokeLinecap="round" />
      </svg>
      {!compact && <span className="logo__word">SCI-FI DATA</span>}
    </span>
  );
}
