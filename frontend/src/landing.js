import { IntroAnimation } from './intro.js';
import { EplyxCoreScene } from './core-scene.js';
import { Header, Footer } from './shell.js';
import { workspaceLink } from './workspace.js';
import data from '/public/evidence/summary.js';
import { number, badge } from './format.js';
import { AnalysisSection } from './analysis.js';
import { readableStatus, limitations, plainPaths } from './presentation.js';
import { showcase } from './showcase.js';
const transitionTerms = ['transition', 'stock split', 'merger', 'share-class conversion', 'token migration', 'delisting', 'IPO', 'acquisition'];
const flow = {
 overview: [
  ['01', 'Describe the change', 'Point Eplyx at your transition program and its terms: ratio, rounding, fee and reserve. One config file.'],
  ['02', 'Read production', 'Eplyx reads today’s real token accounts from Solana mainnet. Read-only: it never sends a transaction.'],
  ['03', 'Run it locally', 'Your exact program runs in an isolated Solana VM against that state. Every unit has to add up.'],
  ['04', 'Stress and search', 'Real accounts are tested, and any failure is narrowed down to the exact amount or reserve where it starts.'],
  ['05', 'Decide and share', 'A clear PASS, WARN or BLOCK for your CI, reproducible offline and shareable with your team.'],
 ],
 technical: [
  ['01', 'Package', 'eplyx.toml compiles to a content-addressed package: candidate SBF bytes, terms, execution config and typed invariants.'],
  ['02', 'Capture', 'Bounded read-only mainnet capture of the population and authority accounts, then a coherent final recapture of the exact execution accounts.'],
  ['03', 'Execute', 'The packaged bytes run in LiteSVM. Proven requires exact reconciliation of debit, burn against captured supply, fee, ratio, rounding, reserve and credit.'],
  ['04', 'Stress & search', 'A frozen stress selection is rebound to final state; bounded observed waves and typed amount/reserve variants are minimized to their boundary.'],
  ['05', 'Gate & replay', 'block-only or strict gate over the report (exit 0 / 3 / 2), offline replay of every artifact, local dashboard and optional team sync.'],
 ],
};
const capabilities = [
 ['01', 'Exact conversion proof', 'Does the conversion add up, to the last unit?', 'Eplyx only reports Proven when your real program ran and every balance reconciled exactly: what leaves the holder, what is burned, the fee, the ratio and rounding, the reserve and what the holder receives.', 'LiteSVM · exact reconciliation'],
 ['02', 'Stress on real accounts', 'Does it work for the accounts that actually exist?', 'Ten real production accounts — the largest holders plus every new account shape and balance range — each executed against its exact latest state.', 'Frozen selection · final-state rebinding'],
 ['03', 'Counterexample search', 'Where exactly does it break?', 'When a case fails, Eplyx tests more real accounts and nearby amounts and reserves, then narrows the failure down to its exact boundary, one unit apart.', 'Observed and derived · minimized'],
 ['04', 'Invariants & CI gate', 'Can we ship this release?', 'Declare the rules that must hold. Eplyx answers PASS, WARN or BLOCK with its reasons, and a matching exit code for your CI pipeline.', 'block-only / strict · exit 0 · 3 · 2'],
 ['05', 'Reproducible evidence', 'Can someone else check it?', 'Every result is saved with its exact inputs and hashes. One command replays it offline, with no RPC and no trust in the original machine.', 'Content-addressed · offline replay'],
 ['06', 'Holder checks in the browser', 'What does it mean for a holder?', 'Look up a tokenized stock or a public wallet, then test a transfer, a market exit or a conversion for one account — without connecting a wallet.', 'Read-only · VM · no signing'],
];
const trust = [
 ['Nothing moves', 'Capture is read-only and your program only runs in a local VM. No transaction is ever sent to mainnet.'],
 ['Every result has a scope', 'Each finding names its exact account, amount and capture. A sample is never presented as the whole population.'],
 ['No borrowed authority', 'Eplyx tests your plan, not the issuer’s. An official issuer transition stays “not tested” until its real mechanism is checked.'],
 ['Verify it yourself', 'Results are content-addressed and replay offline, byte for byte, on any machine.'],
];
const installCommand = 'curl -fsSL https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases/latest/download/install.sh | sh';
const run = (label, value, tone = '') => `<div class="run-line${tone ? ` run-line--${tone}` : ''}"><dt>${label}</dt><dd>${value}</dd></div>`;
function exampleRun() {
 const h = showcase.healthy, u = showcase.underfunded;
 return `<section class="section showcase" id="run"><div class="section-heading reveal"><p class="eyebrow eyebrow--dark"><span></span> A real run</p><h2>One program.<br>Two outcomes.</h2><p>The same demo conversion program, tested twice against ${number(h.accounts)} real Solana token accounts on ${showcase.date}. Only the replacement reserve changed.</p></div>
 <div class="run-pair reveal">
  <article class="run-window"><div class="window-bar"><span><i></i><i></i><i></i></span><b>$ eplyx preflight · reserve funded</b><em class="run-verdict run-verdict--warn">Pass with warnings</em></div><dl>
   ${run('Production', `${number(h.accounts)} token accounts observed`)}
   ${run('Candidate conversion', 'Proven — every unit reconciled', 'pos')}
   ${run('Stress', `${h.stressProven} / ${h.stressSelected} real accounts passed`, 'pos')}
   ${run('Invariants', `${h.invariantsSatisfied} / ${h.invariantsSatisfied} satisfied`, 'pos')}
   ${run('Counterexample search', `none in ${h.searchExecutions} extra executions`, 'pos')}
  </dl><p class="run-note">Why a warning? Ten accounts prove the program works for those accounts, not yet for every holder. Eplyx says so instead of guessing.</p></article>
  <article class="run-window run-window--block"><div class="window-bar"><span><i></i><i></i><i></i></span><b>$ eplyx preflight · reserve empty</b><em class="run-verdict run-verdict--block">Blocked</em></div><dl>
   ${run('Production', `${number(u.accounts)} token accounts observed`)}
   ${run('Candidate conversion', 'Failed — reserve too small', 'crit')}
   ${run('Stress', `${u.stressProven} / ${u.stressSelected} real accounts passed`, 'crit')}
   ${run('Invariants', `${u.invariantsViolated} / ${u.invariantsViolated} violated`, 'crit')}
   ${run('Counterexamples', `${u.counterexamples} found · ${u.observed} real accounts, ${u.derived} derived`, 'crit')}
   ${run('Exact boundary', `${number(u.reserveFails)} fails · ${number(u.reservePasses)} passes`, 'crit')}
  </dl><p class="run-note">Caught before release: Eplyx found the exact reserve at which the tested conversion starts to pass, one unit apart, and anyone can replay it offline.</p><code class="run-command">$ eplyx reproduce cx_2ab4735232a3f228da1a29d1</code></article>
 </div>
 <p class="demo-note technical-only">Figures come from the saved runs <code>${h.run}</code> and <code>${u.run}</code> and are verified against them by <code>npm run check:frontend</code>. Both used the registered demo conversion program under an operator-supplied plan: OfficialTransition remains NotTested, and the stress sample does not establish population rollout readiness.</p>
 </section>`;
}
export function LandingPage({ analysis = false } = {}) {
 const p = data.population;
 return `${IntroAnimation()}<main id="main">
 <section class="hero">${Header()}
  <div class="hero__atmosphere" aria-hidden="true"><div class="hero__stars"></div><div class="hero__twinkle">${Array.from({length:16},(_,i)=>`<i style="left:${5+i*4.3}%;top:${12+(i*23)%67}%;--s:${1.4+(i%4)*.4}px;--t:${4+i%4}s;--d:${i*.3}s"></i>`).join('')}</div><div class="hero__mountains"></div></div><div class="hero__wash"></div>
  <div class="hero__copy reveal"><p class="eyebrow"><span></span> Transition assurance for tokenized stocks on Solana</p><h1>Rehearse the<span class="visually-hidden"> transition </span><em class="transition-roll" aria-hidden="true"><span class="transition-roll__track">${[...transitionTerms, transitionTerms[0]].map(term => `<span class="transition-roll__term">${term}</span>`).join('')}</span></em>before it goes live.</h1><p class="hero__lead">Eplyx runs your token-transition program against today’s real Solana accounts in an isolated VM, checks every balance to the last unit, and shows exactly where it breaks — before a single holder is affected.</p><div class="hero__actions"><a href="/analysis#analysis" data-link class="button button--primary">Run analysis <span>↗</span></a><a href="#how" class="button button--text">How it works <span>↓</span></a>${workspaceLink('Team workspace <span>↗</span>', 'button button--text')}</div><p class="hero__scope">Read-only mainnet capture · Isolated VM execution · No transaction ever submitted</p></div>
  <div class="hero__visual reveal">${EplyxCoreScene()}</div><div class="scroll-cue"><span></span> ${analysis ? 'Scroll to run an analysis' : 'Scroll to see how it works'}</div>
 </section>
 ${analysis ? AnalysisSection() : `
 <section class="system section" id="how"><div class="section-heading reveal"><p class="eyebrow eyebrow--dark"><span></span> How it works</p><h2>From your program<br>to a release decision.</h2><p>Devnet cannot show you the accounts your holders really have. Eplyx tests against production itself — without touching it.</p></div><div class="system-flow reveal consumer-only">${flow.overview.map(([n,t,b],i)=>`<article class="flow-step ${i===2?'flow-step--engine':''}"><span>${n}</span><h3>${t}</h3><p>${b}</p>${i<4?'<i>›</i>':''}</article>`).join('')}</div><div class="system-flow reveal technical-only">${flow.technical.map(([n,t,b],i)=>`<article class="flow-step ${i===2?'flow-step--engine':''}"><span>${n}</span><h3>${t}</h3><p>${b}</p>${i<4?'<i>›</i>':''}</article>`).join('')}</div></section>
 <section class="proofs section section--ink" id="product"><div class="section-heading section-heading--light reveal"><p class="eyebrow"><span></span> What you get</p><h2>Proof, not<br>promises.</h2><p>Six answers Eplyx gives before you ship — each one backed by an exact execution you can replay.</p></div><div class="proof-grid proof-grid--three reveal">${capabilities.map(([n,t,q,b,s])=>`<article><div><span>${n} / ${t}</span><small class="technical-only">${s}</small></div><h3>${q}</h3><p>${b}</p></article>`).join('')}</div><div class="team-strip reveal"><div><h3>Work as a team</h3><p>A read-only local dashboard shows every run on your machine. Optionally sync local and CI runs into one private workspace with history, counterexamples and side-by-side comparisons. Analysis always stays local.</p></div>${workspaceLink('Open the team workspace <span>↗</span>', 'button button--light')}</div></section>
 ${exampleRun()}
 <section class="result section" id="result"><div class="section-heading reveal"><p class="eyebrow eyebrow--dark"><span></span> Holder impact · saved example</p><h2>What a lifecycle event<br>means for holders.</h2><p>A published example for SpaceX PreStocks on Solana: which exits were tested for real holdings, what passed, and what still needs evidence. This is a saved result; run an analysis to read current data.</p></div>
 <div class="result-window reveal technical-only"><div class="window-bar"><span><i></i><i></i><i></i></span><b>eplyx / stock transition / frozen pre-flight</b><em>Published result</em></div><div class="result-summary"><div><p>SPACEX demo population readiness</p><h3 class="status-incomplete">${data.status}</h3></div><div><p>Frozen population</p><h3>${number(p.token_account_entities)}</h3><span>token-account entities</span></div><div><p>Exact mobility coverage</p><h3>${number(p.entities_with_measured_amount)}</h3><span>full represented-amount entities</span></div><a href="/evidence" data-link>Inspect evidence <span>↗</span></a></div><div class="findings"><article class="finding"><div>${badge('Satisfied')}<span>Selected holder</span></div><h3>Direct mobility</h3><p>Independent transfer and market-exit evidence</p><dl><dt>Exact input</dt><dd>${number(data.direct.amount)} raw SPACEX</dd><dt>Official transition</dt><dd>NotTested</dd></dl></article><article class="finding"><div>${badge('Satisfied')}<span>One LP position</span></div><h3>Principal withdrawal</h3><p>Native Meteora DLMM liquidity removal</p><dl><dt>Range / fraction</dt><dd>−102…−53 / 100%</dd><dt>Fees & position closure</dt><dd>NotTested</dd></dl></article></div></div><p class="demo-note technical-only">${number(p.entities_with_measured_amount)} of ${number(p.positive_balance_entities)} positive accounts have full represented-amount mobility evidence; samples do not prove peers. Results use frozen captures and assumed local signing, with key possession unknown. Readiness follows the demo policy, not an asset safety judgment.</p><div class="consumer-only saved-answer holder-summary"><div class="holder-stats">
  <div><strong>${number(p.token_account_entities)}</strong><span>token accounts captured</span></div>
  <div><strong>${number(p.positive_balance_entities)}</strong><span>hold a balance</span></div>
  <div><strong>${number(p.entities_with_measured_amount)}</strong><span>with complete exit evidence</span></div>
  <div><strong class="status-incomplete">${readableStatus(data.status)}</strong><span>overall verdict</span></div>
 </div>
 <h3>For one real example holding</h3>
 ${plainPaths(data.direct.paths.filter(path => path.status !== 'NotApplicable'))}
 <div class="requirement-list"><article><h3>Withdraw liquidity from one real LP position</h3><span>${readableStatus('Proven')}</span></article></div>
 <p>Moving and selling the example tokens worked, and liquidity could be withdrawn. The official conversion has not been tested yet, and most holders still need their own evidence — so Eplyx reports “more evidence needed” instead of a green light.</p>
 <a href="/evidence" data-link>Explore the saved findings ↗</a>
 <details class="holder-limits"><summary>What this example does not show</summary>${limitations}</details>
 </div></section>
  <section class="section section--ink trust" id="trust"><div class="section-heading section-heading--light reveal"><p class="eyebrow"><span></span> Built to be believed</p><h2>Honest about<br>what it proves.</h2><p>Assurance is only useful if it never overstates. Eplyx is strict about that by design.</p></div><div class="trust-grid reveal">${trust.map(([t,b])=>`<article><h3>${t}</h3><p>${b}</p></article>`).join('')}</div></section>
 <section class="section start" id="start"><div class="start__copy reveal"><p class="eyebrow eyebrow--dark"><span></span> Get started</p><h2>Run your first preflight<br>in minutes.</h2><p>One binary for macOS, Linux and Windows. No account needed. Bring your Solana program and a read-only RPC.</p><div class="start__commands"><code><span>$</span> ${installCommand}</code><code><span>$</span> eplyx init &amp;&amp; eplyx preflight</code><code><span>$</span> eplyx search &amp;&amp; eplyx dashboard</code></div><div class="hero__actions"><a href="https://github.com/thomasdevving/eplyx-stock-transition-assurance#install" class="button button--primary" target="_blank" rel="noopener noreferrer">Install Eplyx <span>↗</span></a><a href="/analysis#analysis" data-link class="button button--text button--dark">Try it in the browser <span>↗</span></a></div></div></section>
 `}
 </main>${Footer()}`;
}
