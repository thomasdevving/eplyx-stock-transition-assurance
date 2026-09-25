// Figures from two real Eplyx preflight runs (24 Sep 2026) of the registered
// demo conversion program against Solana mainnet state, read-only. The same
// candidate ran twice: once with a funded replacement reserve, once with none.
// `npm run check:frontend` verifies every number against the saved runs in
// fixtures/dashboard/transition-acceptance, so this copy cannot drift.
export const showcase = {
 date: '24 Sep 2026',
 healthy: {
  run: 'run_20260924122823725_ce7b4d55310c',
  accounts: 17838,
  positive: 10034,
  conversion: 'Proven',
  stressProven: 10,
  stressSelected: 10,
  invariantsSatisfied: 2,
  searchExecutions: 25,
  counterexamples: 0,
  gate: 'Warn',
 },
 underfunded: {
  run: 'run_20260924123736483_ce7b4d55310c',
  accounts: 17838,
  conversion: 'Failed',
  stressProven: 0,
  stressSelected: 10,
  invariantsViolated: 2,
  counterexamples: 37,
  observed: 35,
  derived: 2,
  reserveFails: '747775404621',
  reservePasses: '747775404622',
  gate: 'Block',
 },
};
