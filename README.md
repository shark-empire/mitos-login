# SCI-FI DATA

One React site that replaces four separate projects:

| Original project | Now lives at |
| --- | --- |
| Sci-Fi Data static site (guest data checkout, owner lookup, order tracking) | `/data`, `/data/success`, `/track` |
| Babor (WAEC results-checker PIN store) | `/results` |
| Babor-Api (accounts, wallet, orders, developer page) | `/login`, `/signup`, `/account/*`, `/developers` |
| Store (reseller marketplace: stores, store builder, seller dashboard) | `/marketplace`, `/stores`, `/store/:slug`, `/bundle/:id`, `/store-builder`, `/seller/*` |

Stack: Vite 7, React 19, `react-icons`. No UI framework, plain CSS with design tokens (dark and light themes).

## Run it

```bash
npm install
npm run dev        # http://localhost:5173
npm run build      # outputs ./dist
npm run preview    # serve the production build locally
```

Node 20.19 or newer is required (Vite 7).

## Deploy

The router is hash based (`#/data`) and Vite uses `base: './'`, so the `dist` folder works on any static host, in any sub-folder, with no rewrite rules.

- **GitHub Pages**: push to `main`. `.github/workflows/deploy.yml` builds and publishes. In the repository go to Settings > Pages > Source: GitHub Actions.
- **Netlify**: import the repository. `netlify.toml` already sets the build command and publish folder.
- **Vercel**: import the repository. The Vite preset is detected automatically.
- **Anywhere else**: run `npm run build` and upload `dist/`.

## Configuration

Everything is optional. The defaults in `src/config/env.js` are the values found in your three projects, so the site works as soon as it is built. To override one, copy `.env.example` to `.env` (local) or set the same names in your host's environment settings (or as repository variables for the GitHub Actions workflow).

`VITE_*` values are compiled into the public JavaScript. Only put public or publishable keys there.

| Variable | Used for | Default |
| --- | --- | --- |
| `VITE_PAYSTACK_PUBLIC_KEY` | Data bundles and wallet top-ups | live key from the original projects |
| `VITE_RESULTS_PAYSTACK_KEY` | Results checkers | **test** key from Babor |
| `VITE_DATA_SUPABASE_URL`, `VITE_DATA_SUPABASE_ANON_KEY` | Bundle catalogue (`bundles`) and payment confirmations (`transactions`) | Sci-Fi Data project |
| `VITE_ACCOUNTS_SUPABASE_URL`, `VITE_ACCOUNTS_SUPABASE_ANON_KEY` | Sign-in, `users`, `orders`, `wallet_transactions` | Babor-Api project |
| `VITE_STORES_SUPABASE_URL`, `VITE_STORES_SUPABASE_ANON_KEY` | Reseller stores: `stores`, `bundles`, `orders`, sellers' sign-in | URL from the Store project; **no key**, so stores stay off until you set it |
| `VITE_STORES_PAYSTACK_PUBLIC_KEY` | Customers paying a store | same as `VITE_PAYSTACK_PUBLIC_KEY` |
| `VITE_STORE_LINK_BASE` | Where store owners' share links point, for example `https://shop.yourdomain.com` | empty, so wherever this site is hosted |
| `VITE_STORES_PAYER_EMAIL_DOMAIN` | The part after the `@` of the email Paystack shows to a store's customers | `sci-fidata.com`, so set it to your store domain |
| `VITE_SITE_MODE` | `stores` builds a stores-only site: no main-site pages at all (see "What a store's customers see") | empty, the full site |
| `VITE_OWNER_LOOKUP_URL` | Recipient name lookup | original Activepieces flow |
| `VITE_ORDER_LOOKUP_URL` | Track order by reference | original Activepieces flow |
| `VITE_WALLET_PURCHASE_URL` | Pay for a bundle from the wallet | placeholder, so wallet payment is hidden |
| `VITE_RESULTS_VERIFY_URL` | Verify a results-checker payment and return PINs | placeholder, so PIN delivery is not live |
| `VITE_DATA_TRANSFER_URL` | `/admin/transfer` tool | empty, so the tool is disabled |
| `VITE_WALLET_FEE_PERCENT` | Paystack fee added when someone adds money to their wallet (0 turns it off) | `3` |
| `VITE_WHATSAPP_NUMBER`, `VITE_SUPPORT_EMAIL` | Contact links | from the original site |

## Stores (the Store project)

Anyone can open a store, set their own prices and share one link; customers pay by Mobile Money or card and the store delivers the data.

**Switching it on.** The Store project kept its Supabase key in a `.env` that was not in the upload, so there is no default. Set `VITE_STORES_SUPABASE_ANON_KEY` (and `VITE_STORES_SUPABASE_URL` if it is not the project in `src/config/env.js`) and rebuild. Until then the marketplace, the store builder and the seller area are hidden from the menus and their pages say they are not switched on. The rest of the site is unaffected.

**What the pages do**

| Page | What it is |
| --- | --- |
| `#/marketplace` | Every live bundle from every open store, with network filter, search and sort |
| `#/stores` | Directory of open stores |
| `#/s/<slug>` | **The link a seller shares.** The store's shop and checkout in a frame with no site menu (see below) |
| `#/s/<slug>/track` | The store's order tracker for its customers |
| `#/store/<slug>` | The same store inside the full site (menu, footer), as marketplace visitors see it |
| `#/bundle/<id>` | One bundle, with checkout |
| `#/store-builder` | Four steps: seller account, store details (name, description, WhatsApp, logo), colour, launch |
| `#/seller`, `/seller/orders`, `/seller/bundles` | Seller dashboard: sales, orders, add bundles, hide or show a bundle |
| `#/seller/login` | Seller sign-in and sign-up |

Sellers sign in to the Store project, which is a different Supabase project from customer accounts. The two sessions are kept apart, so signing in as a seller never signs a customer out.

**Tables and rules the backend needs** (taken from the Store code)

- `stores` (`owner_id`, `name`, `slug`, `description`, `logo_url`, `accent_color`, `whatsapp`, `is_active`), `bundles` (`store_id`, `name`, `network`, `size_gb`, `price`, `validity`, `is_active`, with a foreign key to `stores`), `orders` (`bundle_id`, `store_id`, `customer_phone`, `amount`, `paystack_reference`, `payment_method`, `status`), and a storage bucket `assets` for logos. The optional function `get_store_stats(p_store_id)` feeds the dashboard totals; without it they are worked out from the latest 200 orders.
- The network is stored as `MTN`, `Telecel` or `AT`. The site shows `AT` as AirtelTigo and writes it back as `AT`.
- **Row-level security.** Visitors must be able to read live stores and bundles and insert an order; owners must be able to read their own orders and insert or update their own store and bundles; nobody should be able to edit another store.
- **A customer's browser creates the order row, so it is not proof of payment.** The row is saved as `pending` with the Paystack reference. Whatever delivers the data must verify the reference with Paystack, check that the amount paid matches `bundles.price` for that `bundle_id`, and only then deliver and mark the order `completed`. The Paystack metadata (`type: 'bundle_purchase'`, `phone`, `network`, `bundle_id`, `store_id`, `bundle_size`) is sent so a webhook can do this on its own, without relying on the browser.

### What a store's customers see

A store owner's link (`#/s/<slug>`) opens a customer view with nothing else in it: the store's name and logo, **Shop** and **Track order**, the checkout and a WhatsApp link to the store. There is no site menu, no footer, no log-in, no marketplace and no link to the main site. The browser tab shows only the store's name and a tab icon made from its logo (or its first letter), and the page is tinted with the store's colour. After paying, the customer gets a receipt with their reference and a **Track this order** button; the device also remembers their orders so the tracker lists them.

**Order tracking needs one function in the Store project's database.** Run this once in the Supabase SQL editor. It returns only what a customer should see (status, bundle, amount, a masked phone number) for one exact payment reference, so the `orders` table itself stays private:

```sql
create or replace function public.track_store_order(p_reference text)
returns json
language sql
stable
security definer
set search_path = public
as $$
  select json_build_object(
    'status',      o.status,
    'amount',      o.amount,
    'created_at',  o.created_at,
    'bundle_name', b.name,
    'size_gb',     b.size_gb,
    'network',     b.network,
    'phone',       left(o.customer_phone::text, 3) || '****' || right(o.customer_phone::text, 3),
    'store_name',  s.name,
    'store_slug',  s.slug
  )
  from orders o
  left join bundles b on b.id::text = o.bundle_id::text
  left join stores  s on s.id::text = o.store_id::text
  where o.paystack_reference = p_reference
  limit 1;
$$;

grant execute on function public.track_store_order(text) to anon, authenticated;
```

The joins compare ids as text so the function works whatever types your id columns have. Check that `orders.bundle_id` and `bundles.id` have the **same type** (both `uuid`, or both `bigint`): the site saves each order with the bundle's id, and the database refuses the order if the types differ. This query shows them:

```sql
select table_name, column_name, data_type
from information_schema.columns
where table_schema = 'public'
  and ((table_name = 'orders'  and column_name in ('bundle_id', 'store_id'))
    or (table_name in ('bundles', 'stores') and column_name = 'id'))
order by table_name, column_name;
```

Without it the tracker tries to read the `orders` table directly, which only works if your policies let visitors read orders. Do not open the table to everyone just for this; use the function. A customer can only look up an order whose reference they were given (it contains a timestamp and random letters), and another store's references are shown as "not found" on this store's page.

**Hiding the main site completely (the stores-only site).** On one website the main pages still exist at the site's own address, so someone who deletes `#/s/<slug>` from the link and opens the bare address would find them. To remove them entirely, publish a second copy of this project for customers with the variable `VITE_SITE_MODE=stores`. That build:

- contains only the store pages (the shop and the tracker). The home page, data bundles, results checkers, log-in, wallet, marketplace, seller area and admin pages are not compiled in, so every other address shows a blank "Open the link your store gave you" notice;
- has no accounts or wallet code, no logo or icon files, and a page shell with no brand name, description or icon;
- still opens the older `#/store/<slug>` links.

Set it up like this: create a second GitHub repository (for example `shop`) with the same files, add the repository variables `VITE_SITE_MODE` = `stores`, `VITE_STORES_SUPABASE_ANON_KEY`, and `VITE_STORES_PAYER_EMAIL_DOMAIN` (your store domain), turn on Pages, and point the store domain at it. On the **main** site set `VITE_STORE_LINK_BASE` = `https://your-store-domain` so every link owners copy uses the store domain. Owners still sign in, build and manage stores on the main site.

When you buy a domain, a good split is `yourdomain.com` for the main site and a separate, neutral-looking name or subdomain for stores (for example `shop.yourdomain.com`, or a different domain if you do not want any link between them). On GitHub Pages: **Settings > Pages > Custom domain**, then add the DNS records GitHub shows you at your registrar and tick **Enforce HTTPS** once it is ready. A GitHub Pages site can have one custom domain, which is why the stores-only site is a second repository.

**What customers can still find out.** The checkout window belongs to Paystack, and it shows the name of your Paystack business and the email address given to it (set `VITE_STORES_PAYER_EMAIL_DOMAIN` so that is your store domain, not the main site's). If your Paystack business name is the main site's name, customers will see it there; change it in your Paystack dashboard if you do not want that. Someone who opens the browser's developer tools on the stores-only site can read its code, which still contains the default settings (the main site's contact details and backend addresses), but no pages or links to it.

**What was left out or changed from the Store project**

- Left out: wallet top-up and pay-from-wallet (the site already has one wallet, and the Store's `deduct_wallet_balance` function took a user id from the browser), the phone-OTP login (it never created a session), the fabricated testimonials, and the "pretend the payment worked" fallback when Paystack could not load. Also dropped: Tailwind, framer-motion, three, lucide, the Supabase SDK and the debug console (eruda). The pages that used them were rebuilt in the site's own style.
- Fixed: the bundle validity typo (`validit`) that made every bundle `90DAYS`; the dashboard calling a function that did not exist; sellers' totals being worked out from only the last 50 orders; store links that would not have worked on a static host (they are now `#/s/<slug>` and are built from wherever the site is hosted, or from `VITE_STORE_LINK_BASE`).
- A store's accent colour is checked (`#rrggbb` only) before it reaches the page, and uploaded logos must be a PNG, JPG, WebP or GIF under 2 MB.

## Search and link previews

**What is set up.** The page's HTML carries everything a search engine or a chat app reads without running the site: a title and description that say who SCI-FI DATA is and name the domain, the canonical address (`https://scifidata.store/`), Open Graph and Twitter card tags, and structured data (JSON-LD) that declares the organization, every spelling of the name (`SCI-FI DATA`, `SciFi Data`, `Sci-Fi Data`, `SCIFI DATA`, `scifidata`, `scifidata.store`), the company that operates it, the founder, contact details and social profiles. The build also writes `robots.txt` and `sitemap.xml`. There is an **About us** page (`#/about`, linked from the footer) with the same facts for visitors.

**Edit one file: `src/config/profile.json`.** It holds the brand name, site address, spellings, title and description, the company (`organization.legalName`, `foundingYear`), the founder (`name`, `title`, `about`), phone, email, `social` profile links and the search-console verification codes. Anything left blank is simply left out; nothing is invented. Commit the change and the site redeploys. If your main address is `www.scifidata.store` rather than `scifidata.store`, change `siteUrl` to match.

**The picture shown when the link is shared** is `public/og-image.jpg` (1200 x 630, under 300 KB so WhatsApp shows it), made from your flyer. The full flyer is `public/flyer.jpg`. To change the picture, replace `og-image.jpg` with another 1200 x 630 JPEG under 300 KB. WhatsApp, Facebook and others remember the old preview for a while: for Facebook use its Sharing Debugger and click "Scrape again"; WhatsApp usually refreshes within a day, or share the link with `?v=2` on the end to see it at once.

**To get the domain showing in Google for the brand name** (this part only you can do, and no site can force it):

1. Add `https://scifidata.store` as a property in Google Search Console. Choose the HTML tag method, copy the code into `verification.google` in `profile.json`, redeploy, then press Verify. (Or verify with the DNS record at your domain registrar.)
2. In Search Console submit `https://scifidata.store/sitemap.xml`, then use URL Inspection on the home page and press "Request indexing".
3. Do the same in Bing Webmaster Tools (it can import your Google setup). Put its code in `verification.bing` if it asks for a tag.
4. Use the exact same name and website everywhere else you exist: WhatsApp Business profile, Facebook, Instagram, TikTok, X, LinkedIn and a Google Business Profile if you have a place or service area. Add those links to `social` so search engines can tie them together.

New sites usually take days to weeks to appear. Whether Google shows a panel about the founder or the company is Google's decision and depends on outside sources as well; the structured data only makes the facts easy for it to find.

**One limit to know.** The site uses `#` addresses (`#/data`, `#/about`), and search engines treat everything after the `#` as the same page. So the home page is what gets indexed; individual pages like Data bundles will not appear as separate results. Fixing that would mean switching to plain addresses (`/data`) and a host that serves them, which is a bigger change.

## Things you need to finish before taking real money

1. **Results checkers need a backend.** The original Babor project called `https://your-backend-url.com/api/verify-payment`, which never existed. Point `VITE_RESULTS_VERIFY_URL` at a service that verifies the Paystack reference **server-side** and returns `{ "pins": [{ "pin": "...", "serial": "..." }] }`. The page sends `{ reference, product_id, quantity }`. Until then the page shows a clear error and keeps the payment reference so the customer can contact you. Also set `VITE_RESULTS_PAYSTACK_KEY` to a live key when you go live: the default is a test key.
2. **Wallet top-ups carry a 3% fee.** On *Add money* the customer pays the amount plus 3% (Paystack's processing fee) and the wallet should receive only the amount they typed: pay GH₵ 51.50, get GH₵ 50.00. The Paystack charge is the total; the metadata sent with it is `{ type: 'wallet_funding', user_id, wallet_credit: 50, fee: 1.5, fee_percent: 3 }`. **Whatever credits the balance (your Paystack webhook or automation) must credit `wallet_credit`, not the amount Paystack reports**, or customers will be credited the fee as well. Change the rate with `VITE_WALLET_FEE_PERCENT`.
3. **Wallet payments.** The original wallet-purchase webhook was a `YOUR_WALLET_FLOW_ID` placeholder. Set `VITE_WALLET_PURCHASE_URL` and the "Pay from wallet" option appears automatically. The flow must debit the balance server-side.
4. **Rotate the exposed admin webhook.** The original `api.html` had the Make.com transfer webhook and an admin key in public source. Treat them as compromised and regenerate them. This site does not embed them. To use the admin page, set `VITE_DATA_TRANSFER_URL`; it is sent an `admin_key` that you type in, and it is still only a courtesy gate, so enforce authorisation in the webhook itself.
5. **Use one Supabase project.** The data catalogue and the account/wallet system point at two different Supabase projects. Wallet purchases send the catalogue's `bundle_id`, so the wallet flow has to understand those ids. Moving everything to one project (set both URL/key pairs to the same values) avoids this.
6. **Row-level security.** The anon keys are public by design, so your tables must have RLS policies that only let users read their own `orders`, `wallet_transactions` and `users` rows, and never let the browser change a balance. The original client credited wallets directly from the browser; that was removed, and the wallet balance is now only read, and refreshed after payment.
7. **Sign-up and login need the right Supabase auth settings.** In each Supabase project open *Authentication → Providers → Email*:
   - **Confirm email must be OFF** for phone-number accounts. They sign in with a made-up address (`0541234567@sci-fidata.com`) that has no inbox, so a confirmation email can never be opened. With it ON, phone sign-ups are created but cannot log in (the site then tells the customer to message support).
   - With **Confirm email ON**, email sign-ups still work: the site shows "Check your email" and offers to resend. Until the link is opened, login says "Your email isn't confirmed yet".
   - Supabase's built-in email sender allows only a few emails an hour. When that limit is hit, sign-up says so. For real traffic, add your own SMTP provider under *Authentication → SMTP Settings*.
   - The `users` table needs an insert policy that lets a signed-in user create the row whose `id` is their own, and (if `phone` is unique) must allow several rows with no phone. The site tries a few shapes and still signs the customer in if the row is refused; the row is rebuilt on their next login.
8. **Store orders need a verifying backend** (see Stores above). Without it, a paid store order stays `pending` forever.
9. **Run the order-tracking SQL** from "What a store's customers see" in your Store project, or the tracker cannot find orders.
10. **Brand.** `public/logo.svg` and `public/favicon.svg` are placeholder marks. Replace them with your own.

## What changed from the originals

- Fake or insecure pieces were dropped rather than carried over: client-side wallet crediting, base64 "password" handling, demo-mode fallback data, the embedded admin key, the sample API key on the developer page, invented statistics, and the lookup-by-phone order search (it exposed other people's orders).
- Prices are never made up. If the bundle catalogue cannot be loaded the page shows an error with a retry button.
- Sign-in uses Supabase Auth over its REST API; the session lives in `sessionStorage` and refreshes itself on a 401. Login and sign-up errors say what actually went wrong (unconfirmed email, wrong password, already registered, rate limit, offline) instead of one generic message.
- Paystack metadata and webhook payloads are unchanged, so your existing automations keep working.
- The NEXTSTEP promo flyer in the source archive was not used.

## Look and feel

The design is dark-first with a light theme (the toggle is in the header; a saved choice wins, and so does a light system setting).

- **Tinted liquid glass.** Panels, the header, the footer, the phone tab bar and the toasts are translucent glass: a blurred backdrop, a lit top-left edge and a tint. The tint is the colour of the glass and it follows what the visitor is looking at: the home headline cycles MTN, Telecel and AirtelTigo once and re-colours the whole page, Buy data takes the colour of the chosen network, and Results checkers is violet. Change the colours in `src/styles/tokens.css` (`[data-tint='mtn']` and friends).
- **Live layer** (`src/lib/live.js`, mounted by `src/components/LiveFX.jsx`): the backdrop orbs drift toward the pointer, a soft spotlight follows it, the glass under the pointer catches a highlight, cards lean toward it, the main buttons lean toward a nearby cursor, and every click sends out a ring with a few sparks, a ripple inside the pressed control and a small spring through the backdrop. It is one animation loop that sleeps when nothing moves. It is switched off for visitors who ask for reduced motion, and hover effects are skipped on touch screens (taps still get the click effects). To remove the whole layer, delete `<LiveFX />` from `src/App.jsx`.
- **Type and shape.** Large light-weight display type, pill buttons and chips, and corners that get softer as surfaces get bigger. Fonts load from Google Fonts (Bricolage Grotesque and Instrument Sans); the page still works without them.
- **Where things are:** `tokens.css` (colours, radii, the glass recipe numbers), `glass.css` (the backdrop, the glass surfaces and the click and highlight effects), `home.css` (the home page), `components.css` and `layout.css` (everything shared).

The look takes its ideas from the structure and public design language of shopify.com and nas.com (big light display type, pill buttons, a rotating hero word, short lowercase step headings, a stat band, chip rows). No logos, images, copy or brand assets from either site are used.

## Project layout

```
src/
  config/       env.js (runtime config), site.js (brand, networks, FAQ, results products)
  lib/          format, phone validation, ids/references, storage, rate limit, live (pointer and click effects)
  services/     rest.js (Supabase), webhooks.js, paystack.js, stores.js (reseller stores)
  context/      Theme, Toast, Auth, Seller (store owners' own sign-in)
  StoresApp.jsx the stores-only app used when VITE_SITE_MODE=stores
  hooks/        useBundles, usePageTitle, useTint (sets the colour of the glass)
  components/   ui.jsx, Logo, RateCard, LiveFX (backdrop + click layer), layout/ (Header, Footer, TabBar, layouts)
  pages/        public pages, account/ (signed in), admin/, stores/ (marketplace, store pages, builder), seller/ (dashboard)
  styles/       tokens, base, components, layout, pages, home, stores, glass
  router.jsx    small hash router
  App.jsx       providers and route tables
```

To add a page, create it in `src/pages` and add an entry to `PUBLIC_ROUTES`, `ACCOUNT_ROUTES` or `SELLER_ROUTES` in `src/App.jsx`. An address with one variable part (`/store/<slug>`) goes in `PARAM_ROUTES` and the page receives the value as the `param` prop.
