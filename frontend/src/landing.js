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
  ['01', 'Describe the change', 'Configure your transition program, ratio, rounding rule, fee and reserve in one file.'],
  ['02', 'Read production', 'Eplyx captures current token accounts from Solana mainnet without sending a transaction.'],
  ['03', 'Run it locally', 'The packaged program runs against captured state in an isolated Solana VM. Eplyx checks the resulting balances unit by unit.'],
  ['04', 'Stress and search', 'Eplyx tests selected production accounts. If a case fails, bounded search checks additional accounts and amount or reserve variants.'],
  ['05', 'Decide and share', 'The gate records PASS, WARN or BLOCK for CI. You can replay the report offline and optionally sync it with your team.'],
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
 ['01', 'Exact conversion proof', 'Does the conversion reconcile?', 'Eplyx reports Proven for a candidate conversion only after the packaged program runs in LiteSVM and the source debit, burn, fee, ratio, rounding, reserve and replacement credit reconcile.', 'LiteSVM · exact reconciliation'],
 ['02', 'Stress on real accounts', 'Selected accounts at final capture', 'Eplyx freezes a bounded selection of observed accounts, including large balances and distinct account shapes and balance ranges. Each selected account runs against its own final captured state.', 'Frozen selection · final-state rebinding'],
 ['03', 'Counterexample search', 'Where does a failure begin?', 'For a failed case, bounded search checks additional captured accounts and typed amount or reserve variants. The report records any boundary the search finds.', 'Observed and derived · minimized'],
 ['04', 'Invariants & CI gate', 'A gate outcome with reasons', 'Declare the conditions to check. The chosen gate policy records PASS, WARN or BLOCK with reasons and a CI exit code. It does not change the analytical findings.', 'block-only / strict · exit 0 · 3 · 2'],
 ['05', 'Reproducible evidence', 'Replay from saved artifacts', 'Saved inputs and hashes let you replay the report offline without RPC.', 'Content-addressed · offline replay'],
 ['06', 'Holder checks in the browser', 'What can this account do?', 'Inspect a token or public wallet, then run a local transfer, market exit or candidate conversion check for one selected account. No wallet connection is required.', 'Read-only · VM · no signing'],
];
const trust = [
 ['No mainnet transaction', 'Capture reads Solana state. Candidate execution runs in a local VM; no transaction is sent to mainnet.'],
 ['Every result has a scope', 'Each finding names its exact account, amount and capture. A sample is never presented as the whole population.'],
 ['Issuer authority stays unverified', 'An operator-supplied plan does not establish an issuer-defined transition. OfficialTransition remains NotTested without independent mechanism evidence.'],
 ['Replay the evidence', 'Saved artifacts can be replayed offline and checked against their hashes.'],
];
const installCommand = 'curl -fsSL https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases/latest/download/install.sh | sh';
const run = (label, value, tone = '') => `<div class="run-line${tone ? ` run-line--${tone}` : ''}"><dt>${label}</dt><dd>${value}</dd></div>`;
function exampleRun() {
 const h = showcase.healthy, u = showcase.underfunded;
 return `<section class="section showcase" id="run"><div class="section-heading reveal"><p class="eyebrow eyebrow--dark"><span></span> Saved local tests</p><h2>The reserve changes<br>the result.</h2><p>The same demo conversion program was tested twice against ${number(h.accounts)} real Solana token accounts on ${showcase.date}. Only the replacement reserve changed.</p></div>
 <div class="run-pair reveal">
  <article class="run-window"><div class="window-bar"><span><i></i><i></i><i></i></span><b>$ eplyx preflight · reserve funded</b><em class="run-verdict run-verdict--warn">Pass with warnings</em></div><dl>
   ${run('Production', `${number(h.accounts)} token accounts observed`)}
   ${run('Candidate conversion', 'Proven — every unit reconciled', 'pos')}
   ${run('Stress', `${h.stressProven} / ${h.stressSelected} real accounts passed`, 'pos')}
   ${run('Invariants', `${h.invariantsSatisfied} / ${h.invariantsSatisfied} satisfied`, 'pos')}
   ${run('Counterexample search', `none in ${h.searchExecutions} extra executions`, 'pos')}
  </dl><p class="run-note">All ten selected stress cases passed, but stress and population readiness remain Incomplete, and authority coverage is bounded. The block-only gate passes with warnings.</p></article>
  <article class="run-window run-window--block"><div class="window-bar"><span><i></i><i></i><i></i></span><b>$ eplyx preflight · reserve empty</b><em class="run-verdict run-verdict--block">Blocked</em></div><dl>
   ${run('Production', `${number(u.accounts)} token accounts observed`)}
   ${run('Candidate conversion', 'Failed — reserve too small', 'crit')}
   ${run('Stress', `${u.stressProven} / ${u.stressSelected} real accounts passed`, 'crit')}
   ${run('Invariants', `${u.invariantsViolated} / ${u.invariantsViolated} violated`, 'crit')}
   ${run('Counterexamples', `${u.counterexamples} found · ${u.observed} real accounts, ${u.derived} derived`, 'crit')}
   ${run('Exact boundary', `${number(u.reserveFails)} fails · ${number(u.reservePasses)} passes`, 'crit')}
  </dl><p class="run-note">The saved search found a one-unit reserve boundary for this tested conversion. The result can be replayed offline.</p><code class="run-command">$ eplyx reproduce cx_2ab4735232a3f228da1a29d1</code></article>
 </div>
 <p class="demo-note technical-only">Figures come from the saved runs <code>${h.run}</code> and <code>${u.run}</code> and are verified against them by <code>npm run check:frontend</code>. Both used the registered demo conversion program under an operator-supplied plan: OfficialTransition remains NotTested, and the stress sample does not establish population rollout readiness.</p>
 </section>`;
}
export function LandingPage({ analysis = false } = {}) {
 const p = data.population;
 return `${IntroAnimation()}<main id="main">
 <section class="hero">${Header()}
  <div class="hero__atmosphere" aria-hidden="true"><div class="hero__stars"></div><div class="hero__twinkle">${Array.from({length:16},(_,i)=>`<i style="left:${5+i*4.3}%;top:${12+(i*23)%67}%;--s:${1.4+(i%4)*.4}px;--t:${4+i%4}s;--d:${i*.3}s"></i>`).join('')}</div><div class="hero__mountains"></div></div><div class="hero__wash"></div>
  <div class="hero__copy reveal"><p class="eyebrow"><span></span> Transition assurance for tokenized stocks on Solana</p><h1>Rehearse the<span class="visually-hidden"> transition </span><em class="transition-roll" aria-hidden="true"><span class="transition-roll__track">${[...transitionTerms, transitionTerms[0]].map(term => `<span class="transition-roll__term">${term}</span>`).join('')}</span></em>before it goes live.</h1><p class="hero__lead">Eplyx runs a candidate transition program against current Solana account state in an isolated VM. The report shows which tested conversions reconcile and where they fail.</p><div class="hero__actions"><a href="/analysis#analysis" data-link class="button button--primary">Run analysis <span>↗</span></a><a href="#how" class="button button--text">How it works <span>↓</span></a>${workspaceLink('Team workspace <span>↗</span>', 'button button--text')}</div><p class="hero__scope">Read-only mainnet capture · Isolated VM execution · No transaction ever submitted</p></div>
  <div class="hero__visual reveal">${EplyxCoreScene()}</div><div class="scroll-cue"><span></span> ${analysis ? 'Scroll to run an analysis' : 'Scroll to see how it works'}</div>
 </section>
 ${analysis ? AnalysisSection() : `
 <section class="system section" id="how"><div class="section-heading reveal"><p class="eyebrow eyebrow--dark"><span></span> How it works</p><h2>From your program<br>to a gate result.</h2><p>Eplyx captures current mainnet accounts and tests the packaged candidate program against that state in a local VM.</p></div><div class="system-flow reveal consumer-only">${flow.overview.map(([n,t,b],i)=>`<article class="flow-step ${i===2?'flow-step--engine':''}"><span>${n}</span><h3>${t}</h3><p>${b}</p>${i<4?'<i>›</i>':''}</article>`).join('')}</div><div class="system-flow reveal technical-only">${flow.technical.map(([n,t,b],i)=>`<article class="flow-step ${i===2?'flow-step--engine':''}"><span>${n}</span><h3>${t}</h3><p>${b}</p>${i<4?'<i>›</i>':''}</article>`).join('')}</div></section>
 <section class="proofs section section--ink" id="product"><div class="section-heading section-heading--light reveal"><p class="eyebrow"><span></span> Capabilities</p><h2>What Eplyx<br>checks.</h2><p>Each result identifies the captured state and exact scope it covers.</p></div><div class="proof-grid proof-grid--three reveal">${capabilities.map(([n,t,q,b,s])=>`<article><div><span>${n} / ${t}</span><small class="technical-only">${s}</small></div><h3>${q}</h3><p>${b}</p></article>`).join('')}</div><div class="team-strip reveal"><div><h3>Work as a team</h3><p>The local dashboard shows runs saved on your machine. Optional sync brings local and CI results into a private workspace for history, counterexamples and comparisons. Analysis stays local.</p></div>${workspaceLink('Open the team workspace <span>↗</span>', 'button button--light')}</div></section>
 ${exampleRun()}
 <section class="result section" id="result"><div class="section-heading reveal"><p class="eyebrow eyebrow--dark"><span></span> Holder impact · saved example</p><h2>What the saved example<br>shows for holders.</h2><p>The published SpaceX PreStocks example records which actions passed for selected holdings and which still need evidence. Run an analysis to inspect current data.</p></div>
 <div class="result-window reveal technical-only"><div class="window-bar"><span><i></i><i></i><i></i></span><b>eplyx / stock transition / frozen pre-flight</b><em>Published result</em></div><div class="result-summary"><div><p>SPACEX demo population readiness</p><h3 class="status-incomplete">${data.status}</h3></div><div><p>Frozen population</p><h3>${number(p.token_account_entities)}</h3><span>token-account entities</span></div><div><p>Exact mobility coverage</p><h3>${number(p.entities_with_measured_amount)}</h3><span>full represented-amount entities</span></div><a href="/evidence" data-link>Inspect evidence <span>↗</span></a></div><div class="findings"><article class="finding"><div>${badge('Satisfied')}<span>Selected holder</span></div><h3>Direct mobility</h3><p>Independent transfer and market-exit evidence</p><dl><dt>Exact input</dt><dd>${number(data.direct.amount)} raw SPACEX</dd><dt>Official transition</dt><dd>NotTested</dd></dl></article><article class="finding"><div>${badge('Satisfied')}<span>One LP position</span></div><h3>Principal withdrawal</h3><p>Native Meteora DLMM liquidity removal</p><dl><dt>Range / fraction</dt><dd>−102…−53 / 100%</dd><dt>Fees & position closure</dt><dd>NotTested</dd></dl></article></div></div><p class="demo-note technical-only">${number(p.entities_with_measured_amount)} of ${number(p.positive_balance_entities)} positive accounts have full represented-amount mobility evidence; samples do not prove peers. Results use frozen captures and assumed local signing, with key possession unknown. Readiness follows the demo policy, not an asset safety judgment.</p><div class="consumer-only saved-answer holder-summary"><div class="holder-stats">
  <div><strong>${number(p.token_account_entities)}</strong><span>token accounts captured</span></div>
  <div><strong>${number(p.positive_balance_entities)}</strong><span>hold a balance</span></div>
  <div><strong>${number(p.entities_with_measured_amount)}</strong><span>with full-amount mobility evidence</span></div>
  <div><strong class="status-incomplete">${readableStatus(data.status)}</strong><span>overall verdict</span></div>
 </div>
 <h3>For one real example holding</h3>
 ${plainPaths(data.direct.paths.filter(path => path.status !== 'NotApplicable'))}
 <div class="requirement-list"><article><h3>Withdraw liquidity from one real LP position</h3><span>${readableStatus('Proven')}</span></article></div>
 <p>In saved local tests, the example account transferred and sold tokens. A separate LP position yielded principal. Official conversion remains untested, and most positive-balance accounts lack matching evidence. The report still needs more evidence.</p>
 <a href="/evidence" data-link>Explore the saved findings ↗</a>
 <details class="holder-limits"><summary>What this example does not show</summary>${limitations}</details>
 </div></section>
  <section class="section section--ink trust" id="trust"><div class="section-heading section-heading--light reveal"><p class="eyebrow"><span></span> Evidence boundaries</p><h2>Where the evidence<br>ends.</h2><p>Eplyx records what each run tested and which requirements still need evidence.</p></div><div class="trust-grid reveal">${trust.map(([t,b])=>`<article><h3>${t}</h3><p>${b}</p></article>`).join('')}</div></section>
 <section class="section start" id="start"><div class="start__copy reveal"><p class="eyebrow eyebrow--dark"><span></span> Get started</p><h2>Run your first<br>preflight.</h2><p>Install the binary for macOS, Linux or Windows, then supply your Solana program and a read-only RPC. You do not need an account.</p><div class="start__commands"><code><span>$</span> ${installCommand}</code><code><span>$</span> eplyx init &amp;&amp; eplyx preflight</code><code><span>$</span> eplyx search &amp;&amp; eplyx dashboard</code></div><div class="hero__actions"><a href="https://github.com/thomasdevving/eplyx-stock-transition-assurance#install" class="button button--primary" target="_blank" rel="noopener noreferrer">Install Eplyx <span>↗</span></a><a href="/analysis#analysis" data-link class="button button--text button--dark">Try it in the browser <span>↗</span></a></div></div></section>
 `}
 </main>${Footer()}`;
}
