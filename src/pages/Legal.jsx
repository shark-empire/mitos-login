import { PageHeader } from '../components/ui';
import { usePageTitle } from '../hooks/usePageTitle';
import { config } from '../config/env';
import { site } from '../config/site';

const UPDATED = 'October 2026';

function Doc({ title, sections }) {
  usePageTitle(title);
  return (
    <div className="container page page--narrow">
      <PageHeader title={title} lead={`Last updated ${UPDATED}`} />
      <div className="legal">
        {sections.map(([h, p]) => (
          <section key={h}>
            <h2>{h}</h2>
            <p>{p}</p>
          </section>
        ))}
      </div>
    </div>
  );
}

export function Terms() {
  return (
    <Doc
      title="Terms and conditions"
      sections={[
        ['1. Our service', `${site.name} resells mobile data bundles and WAEC results checker vouchers to customers in Ghana. We help you buy and receive data on MTN, Telecel and AirtelTigo numbers.`],
        ['2. Accounts and guest purchases', 'You can buy as a guest with just a phone number, or create an account. Account holders must give accurate details and are responsible for keeping their sign-in details safe.'],
        ['3. Payments', 'All payments are processed by Paystack, in Ghanaian cedis (GHS). We do not see or store your card details or Mobile Money PIN.'],
        ['4. Delivery', 'Data is usually delivered within 2 minutes but can take up to 30 minutes when networks are busy. If your data has not arrived after 30 minutes, contact us on WhatsApp with your payment reference.'],
        ['5. Correct numbers', 'Check the recipient number before you pay. Data sent to a wrong number cannot be recalled and is not refundable.'],
        ['6. Refunds', 'Orders are not refundable once data has been delivered to the recipient number. Wallet funds are not refundable once credited.'],
        ['7. Availability', 'We aim to keep the service running at all times but cannot guarantee uninterrupted access. Delivery can be delayed by network congestion or maintenance by the telecom providers.'],
        ['8. Acceptable use', 'You may not resell our services without written permission, try to bypass our payment systems, or use our API for anything other than what it was issued for.'],
        ['9. Governing law', 'These terms are governed by the laws of the Republic of Ghana.'],
      ]}
    />
  );
}

export function Privacy() {
  return (
    <Doc
      title="Privacy policy"
      sections={[
        ['What we collect', 'Your name, email address, phone number and payment references. For account holders we also keep your wallet balance and order history.'],
        ['How we use it', 'We use your details to process orders, manage wallet balances, send order confirmations and give you support. We do not sell your data.'],
        ['Payment security', 'Payment details are handled entirely by Paystack and are never stored on our servers. We only receive a confirmation and a reference.'],
        ['How long we keep it', 'Account data is kept for as long as your account exists plus 12 months. Guest order data is kept for 90 days.'],
        ['Your rights', `You can ask to see, correct or delete your personal data by writing to ${config.contact.supportEmail}.`],
        ['Cookies and storage', 'We use your browser’s storage to keep you signed in, remember your theme and list recent order references on your own device. We do not use advertising or tracking cookies.'],
        ['Contact', `For privacy questions, email ${config.contact.privacyEmail}.`],
      ]}
    />
  );
}
