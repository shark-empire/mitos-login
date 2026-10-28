import { useState } from 'react';
import { LuExternalLink, LuMail, LuMessageCircle } from 'react-icons/lu';
import { Button, PageHeader, TextField } from '../components/ui';
import { usePageTitle } from '../hooks/usePageTitle';
import { config } from '../config/env';
import { site } from '../config/site';
import { useAuth } from '../context/Auth';

const TOPICS = [
  'Data bundle not received',
  'Payment issue',
  'Wrong number entered',
  'Wallet not credited',
  'Results checker problem',
  'API access',
  'Something else',
];

export default function Support() {
  usePageTitle('Support');
  const { user } = useAuth();
  const [topic, setTopic] = useState(TOPICS[0]);
  const [reference, setReference] = useState('');
  const [message, setMessage] = useState('');

  const text = [
    `Hi, I need help: ${topic}.`,
    reference.trim() && `Reference: ${reference.trim()}`,
    message.trim(),
    user?.first_name && `– ${user.first_name}`,
  ].filter(Boolean).join('\n');

  return (
    <div className="container page">
      <PageHeader title="Support" lead="Tell us what went wrong and we’ll pick it up on WhatsApp. Having your payment reference to hand makes it much faster." />

      <div className="supportgrid">
        <form
          className="panel supportform"
          onSubmit={(e) => {
            e.preventDefault();
            window.open(site.whatsappUrl(text), '_blank', 'noopener');
          }}
        >
          <h2 className="panel__title">Send us the details</h2>
          <TextField as="select" label="What do you need help with?" value={topic} onChange={(e) => setTopic(e.target.value)}>
            {TOPICS.map((t) => <option key={t}>{t}</option>)}
          </TextField>
          <TextField
            label="Payment reference or order ID"
            placeholder="SF-1718123456-482910"
            value={reference}
            onChange={(e) => setReference(e.target.value)}
            hint="Optional, but it helps us find your order."
          />
          <TextField as="textarea" rows={5} label="Message" placeholder="Tell us what happened" value={message} onChange={(e) => setMessage(e.target.value)} />
          <Button type="submit" size="lg"><LuMessageCircle aria-hidden="true" /> Continue on WhatsApp</Button>
          <p className="field__hint">This opens WhatsApp with your message ready to send.</p>
        </form>

        <div className="supportside">
          <a className="contactcard" href={site.whatsappUrl()} target="_blank" rel="noreferrer">
            <LuMessageCircle aria-hidden="true" />
            <span><strong>WhatsApp</strong><span>+{config.contact.whatsapp}</span></span>
            <LuExternalLink className="contactcard__ext" aria-hidden="true" />
          </a>
          <a className="contactcard" href={`mailto:${config.contact.supportEmail}`}>
            <LuMail aria-hidden="true" />
            <span><strong>Email</strong><span>{config.contact.supportEmail}</span></span>
          </a>
          <div className="panel">
            <h2 className="panel__title">Before you write</h2>
            <ul className="bullets">
              <li>Data usually arrives within 2 minutes, and up to 30 minutes when networks are busy.</li>
              <li>Data sent to a wrong number cannot be recalled.</li>
              <li>You can check delivery any time on <a href="#/track">Track order</a>.</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
