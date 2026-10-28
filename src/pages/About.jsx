import { LuExternalLink, LuMessageCircle } from 'react-icons/lu';
import { Button, PageHeader } from '../components/ui';
import { usePageTitle } from '../hooks/usePageTitle';
import { site } from '../config/site';
import profile from '../config/profile';

const host = (url) => String(url || '').replace(/^https?:\/\//, '').replace(/\/+$/, '');
const label = (url) => host(url).replace(/^www\./, '').replace(/^(facebook|instagram|tiktok|x|twitter|linkedin|youtube)\.com\//, (m, n) => `${n[0].toUpperCase()}${n.slice(1)}: `);

/** Who runs the business. Everything here comes from src/config/profile.json; a blank detail is simply left out. */
export default function About() {
  usePageTitle('About us');
  const { brand, organization: org, founder, phone, email, social = [], alternateNames = [] } = profile;
  const others = alternateNames.filter((n) => n !== brand && !/\.store$/.test(n));
  const rows = [
    ['Name', <>{brand}{others.length ? <span className="about__aka"> (also written {others.join(', ')})</span> : null}</>],
    ['Website', host(profile.siteUrl)],
    org.legalName && ['Operated by', org.legalName],
    founder.name && [founder.title || 'Founder', founder.name],
    org.foundingYear && ['Founded', org.foundingYear],
    org.country && ['Based in', org.country],
    phone && ['WhatsApp and calls', phone],
    email && ['Email', <a key="m" href={`mailto:${email}`}>{email}</a>],
  ].filter(Boolean);

  return (
    <div className="container page page--narrow">
      <PageHeader title={`About ${brand}`} lead="Data bundles and WAEC results checkers for Ghana." />

      <div className="stack">
      <section className="panel stack">
        <h2 className="panel__title">What we do</h2>
        <p>
          {brand} sells MTN, Telecel and AirtelTigo data bundles and WAEC results checkers (WASSCE, NOVDEC and BECE). You pay with Mobile Money,
          card or bank transfer through Paystack, and most bundles arrive within a minute or two. Every order has a reference you can use to track it.
        </p>
      </section>

      <section className="panel stack">
        <h2 className="panel__title">The business</h2>
        <dl className="summary">
          {rows.map(([k, v]) => (
            <div key={k}><dt>{k}</dt><dd>{v}</dd></div>
          ))}
        </dl>
      </section>

      {founder.name && founder.about && (
        <section className="panel stack">
          <h2 className="panel__title">{founder.title || 'Founder'}: {founder.name}</h2>
          <p>{founder.about}</p>
        </section>
      )}

      <section className="panel stack">
        <h2 className="panel__title">Talk to us</h2>
        <div className="status__actions status__actions--left">
          <Button href={site.whatsappUrl(`Hi ${brand}, I have a question.`)} target="_blank" rel="noreferrer"><LuMessageCircle aria-hidden="true" /> Message us on WhatsApp</Button>
          {social.map((u) => (
            <Button key={u} variant="secondary" href={u} target="_blank" rel="noreferrer me"><LuExternalLink aria-hidden="true" /> {label(u)}</Button>
          ))}
        </div>
      </section>
      </div>
    </div>
  );
}
