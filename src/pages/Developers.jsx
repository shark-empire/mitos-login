import { PageHeader, Button, Callout } from '../components/ui';
import { LuInfo } from 'react-icons/lu';
import { usePageTitle } from '../hooks/usePageTitle';

const ENDPOINTS = [
  {
    method: 'GET', path: '/v1/bundles', title: 'List bundles',
    desc: 'Returns every data bundle in the catalogue.',
    response: `{
  "data": [
    { "bundle_id": "b1", "size": "1GB", "price": 5.00, "network": "MTN" }
  ]
}`,
  },
  {
    method: 'POST', path: '/v1/orders', title: 'Create an order',
    desc: 'Places a data bundle order for a phone number.',
    body: `{
  "bundle_id": "b1",
  "phone": "0244123456",
  "payment_method": "wallet"
}`,
    response: `{
  "order_id": "SF3K9XA",
  "status": "processing",
  "bundle": "1GB",
  "amount": 5.00
}`,
  },
  {
    method: 'GET', path: '/v1/orders/:id', title: 'Get order status',
    desc: 'Returns the current status of an order.',
    response: `{
  "order_id": "SF3K9XA",
  "status": "completed",
  "bundle": "1GB",
  "network": "MTN",
  "created_at": "2025-01-15T10:30:00Z"
}`,
  },
  {
    method: 'GET', path: '/v1/wallet/balance', title: 'Get wallet balance',
    desc: 'Returns the wallet balance for the authenticated account.',
    response: `{
  "balance": 45.50,
  "currency": "GHS"
}`,
  },
  {
    method: 'POST', path: '/v1/wallet/fund', title: 'Fund wallet',
    desc: 'Starts a Paystack payment to add money to the wallet.',
    body: `{
  "amount": 50.00
}`,
    response: `{
  "payment_url": "https://checkout.paystack.com/...",
  "tx_ref": "SCIFI-1234567890-ABCXYZ"
}`,
  },
];

const EVENTS = [
  ['order.completed', 'Data was delivered'],
  ['order.failed', 'Delivery failed'],
  ['wallet.funded', 'Wallet was credited'],
  ['payment.received', 'Paystack confirmed a payment'],
];

export default function Developers() {
  usePageTitle('Developers');
  return (
    <div className="container page page--narrow">
      <PageHeader
        title="API documentation"
        lead="Place orders, check their status and manage your wallet from your own app."
        actions={<Button to="/support" variant="secondary">Request API access</Button>}
      />

      <Callout tone="info" icon={LuInfo}>
        API keys are issued on request. Contact support with a short note on what you are building.
      </Callout>

      <section className="prose-section">
        <h2 className="prose-section__title">Base URL</h2>
        <pre className="codeblock" tabIndex={0}><code>https://api.sci-fidata.com</code></pre>
      </section>

      <section className="prose-section">
        <h2 className="prose-section__title">Authentication</h2>
        <p>Send your API key as a bearer token on every request.</p>
        <pre className="codeblock" tabIndex={0}><code>{`Authorization: Bearer YOUR_API_KEY
Content-Type: application/json`}</code></pre>
      </section>

      <section className="prose-section">
        <h2 className="prose-section__title">Endpoints</h2>
        <div className="endpoints">
          {ENDPOINTS.map((ep) => (
            <details key={ep.path + ep.method} className="endpoint">
              <summary>
                <span className={`method method--${ep.method.toLowerCase()}`}>{ep.method}</span>
                <code className="endpoint__path">{ep.path}</code>
                <span className="endpoint__title">{ep.title}</span>
              </summary>
              <div className="endpoint__body">
                <p>{ep.desc}</p>
                {ep.body && (
                  <>
                    <h3 className="endpoint__label">Request body</h3>
                    <pre className="codeblock" tabIndex={0}><code>{ep.body}</code></pre>
                  </>
                )}
                <h3 className="endpoint__label">Response</h3>
                <pre className="codeblock" tabIndex={0}><code>{ep.response}</code></pre>
              </div>
            </details>
          ))}
        </div>
      </section>

      <section className="prose-section">
        <h2 className="prose-section__title">Webhook events</h2>
        <p>We send these events to the URL you configure when an order or payment changes state.</p>
        <table className="datatable">
          <thead><tr><th scope="col">Event</th><th scope="col">Meaning</th></tr></thead>
          <tbody>
            {EVENTS.map(([e, d]) => (
              <tr key={e}><th scope="row"><code>{e}</code></th><td>{d}</td></tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}
