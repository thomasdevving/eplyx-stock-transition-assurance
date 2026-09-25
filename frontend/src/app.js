import './access.js';
import { LandingPage } from './landing.js';
import { EvidencePage, attachEvidence } from './evidence-page.js';
import { finishIntro } from './intro.js';
import { attachCoreParallax } from './core-scene.js';
import { attachShell } from './shell.js';
import { attachHeadlineRoll } from './headline-roll.js';
import { initializeMode } from './mode.js';
import { attachAnalysis } from './analysis.js';
initializeMode();
const app = document.querySelector('#app');
let revealObserver, disposeScene, disposeHeadline, disposeAnalysis;
function scrollToHash() {
 let id;
 try { id = decodeURIComponent(location.hash.slice(1)); } catch { return; }
 document.getElementById(id)?.scrollIntoView();
}
function route({ focus = false } = {}) {
 disposeAnalysis?.(); disposeAnalysis=undefined; revealObserver?.disconnect(); disposeScene?.(); disposeHeadline?.(); disposeScene = undefined; disposeHeadline = undefined;
 const evidence = location.pathname.replace(/\/+$/, '') === '/evidence';
 app.innerHTML = evidence ? EvidencePage() : LandingPage({analysis:location.pathname.replace(/\/+$/, '') === '/analysis'});
 document.querySelector('.skip-link').href=evidence&&document.documentElement.dataset.mode==='technical'?'#technical-main':'#main';
 document.title = evidence ? 'Evidence — Eplyx Stock Transition' : location.pathname.startsWith('/analysis') ? 'Analysis — Eplyx Stock Transition' : 'Eplyx — Stock Transition';
 attachShell();
 revealObserver = new IntersectionObserver(entries => entries.forEach(entry => {
  if (entry.isIntersecting) { entry.target.classList.add('is-visible'); revealObserver.unobserve(entry.target); }
 }), { threshold: .08 });
 document.querySelectorAll('.reveal').forEach(el=>revealObserver.observe(el));
 if (evidence) attachEvidence(); else { disposeAnalysis=attachAnalysis(); disposeScene=attachCoreParallax(); disposeHeadline=attachHeadlineRoll(); finishIntro(); }
 if (focus) { const main=document.querySelector(document.documentElement.dataset.mode==='technical'&&evidence?'#technical-main':'#main'); main.tabIndex=-1; main.focus({preventScroll:true}); }
 if (location.hash) requestAnimationFrame(scrollToHash); else window.scrollTo(0,0);
}
document.addEventListener('click', event => {
 const link=event.target.closest('a[data-link]');
 if (!link || event.defaultPrevented || event.button!==0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
 const url=new URL(link.href);
 if (url.origin!==location.origin) return;
 if (url.pathname===location.pathname && url.hash) return;
 event.preventDefault(); history.pushState({},'',url.pathname+url.hash); route({focus:true});
});
addEventListener('popstate',()=>route());
route();
