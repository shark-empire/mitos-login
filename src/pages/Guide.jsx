import { PageHeader, NetworkChip, Button } from '../components/ui';
import { usePageTitle } from '../hooks/usePageTitle';
import { FAQ, NETWORKS } from '../config/site';

const STEPS = [
  { title: 'Pick your network', text: 'Choose MTN Ghana, Telecel or AirtelTigo, depending on whose SIM should get the data.' },
  { title: 'Choose a bundle', text: 'Select the size and price that suits you, from 1GB up to 10GB and beyond.' },
  { title: 'Enter the recipient’s number', text: 'Type the 10-digit number that should receive the data. We show the registered owner’s name so you can confirm it is correct.' },
  { title: 'Pay with Paystack', text: 'Tap Pay. A secure Paystack window opens where you can pay with Mobile Money, card or bank transfer.' },
  { title: 'Receive your data', text: 'Once payment is confirmed the bundle is sent automatically, usually within 2 minutes.' },
];

export default function Guide() {
  usePageTitle('How it works');
  return (
    <div className="container page page--narrow">
      <PageHeader title="How it works" lead="Buy and receive mobile data in under two minutes. Here is exactly what happens." />

      <section className="prose-section" aria-labelledby="g-steps">
        <h2 id="g-steps" className="prose-section__title">Buying data, step by step</h2>
        <ol className="steps steps--stack">
          {STEPS.map((s, i) => (
            <li key={s.title} className="steps__item">
              <span className="steps__n" aria-hidden="true">{i + 1}</span>
              <h3 className="steps__title">{s.title}</h3>
              <p className="steps__text">{s.text}</p>
            </li>
          ))}
        </ol>
        <Button to="/data">Buy data</Button>
      </section>

      <section className="prose-section" aria-labelledby="g-net">
        <h2 id="g-net" className="prose-section__title">Networks and delivery times</h2>
        <table className="datatable">
          <thead><tr><th scope="col">Network</th><th scope="col">Usually delivered in</th></tr></thead>
          <tbody>
            {NETWORKS.map((n) => (
              <tr key={n.id}><th scope="row"><NetworkChip network={n.id} long /></th><td>{n.delivery}</td></tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="prose-section" aria-labelledby="g-faq">
        <h2 id="g-faq" className="prose-section__title">Frequently asked questions</h2>
        <div className="faq__list">
          {FAQ.map((f) => (
            <details key={f.q} className="faq__item">
              <summary>{f.q}</summary>
              <p>{f.a}</p>
            </details>
          ))}
        </div>
      </section>
    </div>
  );
}
