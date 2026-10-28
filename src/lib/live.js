/*
  The "live" layer: the page answers the pointer and clicks.

  What it does
  - Backdrop: soft colour orbs drift a little toward/away from the pointer (parallax), and a tinted spotlight follows it.
  - Glass: the surface under the pointer catches a specular highlight that follows the cursor.
  - Tilt: cards lean toward the pointer in 3D.
  - Magnetic buttons: elements marked data-magnetic lean toward a nearby pointer.
  - Clicks: a shock ring and a few sparks at the click point, a ripple inside the control that was pressed,
    and a small impulse that makes the backdrop spring.

  Design rules
  - One rAF loop, written only to CSS variables / transforms, and it sleeps when nothing is moving.
  - Everything is skipped for visitors who prefer reduced motion. Hover effects are skipped on touch screens.
  - Nothing here is required to use the site: it only adds feedback.
*/

// Elements that catch the highlight. The first hover tags them with .fx-lit (see glass.css).
const LIT =
  '.header, .footer, .tabbar, .panel, .ticket, .offer, .account__who, .stat, .walletcard, .order, .contactcard, .endpoint, .codecard, .cta__inner, ' +
  '.how__item, .feature, .statband, .nettile, .bundle, .product, .mbundle, .storecard, .storehead, .quick__item, .recent__chip, .toast, .ratecard__link, .chip';

// Elements that lean toward the pointer.
const TILT = '.ratecard, .offer, .nettile, .bundle, .product, .mbundle, .storecard, .stat, .walletcard, .how__item, .feature, .contactcard, .order';

// Controls that show a ripple when pressed.
const RIPPLE =
  '.btn, .iconbtn, .ratecard__tab, .nettile, .bundle, .product, .mbundle, .storecard, .swatch, .quick__item, .recent__chip, .segmented__item, .account__link, .tabbar__item, ' +
  '.header__link, .copybtn, .chip, .rot, .stepper button, .how__item, .feature, .ratecard__link';

const clamp = (n, a, b) => Math.min(b, Math.max(a, n));

export function startLive({ aurora, layer }) {
  if (!aurora || !layer || typeof window === 'undefined') return () => {};
  const mq = (q) => window.matchMedia(q).matches;
  if (mq('(prefers-reduced-motion: reduce)')) return () => {};

  const hover = mq('(hover: hover) and (pointer: fine)');
  const state = {
    W: window.innerWidth,
    H: window.innerHeight,
    tx: window.innerWidth / 2,
    ty: window.innerHeight * 0.35,
    sx: window.innerWidth / 2,
    sy: window.innerHeight * 0.35,
    kx: 0,
    ky: 0,
    vx: 0,
    vy: 0,
    target: null,
    lit: null,
    tilt: null,
    inside: false,
    raf: 0,
  };
  const magnets = new Set();

  const setVar = (el, k, v) => el.style.setProperty(k, v);

  /* ---------- frame loop ---------- */
  function frame() {
    state.raf = 0;

    // Ease the pointer so the backdrop glides instead of snapping.
    state.sx += (state.tx - state.sx) * 0.1;
    state.sy += (state.ty - state.sy) * 0.1;

    // Spring for click impulses.
    state.vx += -state.kx * 0.04;
    state.vy += -state.ky * 0.04;
    state.vx *= 0.9;
    state.vy *= 0.9;
    state.kx += state.vx;
    state.ky += state.vy;

    const px = (state.sx / state.W) * 2 - 1;
    const py = (state.sy / state.H) * 2 - 1;
    setVar(aurora, '--px', px.toFixed(3));
    setVar(aurora, '--py', py.toFixed(3));
    setVar(aurora, '--kx', state.kx.toFixed(2));
    setVar(aurora, '--ky', state.ky.toFixed(2));
    setVar(aurora, '--cx', `${state.sx.toFixed(1)}px`);
    setVar(aurora, '--cy', `${state.sy.toFixed(1)}px`);

    if (hover) {
      track(state.target);
      magnetize();
    }

    const moving = Math.abs(state.tx - state.sx) > 0.3 || Math.abs(state.ty - state.sy) > 0.3;
    const springing = Math.abs(state.kx) > 0.05 || Math.abs(state.ky) > 0.05 || Math.abs(state.vx) > 0.05 || Math.abs(state.vy) > 0.05;
    if (moving || springing) wake();
  }

  function wake() {
    if (!state.raf) state.raf = requestAnimationFrame(frame);
  }

  /* ---------- highlight + tilt on the element under the pointer ---------- */
  function track(target) {
    const lit = state.inside && target && target.closest ? target.closest(LIT) : null;
    if (lit !== state.lit) {
      if (state.lit) state.lit.classList.remove('is-lit');
      state.lit = lit;
      if (lit) lit.classList.add('fx-lit', 'is-lit');
    }
    if (lit) {
      const r = lit.getBoundingClientRect();
      setVar(lit, '--lx', `${(state.tx - r.left).toFixed(0)}px`);
      setVar(lit, '--ly', `${(state.ty - r.top).toFixed(0)}px`);
    }

    const tilt = state.inside && target && target.closest ? target.closest(TILT) : null;
    if (tilt !== state.tilt) {
      releaseTilt(state.tilt);
      state.tilt = tilt;
      if (tilt) tilt.classList.add('is-tilting');
    }
    if (tilt) {
      const r = tilt.getBoundingClientRect();
      const nx = clamp((state.tx - r.left) / r.width, 0, 1);
      const ny = clamp((state.ty - r.top) / r.height, 0, 1);
      // Small tiles lean further than big panels, which would otherwise swing too much.
      const max = clamp(1500 / r.width, 2.4, 9);
      setVar(tilt, '--rx', `${((0.5 - ny) * 2 * max).toFixed(2)}deg`);
      setVar(tilt, '--ry', `${((nx - 0.5) * 2 * max).toFixed(2)}deg`);
    }
  }

  function releaseTilt(el) {
    if (!el) return;
    el.classList.remove('is-tilting');
    el.style.removeProperty('--rx');
    el.style.removeProperty('--ry');
  }

  function clearHover() {
    state.inside = false;
    if (state.lit) state.lit.classList.remove('is-lit');
    state.lit = null;
    releaseTilt(state.tilt);
    state.tilt = null;
    aurora.style.setProperty('--cop', '0');
    magnets.forEach((el) => (el.style.translate = ''));
  }

  /* ---------- magnetic buttons ---------- */
  function magnetize() {
    document.querySelectorAll('[data-magnetic]').forEach((el) => {
      const r = el.getBoundingClientRect();
      const cx = r.left + r.width / 2;
      const cy = r.top + r.height / 2;
      const dx = state.tx - cx;
      const dy = state.ty - cy;
      const reach = Math.max(96, r.width * 0.75);
      if (state.inside && Math.hypot(dx, dy) < reach) {
        magnets.add(el);
        el.style.translate = `${clamp(dx * 0.2, -9, 9).toFixed(1)}px ${clamp(dy * 0.3, -7, 7).toFixed(1)}px`;
      } else if (magnets.has(el)) {
        el.style.translate = '';
        magnets.delete(el);
      }
    });
  }

  /* ---------- clicks ---------- */
  function burst(x, y) {
    const ring = document.createElement('span');
    ring.className = 'fx-ring';
    ring.style.left = `${x}px`;
    ring.style.top = `${y}px`;
    layer.appendChild(ring);
    ring.addEventListener('animationend', () => ring.remove(), { once: true });

    for (let i = 0; i < 7; i++) {
      const a = (Math.PI * 2 * i) / 7 + Math.random() * 0.6;
      const d = 34 + Math.random() * 38;
      const dot = document.createElement('span');
      dot.className = 'fx-dot';
      dot.style.left = `${x}px`;
      dot.style.top = `${y}px`;
      dot.style.setProperty('--dx', `${(Math.cos(a) * d).toFixed(1)}px`);
      dot.style.setProperty('--dy', `${(Math.sin(a) * d).toFixed(1)}px`);
      layer.appendChild(dot);
      dot.addEventListener('animationend', () => dot.remove(), { once: true });
    }
  }

  function ripple(host, x, y) {
    const r = host.getBoundingClientRect();
    if (getComputedStyle(host).position === 'static') host.classList.add('fx-rel');
    const wrap = document.createElement('span');
    wrap.className = 'fx-rhost';
    const dot = document.createElement('span');
    dot.className = 'fx-ripple';
    const size = Math.max(r.width, r.height) * 2.4;
    dot.style.setProperty('--x', `${(x - r.left).toFixed(0)}px`);
    dot.style.setProperty('--y', `${(y - r.top).toFixed(0)}px`);
    dot.style.setProperty('--s', (size / 10).toFixed(1));
    wrap.appendChild(dot);
    host.appendChild(wrap);
    dot.addEventListener('animationend', () => wrap.remove(), { once: true });
  }

  function onDown(e) {
    if (e.button > 0) return;
    const x = e.clientX;
    const y = e.clientY;
    burst(x, y);
    const host = e.target.closest ? e.target.closest(RIPPLE) : null;
    if (host && !host.disabled) ripple(host, x, y);

    // Push the backdrop away from the click; the spring brings it back.
    state.vx += ((state.W / 2 - x) / state.W) * 7;
    state.vy += ((state.H / 2 - y) / state.H) * 7;

    if (e.pointerType === 'touch') {
      // No hover on touch: show the highlight where the finger is, briefly.
      const lit = e.target.closest ? e.target.closest(LIT) : null;
      if (lit) {
        const r = lit.getBoundingClientRect();
        lit.classList.add('fx-lit', 'is-lit');
        setVar(lit, '--lx', `${(x - r.left).toFixed(0)}px`);
        setVar(lit, '--ly', `${(y - r.top).toFixed(0)}px`);
        setTimeout(() => lit.classList.remove('is-lit'), 650);
      }
    }
    wake();
  }

  /* ---------- pointer + page events ---------- */
  function onMove(e) {
    if (e.pointerType === 'touch') return;
    state.tx = e.clientX;
    state.ty = e.clientY;
    state.target = e.target;
    if (!state.inside) {
      state.inside = true;
      aurora.style.setProperty('--cop', '1');
    }
    wake();
  }

  function onLeave(e) {
    if (e.relatedTarget === null || e.type === 'blur') {
      clearHover();
      wake();
    }
  }

  function onScroll() {
    // The page moved under a still pointer: find what is there now.
    if (hover && state.inside) {
      state.target = document.elementFromPoint(state.tx, state.ty);
      wake();
    }
  }

  function onResize() {
    state.W = window.innerWidth;
    state.H = window.innerHeight;
  }

  window.addEventListener('pointerdown', onDown, { passive: true });
  window.addEventListener('resize', onResize, { passive: true });
  if (hover) {
    window.addEventListener('pointermove', onMove, { passive: true });
    window.addEventListener('scroll', onScroll, { passive: true });
    document.addEventListener('pointerout', onLeave);
    window.addEventListener('blur', onLeave);
  }
  wake();

  return () => {
    cancelAnimationFrame(state.raf);
    window.removeEventListener('pointerdown', onDown);
    window.removeEventListener('resize', onResize);
    window.removeEventListener('pointermove', onMove);
    window.removeEventListener('scroll', onScroll);
    document.removeEventListener('pointerout', onLeave);
    window.removeEventListener('blur', onLeave);
    clearHover();
    layer.replaceChildren();
  };
}
