import { Logo } from './brand.js';

export function Header({ light = false } = {}) {
  return `
    <header class="site-header ${light ? 'site-header--light' : ''}">
      <a href="/" data-link class="logo-link">${Logo()}</a>
      <nav id="primary-navigation" aria-label="Primary navigation">
        <a href="/#how" data-link>How it works</a>
        <a href="/#product" data-link>Capabilities</a>
        <a href="/evidence" data-link class="nav-cta">Explore evidence <span>↗</span></a>
      </nav>
      <button class="menu-button" type="button" aria-label="Open navigation" aria-controls="primary-navigation" aria-expanded="false"><span></span><span></span></button>
    </header>`;
}

export function Footer() {
  return `
    <footer class="footer">
      <div>${Logo()}</div>
      <p>Lifecycle impact intelligence for tokenized stocks.</p>
      <div class="footer__links"><a href="/evidence" data-link>Saved findings</a><a href="/public/evidence/lifecycle-phase-15-production-report.md">Documentation</a></div>
      <small>Stocklana · Read-only inspection · Scoped evidence · No funds moved</small>
    </footer>`;
}

export function attachShell() {
  const button = document.querySelector('.menu-button');
  const nav = document.querySelector('.site-header nav');
  button?.addEventListener('click', () => {
    const open = button.getAttribute('aria-expanded') === 'true';
    button.setAttribute('aria-expanded', String(!open));
    button.setAttribute('aria-label', open ? 'Open navigation' : 'Close navigation');
    nav?.classList.toggle('is-open', !open);
  });
  nav?.querySelectorAll('a').forEach(link => link.addEventListener('click', () => {
    nav.classList.remove('is-open');
    button?.setAttribute('aria-expanded', 'false');
    button?.setAttribute('aria-label', 'Open navigation');
  }));
  button?.addEventListener('keydown', event => {
    if (event.key === 'Escape') {
      nav?.classList.remove('is-open');
      button.setAttribute('aria-expanded', 'false');
      button.setAttribute('aria-label', 'Open navigation');
    }
  });
}
