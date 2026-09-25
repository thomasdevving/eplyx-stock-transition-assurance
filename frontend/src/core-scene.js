import { presentationMode, setPresentationMode } from './mode.js';
import { Mark } from './brand.js';

// Two orbital planes around the mark: the changes Eplyx tracks, and the
// consequences it measures. Each ring keeps its own inclination and travel
// direction, so the toggle reads as a change of axis rather than a swap of
// labels. Depth is the ring angle alone — the far half passes behind the mark.
const rings = {
  changes: {
    label: 'Overview', caption: 'Production state · Candidate execution · Measured result',
    tilt: -17, direction: -1, duration: 38,
    bodies: [
      ['Production state', 'live', 'Eplyx reads current token accounts on Solana mainnet. Capture is read-only.'],
      ['Candidate execution', 'live', 'The packaged candidate program runs locally in an isolated VM against captured accounts. No mainnet transaction is sent.'],
      ['Measured result', 'live', 'Eplyx reconciles every required token balance change to the last unit. It records any failure boundary found by bounded search, and saved reports replay offline.'],
    ],
  },
  consequences: {
    label: 'Technical', caption: 'Preflight CLI · Browser VM checks · Evidence & workspace',
    tilt: 19, direction: 1, duration: 34,
    bodies: [
      ['Preflight CLI', 'live', 'Package the candidate and capture production state. The CLI executes in LiteSVM, checks exact reconciliation, stresses selected accounts, searches for counterexamples and evaluates typed invariants.'],
      ['Browser VM checks', 'live', 'For one focused wallet account, capture fresh state and run a transfer, one supported DLMM exit or the registered demo conversion in the analysis service’s VM. No transaction is submitted.'],
      ['Evidence & workspace', 'live', 'Saved artifacts can be hash-checked and replayed offline. The local dashboard reads them; optional sync copies results to a team workspace. The workspace does not run analysis. OfficialTransition remains NotTested, and an operator plan does not establish an issuer mechanism.', 'SPACEX demo population readiness', 'Incomplete'],
    ],
  },
};

// Each stone is drawn twice: one copy always sits under the mark and one above
// it, faded in with depth. Swapping a single element's z-index made a stone
// jump in front of the mark in one frame wherever it overlapped the edge.
const orbitRock = (ring, index, layer) =>
  `<span class="orbit-rock orbit-rock--${layer}" data-ring="${ring}" data-index="${index}" aria-hidden="true"><span class="orbit-body__rock orbit-body__rock--${index}"></span></span>`;

const orbitBody = (ring, [name, status, detail, metric, value], index) =>
  `${orbitRock(ring, index, 'back')}${orbitRock(ring, index, 'front')}
  <button type="button" class="orbit-body orbit-body--${status}" data-ring="${ring}" data-index="${index}" data-name="${name}" data-status="${status === 'live' ? 'Available' : 'Planned'}" data-detail="${detail}"${metric ? ` data-metric="${metric}" data-value="${value}"` : ''}>
    <span class="visually-hidden">${name}. ${status === 'live' ? 'Available capability' : 'Planned layer'}. ${detail}</span>
  </button>
  <div class="orbit-label orbit-label--${status}" data-ring="${ring}" data-index="${index}" aria-hidden="true">
    <svg class="orbit-label__leader"><path/><circle r="2"/></svg>
    <span class="orbit-body__name">${name}</span>
    <small class="orbit-body__status">${status === 'live' ? 'Available' : 'Planned'}</small>
  </div>`;

// The front half of the ring is cut around every stone, so the line reads as
// passing into the stone rather than being painted over it.
const orbitPlane = side => `<svg class="orbit-plane orbit-plane--${side}" aria-hidden="true">
  ${side === 'front' ? `<defs>
    <linearGradient id="orbit-main"><stop offset="0" stop-color="#8cbcf0"/><stop offset=".38" stop-color="#f6faff"/><stop offset=".72" stop-color="#b0d4fb"/><stop offset="1" stop-color="#468fdd"/></linearGradient>
    <linearGradient id="orbit-sheen"><stop offset="0" stop-color="#fafcff" stop-opacity=".1"/><stop offset=".45" stop-color="#fafcff" stop-opacity=".9"/><stop offset="1" stop-color="#dfeeff" stop-opacity=".15"/></linearGradient>
    <filter id="orbit-bloom" x="-25%" y="-70%" width="150%" height="240%"><feGaussianBlur stdDeviation="5"/></filter>
    <radialGradient id="orbit-cut-fade"><stop offset=".78" stop-color="#000"/><stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>
    <mask id="orbit-cut" class="orbit-cut" maskUnits="userSpaceOnUse"><rect class="orbit-cut__field" fill="#fff"/>${Object.entries(rings).map(([ring, { bodies }]) => bodies.map((_, index) => `<circle class="orbit-cut__hole" data-ring="${ring}" data-index="${index}" r="0" fill="url(#orbit-cut-fade)"/>`).join('')).join('')}</mask>
  </defs>
  <g mask="url(#orbit-cut)"><path class="orbit-line orbit-line--bloom"/><path class="orbit-line orbit-line--main"/><path class="orbit-line orbit-line--sheen"/><path class="orbit-pulse" pathLength="1"/></g>` : '<path class="orbit-line orbit-line--main"/>'}
</svg>`;

export function EplyxCoreScene() {
  const planes = Object.entries(rings);
  return `<div class="core-scene" data-ring="changes" role="group" aria-label="Eplyx production-state capture, candidate execution and measured-result capabilities. Explore each item for its evidence scope.">
    <div class="core-stage">
      ${orbitPlane('back')}
      <div class="core-glow" aria-hidden="true"></div>
      <div class="logo-core" aria-hidden="true"><div class="sculpture-fallback">${Mark({ className: 'core-mark core-mark--depth' })}${Mark({ className: 'core-mark core-mark--bevel' })}${Mark({ className: 'core-mark core-mark--face' })}</div><div class="sculpture-mount"></div></div>
      ${orbitPlane('front')}
      ${planes.map(([name, ring]) => ring.bodies.map((body, index) => orbitBody(name, body, index)).join('')).join('')}
      <div class="orbit-detail" aria-hidden="true">
        <div class="orbit-detail__head"><strong class="orbit-detail__name"></strong><em class="orbit-detail__status"></em></div>
        <p class="orbit-detail__text"></p>
        <div class="orbit-detail__metric"><span></span><b></b></div>
      </div>
    </div>
    <div class="orbit-switch">
      <i class="orbit-switch__thumb" aria-hidden="true"></i>
      ${planes.map(([key, ring], index) => `<button type="button" class="orbit-switch__option" data-target="${key}" aria-pressed="${index === 0}">${ring.label}</button>`).join('')}
    </div>
    <p class="orbit-caption">${rings.changes.caption}</p>
  </div>`;
}

export function attachCoreParallax() {
  const scene = document.querySelector('.core-scene');
  if (!scene) return () => {};
  const stage = scene.querySelector('.core-stage');
  const detail = scene.querySelector('.orbit-detail');
  const caption = scene.querySelector('.orbit-caption');
  const orbitPlanes = [...scene.querySelectorAll('.orbit-plane')];
  const fallback = scene.querySelector('.sculpture-fallback');
  const cut = scene.querySelector('.orbit-cut');
  // `pointer` is the raw target shared with the sculpture; `drift` eases toward
  // it each frame so ring, stones, labels and mark move as one layer.
  const pointer = { x: 0, y: 0 };
  const drift = { x: 0, y: 0 };
  const bodies = [...scene.querySelectorAll('.orbit-body')];
  // Labels stay above the mark even when their rock travels behind it.
  const labels = new Map(bodies.map(body => [body, {
    element: scene.querySelector(`.orbit-label[data-ring="${body.dataset.ring}"][data-index="${body.dataset.index}"]`),
    hole: scene.querySelector(`.orbit-cut__hole[data-ring="${body.dataset.ring}"][data-index="${body.dataset.index}"]`),
    rocks: [...scene.querySelectorAll(`.orbit-rock[data-ring="${body.dataset.ring}"][data-index="${body.dataset.index}"]`)],
    width: 0,
    height: 0,
  }]));
  const motion = matchMedia('(prefers-reduced-motion: reduce)');
  // The inclination is tweened on a switch, so line and bodies never disagree.
  const view = { tilt: rings.changes.tilt, from: rings.changes.tilt, to: rings.changes.tilt, swing: 1 };
  const listeners = [];
  let active = 'changes';
  let width = 0, height = 0, bodyWidth = 126, spanX = 0, spanY = 0;
  let frame = 0, last = 0, elapsed = 0, held = null, onscreen = true;

  const point = (angle, spread = 1) => {
    const radians = view.tilt * Math.PI / 180;
    const x = Math.cos(angle) * spanX * spread;
    const y = Math.sin(angle) * spanY * spread;
    return {
      x: width / 2 + x * Math.cos(radians) - y * Math.sin(radians),
      y: height / 2 + x * Math.sin(radians) + y * Math.cos(radians),
      depth: Math.sin(angle),
    };
  };

  const arc = (from, to, spread) => {
    let d = '';
    for (let step = 0; step <= 44; step++) {
      const { x, y } = point(from + (to - from) * step / 44, spread);
      d += `${step ? 'L' : 'M'}${x.toFixed(1)} ${y.toFixed(1)}`;
    }
    return d;
  };

  const drawPlanes = () => {
    if (!width || !height) return;
    for (const [side, from, to] of [['back', Math.PI, Math.PI * 2], ['front', 0, Math.PI]]) {
      const plane = scene.querySelector(`.orbit-plane--${side}`);
      plane.setAttribute('viewBox', `0 0 ${width} ${height}`);
      const main = arc(from, to, 1);
      plane.querySelectorAll('.orbit-line--main, .orbit-line--sheen, .orbit-line--bloom, .orbit-pulse').forEach(path => path.setAttribute('d', main));
    }
    for (const element of [cut, cut.querySelector('rect')]) {
      element.setAttribute('x', -80);
      element.setAttribute('y', -80);
      element.setAttribute('width', width + 160);
      element.setAttribute('height', height + 160);
    }
  };

  // The panel aligns with the body's outward edge so it opens away from the
  // mark, and only flips back when that side has no room left in the stage.
  const anchor = (x, y) => {
    const half = (detail.offsetWidth || 228) / 2;
    const depth = detail.offsetHeight || 150;
    const outward = x >= width / 2 ? 1 : -1;
    const span = Math.max(width - half - 2, half + 2);
    const left = Math.min(Math.max(x + outward * (bodyWidth / 2 - half), half + 2), span);
    const gap = bodyWidth / 2 + 12;
    const below = y < height * .5 ? y + gap + depth < height : y - gap - depth < 0;
    detail.style.transform = `translate(${left.toFixed(1)}px, ${(y + (below ? gap : -gap)).toFixed(1)}px) translate(-50%, ${below ? '0' : '-100%'})`;
  };

  const place = () => {
    if (!width || !height) return;
    const ring = rings[active];
    const turn = elapsed / ring.duration * Math.PI * 2 * ring.direction;
    const shiftX = drift.x * 13, shiftY = drift.y * 9;
    for (const plane of orbitPlanes) plane.style.transform = `translate(${shiftX.toFixed(1)}px, ${shiftY.toFixed(1)}px)`;
    if (fallback) fallback.style.transform = `perspective(900px) rotateY(${(drift.x * 9).toFixed(2)}deg) rotateX(${(-drift.y * 6).toFixed(2)}deg) rotate(-2deg)`;
    for (const body of bodies) {
      const { hole } = labels.get(body);
      if (body.dataset.ring !== active) { hole.setAttribute('r', 0); continue; }
      const index = Number(body.dataset.index);
      const angle = turn - 2.25 + index / ring.bodies.length * Math.PI * 2;
      const orbit = point(angle);
      const { depth } = orbit;
      const near = (depth + 1) / 2;
      const scale = .78 + near * .22;
      // The cut lives inside the shifted plane, so it uses the unshifted point.
      hole.setAttribute('cx', orbit.x.toFixed(1));
      hole.setAttribute('cy', orbit.y.toFixed(1));
      hole.setAttribute('r', (bodyWidth * scale * .56).toFixed(1));
      const x = orbit.x + shiftX, y = orbit.y + shiftY;
      const transform = `translate(${x.toFixed(1)}px, ${y.toFixed(1)}px) translate(-50%, -50%) scale(${scale.toFixed(3)})`;
      body.style.transform = transform;
      // The hit target still switches sides at once; only the visuals blend.
      body.style.zIndex = depth > 0 ? 6 : 2;
      const label = labels.get(body);
      const rise = Math.min(1, Math.max(0, (depth + .2) / .4));
      const front = rise * rise * (3 - 2 * rise);
      const spin = `${(Math.sin(elapsed * .12 + index * 1.7) * 9).toFixed(2)}deg`;
      for (const rock of label.rocks) {
        rock.style.transform = transform;
        rock.style.setProperty('--near', near.toFixed(3));
        rock.style.setProperty('--rock-turn', spin);
        if (rock.classList.contains('orbit-rock--front')) rock.style.setProperty('--front', front.toFixed(3));
      }
      const outwardX = (x - width / 2) / spanX;
      const outwardY = (y - height / 2) / spanY;
      const radius = bodyWidth * scale * .4;
      const wantedX = x + outwardX * (radius + label.width / 2 + 12);
      const labelX = Math.min(width - label.width / 2 - 8, Math.max(label.width / 2 + 8, wantedX));
      // Blend continuously into an above/below placement at the edges.
      // Hard edge and direction thresholds made the text jump mid-orbit.
      const edge = Math.min(1, Math.abs(wantedX - labelX) / 36);
      const blend = edge * edge * (3 - 2 * edge);
      const vertical = outwardY + (Math.tanh((outwardY - .15) * 4) - outwardY) * blend;
      const wantedY = y + vertical * (radius + label.height / 2 + 14);
      const labelY = Math.min(height - label.height / 2 - 8, Math.max(label.height / 2 + 8, wantedY));
      label.element.style.transform = `translate(${labelX.toFixed(1)}px, ${labelY.toFixed(1)}px) translate(-50%, -50%)`;
      label.element.style.setProperty('--near', near.toFixed(3));
      const reach = Math.hypot(labelX - x, labelY - y) || 1;
      const dotX = x - labelX + (labelX - x) / reach * radius * .7;
      const dotY = y - labelY + (labelY - y) / reach * radius * .7;
      // Follow the nearest edge of the label, without flipping the leader
      // abruptly between its top and bottom while the label moves past it.
      const endScale = 1 / Math.max(Math.abs(x - labelX) / (label.width / 2 + 4), Math.abs(y - labelY) / (label.height / 2 + 4), 1);
      const endX = (x - labelX) * endScale;
      const endY = (y - labelY) * endScale;
      label.element.querySelector('path').setAttribute('d', `M${dotX.toFixed(1)} ${dotY.toFixed(1)} L${endX.toFixed(1)} ${endY.toFixed(1)}`);
      const dot = label.element.querySelector('circle');
      dot.setAttribute('cx', dotX.toFixed(1));
      dot.setAttribute('cy', dotY.toFixed(1));
      if (body === held) anchor(x, y);
    }
    scene.classList.add('is-placed');
  };

  const step = now => {
    frame = 0;
    const delta = last ? Math.min((now - last) / 1000, .05) : 0;
    last = now;
    if (view.swing < 1) {
      view.swing = Math.min(1, view.swing + delta / .28);
      const eased = 1 - (1 - view.swing) ** 3;
      view.tilt = view.from + (view.to - view.from) * eased;
      drawPlanes();
    }
    if (!held) elapsed += delta;
    const follow = 1 - Math.exp(-delta * 6);
    drift.x += (pointer.x - drift.x) * follow;
    drift.y += (pointer.y - drift.y) * follow;
    place();
    if (onscreen && !document.hidden && !motion.matches) frame = requestAnimationFrame(step);
  };

  const start = () => {
    if (frame || motion.matches || !onscreen || document.hidden) return;
    last = 0;
    frame = requestAnimationFrame(step);
  };
  const stop = () => {
    cancelAnimationFrame(frame);
    frame = 0;
  };

  const measure = () => {
    const rect = stage.getBoundingClientRect();
    if (!rect.width || !rect.height) return;
    width = rect.width;
    height = rect.height;
    bodyWidth = parseFloat(getComputedStyle(scene).getPropertyValue('--orbit-body')) || bodyWidth;
    for (const label of labels.values()) {
      label.width = label.element.offsetWidth;
      label.height = label.element.offsetHeight;
    }
    spanX = Math.min(width * .39, width / 2 - bodyWidth / 2 - 2);
    spanY = height * .29;
    drawPlanes();
    place();
  };

  const show = body => {
    held = body;
    detail.querySelector('.orbit-detail__name').textContent = body.dataset.name;
    detail.querySelector('.orbit-detail__status').textContent = body.dataset.status;
    detail.querySelector('.orbit-detail__text').textContent = body.dataset.detail;
    const metric = detail.querySelector('.orbit-detail__metric');
    metric.style.display = body.dataset.metric ? '' : 'none';
    if (body.dataset.metric) {
      metric.querySelector('span').textContent = body.dataset.metric;
      metric.querySelector('b').textContent = body.dataset.value;
    }
    detail.dataset.status = body.dataset.status;
    detail.classList.add('is-visible');
    body.classList.add('orbit-body--held');
    labels.get(body).rocks.forEach(rock => rock.classList.add('orbit-rock--held'));
    place();
  };

  const hide = () => {
    held?.classList.remove('orbit-body--held');
    if (held) labels.get(held).rocks.forEach(rock => rock.classList.remove('orbit-rock--held'));
    held = null;
    detail.classList.remove('is-visible');
  };

  const select = next => {
    if (next === active || !rings[next]) return;
    active = next;
    scene.dataset.ring = next;
    scene.querySelectorAll('.orbit-switch__option').forEach(option => option.setAttribute('aria-pressed', String(option.dataset.target === next)));
    caption.textContent = rings[next].caption;
    hide();
    view.from = view.tilt;
    view.to = rings[next].tilt;
    view.swing = motion.matches ? 1 : 0;
    if (motion.matches) view.tilt = view.to;
    drawPlanes();
    place();
    start();
  };

  const bind = (target, event, handler) => {
    target.addEventListener(event, handler);
    listeners.push(() => target.removeEventListener(event, handler));
  };

  for (const body of bodies) {
    bind(body, 'pointerenter', event => { if (event.pointerType !== 'touch') show(body); });
    bind(body, 'focus', () => { if (body.matches(':focus-visible')) show(body); });
    bind(body, 'pointerleave', event => { if (event.pointerType !== 'touch') hide(); });
    bind(body, 'blur', hide);
    bind(body, 'click', () => (held === body ? hide() : show(body)));
    bind(body, 'keydown', event => {
      if (event.key === 'Escape') hide();
    });
  }
  for (const option of scene.querySelectorAll('.orbit-switch__option')) {
    bind(option, 'click', () => {select(option.dataset.target);setPresentationMode(option.dataset.target==='consequences'?'technical':'overview');});
  }

  bind(document, 'eplyx-mode', event=>{select(event.detail==='technical'?'consequences':'changes');measure();});
  select(presentationMode()==='technical'?'consequences':'changes');

  // Only a pointer over the mark steers it; anywhere else it eases back.
  const mark = scene.querySelector('.logo-core');
  const leave = () => {
    pointer.x = 0;
    pointer.y = 0;
    if (motion.matches) { drift.x = 0; drift.y = 0; place(); }
  };
  const move = ({ clientX, clientY, pointerType }) => {
    if (motion.matches || pointerType === 'touch') return;
    const rect = mark.getBoundingClientRect();
    const x = (clientX - rect.left) / rect.width - .5;
    const y = (clientY - rect.top) / rect.height - .5;
    if (!rect.width || Math.abs(x) > .5 || Math.abs(y) > .5) return leave();
    pointer.x = x;
    pointer.y = y;
    start();
  };
  bind(scene, 'pointermove', move);
  bind(scene, 'pointerleave', leave);

  const onMotion = () => {
    stop();
    if (motion.matches) {
      leave();
      view.swing = 1;
      view.tilt = view.to;
      drawPlanes();
      place();
    } else start();
  };
  const onVisibility = () => (document.hidden ? stop() : start());
  motion.addEventListener('change', onMotion);
  document.addEventListener('visibilitychange', onVisibility);

  const sizeObserver = new ResizeObserver(measure);
  sizeObserver.observe(stage);
  const screenObserver = new IntersectionObserver(([entry]) => {
    onscreen = entry.isIntersecting;
    if (onscreen) start(); else stop();
  });
  screenObserver.observe(stage);

  measure();
  document.fonts?.ready.then(measure);
  start();

  let disposed = false;
  let disposeSculpture;
  import('./sculpture.js').then(module => {
    if (!disposed) disposeSculpture = module.mountSculpture(scene.querySelector('.sculpture-mount'), pointer);
  }).catch(() => { /* The original vector remains visible when WebGL is unavailable. */ });

  return () => {
    disposed = true;
    stop();
    sizeObserver.disconnect();
    screenObserver.disconnect();
    motion.removeEventListener('change', onMotion);
    document.removeEventListener('visibilitychange', onVisibility);
    listeners.forEach(dispose => dispose());
    disposeSculpture?.();
  };
}
