import { readdir } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { prepareEvidence } from './evidence.mjs';
for (const directory of ['src', 'dashboard']) for (const file of await readdir(new URL(`./${directory}/`, import.meta.url))) {
 if (!file.endsWith('.js')) continue;
 const result=spawnSync(process.execPath,['--check',fileURLToPath(new URL(`./${directory}/${file}`,import.meta.url))],{stdio:'inherit'});
 if (result.status!==0) process.exit(result.status || 1);
}
await prepareEvidence();
await verifyShowcase();
console.log('Frontend JavaScript syntax, published report digests and showcase figures verified.');

// Every number the landing page quotes from the example runs must match the
// saved engine artifacts exactly.
async function verifyShowcase() {
 const { readFile } = await import('node:fs/promises');
 const { showcase } = await import('./src/showcase.js');
 const run = async (id, member) => JSON.parse(await readFile(new URL(`../fixtures/dashboard/transition-acceptance/.eplyx/runs/${id}/${member}`, import.meta.url)));
 const fail = message => { console.error(`Showcase figure mismatch: ${message}`); process.exit(1); };
 for (const [name, figures] of Object.entries({ healthy: showcase.healthy, underfunded: showcase.underfunded })) {
  const report = await run(figures.run, 'result/report.json');
  const search = await run(figures.run, 'search/counterexamples.json');
  const counts = report.population_summary.counts;
  const check = (label, actual, expected) => { if (expected !== undefined && actual !== expected) fail(`${name} ${label}: page says ${expected}, run says ${actual}`); };
  check('accounts', counts.token_accounts_observed, figures.accounts);
  check('positive balances', counts.positive_balance_accounts_observed, figures.positive);
  check('conversion', report.conversion_result.status, figures.conversion);
  check('stress proven', report.selected_stress_counts.Proven ?? 0, figures.stressProven);
  check('stress selected', report.exact_selected_cases.length, figures.stressSelected);
  check('invariants satisfied', report.invariants.filter(i => i.status === 'Satisfied').length, figures.invariantsSatisfied);
  check('invariants violated', report.invariants.filter(i => i.status === 'Violated').length, figures.invariantsViolated);
  check('gate', report.gate_outcome, figures.gate);
  check('search executions', search.budget.observed_executions, figures.searchExecutions);
  check('counterexamples', search.counterexamples.length, figures.counterexamples);
  check('observed', search.counterexamples.filter(c => c.observed_state_digest && !c.search_dimension).length, figures.observed);
  check('derived', search.counterexamples.filter(c => c.search_dimension).length, figures.derived);
  const reserve = search.counterexamples.find(c => c.search_dimension === 'ProposedReserve');
  check('reserve failing value', reserve?.derived_value_raw, figures.reserveFails);
  check('reserve passing value', reserve?.first_passing_value_raw, figures.reservePasses);
 }
}
