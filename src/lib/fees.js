import { config } from '../config/env';

/**
 * What it costs to put `amount` cedis into the wallet.
 * The wallet gets exactly `credit`; the customer pays `total`, which includes the Paystack fee.
 * Works in pesewas so the sum is exact (no 0.1 + 0.2 surprises) and the fee rounds to the nearest pesewa.
 */
export function walletTopUp(amount, percent = config.wallet.feePercent) {
  const creditP = Math.round((Number(amount) || 0) * 100);
  const feeP = Math.round((creditP * percent) / 100);
  return { credit: creditP / 100, fee: feeP / 100, total: (creditP + feeP) / 100, percent };
}
