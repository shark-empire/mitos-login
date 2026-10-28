import { useCallback, useEffect, useState } from 'react';
import { LuPackage } from 'react-icons/lu';
import { Badge, Button, Empty, NetworkChip, PageHeader, Segmented, TextField } from '../../components/ui';
import { useMyStore } from '../../components/layout/SellerLayout';
import { useSeller } from '../../context/Seller';
import { useToast } from '../../context/Toast';
import { usePageTitle } from '../../hooks/usePageTitle';
import { NETWORKS } from '../../config/site';
import { money } from '../../lib/format';
import { createBundle, fetchBundlesByStore, setBundleActive, VALIDITIES } from '../../services/stores';

const EMPTY = { name: '', network: 'MTN', size: '', price: '', validity: '90 days' };

export default function Bundles() {
  usePageTitle('Store bundles');
  const { store } = useMyStore();
  const { authed } = useSeller();
  const toast = useToast();
  const [bundles, setBundles] = useState(null);
  const [failed, setFailed] = useState(false);
  const [f, setF] = useState(EMPTY);
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);
  const [toggling, setToggling] = useState(null);

  const load = useCallback(() => {
    setFailed(false);
    return authed((t) => fetchBundlesByStore(store.id, { token: t, includeHidden: true }))
      .then(setBundles)
      .catch(() => {
        setFailed(true);
        setBundles((b) => b || []);
      });
  }, [store.id, authed]);

  useEffect(() => {
    load();
  }, [load]);

  const size = parseFloat(f.size);
  const price = parseFloat(f.price);
  const errors = {
    size: !(size > 0 && size <= 1000) && 'Enter the size in GB, for example 5 (or 0.5 for 500MB)',
    price: !(price >= 0.5 && price <= 5000) && 'Enter a price between GH₵ 0.50 and GH₵ 5,000',
  };
  const show = (k) => (touched && errors[k]) || undefined;
  const set = (k) => (e) => setF((p) => ({ ...p, [k]: e.target.value }));

  const add = async (e) => {
    e.preventDefault();
    setTouched(true);
    if (Object.values(errors).some(Boolean)) return;
    setBusy(true);
    try {
      const sizeLabel = size < 1 ? `${Math.round(size * 1000)}MB` : `${Number(size.toFixed(2))}GB`;
      await authed((t) =>
        createBundle(
          { storeId: store.id, name: f.name.trim() || `${sizeLabel} ${f.network}`, network: f.network, sizeGb: size, price, validity: f.validity },
          t,
        ),
      );
      toast.ok('Bundle added.');
      setF((p) => ({ ...EMPTY, network: p.network, validity: p.validity }));
      setTouched(false);
      await load();
    } catch (err) {
      toast.err(err.message || 'We could not add that bundle.');
    } finally {
      setBusy(false);
    }
  };

  const toggle = async (b) => {
    setToggling(b.id);
    try {
      await authed((t) => setBundleActive(b.id, !b.active, t));
      setBundles((list) => list.map((x) => (x.id === b.id ? { ...x, active: !b.active } : x)));
      toast.ok(b.active ? 'Hidden from your store.' : 'Back on sale.');
    } catch (err) {
      toast.err(err.message || 'We could not change that bundle.');
    } finally {
      setToggling(null);
    }
  };

  return (
    <>
      <PageHeader title="Bundles" lead="What customers can buy from your store. You set the price." />

      <form className="panel stack" onSubmit={add} noValidate>
        <div className="panel__head"><h2 className="panel__title">Add a bundle</h2></div>
        <Segmented label="Network" value={f.network} onChange={(v) => setF((p) => ({ ...p, network: v }))} options={NETWORKS.map((n) => ({ value: n.id, label: n.label }))} />
        <div className="fields2">
          <TextField label="Size (GB)" type="number" inputMode="decimal" min="0.1" step="any" placeholder="5" value={f.size} onChange={set('size')} error={show('size')} />
          <TextField label="Your price (GH₵)" type="number" inputMode="decimal" min="0.5" step="0.01" placeholder="22.00" value={f.price} onChange={set('price')} error={show('price')} />
        </div>
        <div className="fields2">
          <TextField label="Name (optional)" placeholder="Leave blank to use the size" maxLength={40} value={f.name} onChange={set('name')} />
          <TextField label="Valid for" as="select" value={f.validity} onChange={set('validity')}>
            {VALIDITIES.map((v) => <option key={v} value={v}>{v}</option>)}
          </TextField>
        </div>
        <div><Button type="submit" loading={busy}>Add bundle</Button></div>
      </form>

      <section className="panel">
        <div className="panel__head"><h2 className="panel__title">Your bundles</h2></div>
        {bundles === null && <div className="skeleton-block" aria-busy="true" />}
        {bundles && failed && <Empty icon={LuPackage} title="We couldn’t load your bundles" action={<Button size="sm" onClick={load}>Try again</Button>}>Check your connection.</Empty>}
        {bundles && !failed && bundles.length === 0 && <Empty icon={LuPackage} title="No bundles yet">Add your first one above. Customers see it straight away.</Empty>}
        {bundles && bundles.length > 0 && (
          <div className="tablewrap">
            <table className="datatable datatable--stack">
              <thead>
                <tr>
                  <th scope="col">Bundle</th>
                  <th scope="col">Network</th>
                  <th scope="col">Price</th>
                  <th scope="col">Valid for</th>
                  <th scope="col">Status</th>
                  <th scope="col"><span className="sr-only">Actions</span></th>
                </tr>
              </thead>
              <tbody>
                {bundles.map((b) => (
                  <tr key={b.id}>
                    <th scope="row" data-label="Bundle">{b.name}{!b.name.toLowerCase().includes(b.size.toLowerCase()) && <span className="muted"> ({b.size})</span>}</th>
                    <td data-label="Network"><NetworkChip network={b.network} /></td>
                    <td data-label="Price">{money(b.price)}</td>
                    <td data-label="Valid for">{b.validity}</td>
                    <td data-label="Status"><Badge tone={b.active ? 'ok' : 'muted'}>{b.active ? 'On sale' : 'Hidden'}</Badge></td>
                    <td data-label="">
                      <Button variant="secondary" size="sm" loading={toggling === b.id} onClick={() => toggle(b)}>{b.active ? 'Hide' : 'Show'}</Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </>
  );
}
