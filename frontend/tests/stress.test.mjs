import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, rm, writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {randomUUID, createHash} from 'node:crypto';
import {Readable} from 'node:stream';
import {AnalysisService, handleAnalysisAPI} from '../analysis-service.mjs';
import {verifyStressResult} from '../stress-service.mjs';

const root = resolve('.');
const hash = b => createHash('sha256').update(b).digest('hex');
const digest = n => String(n).repeat(64).slice(0, 64);
const MINT = 'PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh';

const job = () => ({
 id: 'stress-id', wallet_run_id: 'wallet-run', selection: {mint: MINT},
 population_capture_sha256: digest(1), stress_plan_sha256: digest(2),
 cases_capture_sha256: digest(3), parent_plan_sha256: digest(4),
});
const parent = () => ({id: 'conversion-id', selection: {mint: MINT}});
const mechanism = () => ({program_sha256: digest(5)});

const selectedCase = (n, amount) => ({
 case_id: `case-0${n}`, entity_id: `current-stress:${digest(1)}:account-${n}`,
 token_account: `account-${n}`, selected_amount_raw: amount,
 case_plan_sha256: digest(6 + n), state_shape_sha256: digest(9),
});
const caseResult = (n, amount, status) => ({
 ...selectedCase(n, amount), candidate_program_sha256: digest(5), status,
 execution_performed: status !== 'Indeterminate', local_execution_performed: status !== 'Indeterminate',
 signer_possession_known: false, issuer_binding_established: false,
 official_transition: 'NotTested', funds_moved: false,
});

/** A structurally valid result: two exact cases proven out of 10 000 observed. */
function result(overrides = {}) {
 const cases = [selectedCase(1, '100'), selectedCase(2, '250')];
 const results = [caseResult(1, '100', 'Proven'), caseResult(2, '250', 'Proven')];
 return {
  kind: 'current-conversion-stress', stress_id: 'stress-id', run_id: 'wallet-run', asset_mint: MINT,
  population_capture: {capture_sha256: digest(1), historical_population_used: false},
  candidate_plan_sha256: digest(4),
  candidate_mechanism: {program_sha256: digest(5), deployed_on_mainnet: false, issuer_mechanism: false},
  selection_plan: {stress_plan_sha256: digest(2), frozen_before_execution: true},
  selected_cases: cases, results,
  coverage_summary: {
   exact_accounts_selected: 2, exact_accounts_executed: 2, exact_accounts_proven: 2,
   positive_balance_accounts_observed: 10000, accounts_observed: 12000,
  },
  shape_coverage: [{state_shape_sha256: digest(9), entities_in_shape: 9000, entities_selected: 2,
   entities_executed: 2, entities_untested: 8998,
   executed_entity_ids: [cases[0].entity_id, cases[1].entity_id]}],
  readiness: {scope: 'ConversionStressReadiness', status: 'Incomplete', population_readiness: null},
  population_rollout_readiness: {scope: 'PopulationRolloutReadiness', status: 'Incomplete'},
  official_transition: 'NotTested', funds_moved: false, authorization: false,
  ...overrides,
 };
}
const check = (mutate, label) => {
 const value = result();
 mutate(value);
 assert.throws(() => verifyStressResult(value, job(), parent(), mechanism(), 0), /InvalidEngineResult/, label);
};

test('a structurally sound bounded stress result is accepted', () => {
 verifyStressResult(result(), job(), parent(), mechanism(), 0);
});

test('resolved non-wallet control never grants a signer or conversion proof', () => {
 const scopedJob={...job(),authority_plan_sha256:digest('a')};
 const controlCase={signer_assumed_locally:false,execution_supported:false,conversion:'Unsupported'};
 const value=result({
  non_standard_account_control:{plan_sha256:scopedJob.authority_plan_sha256,
   population_digest:scopedJob.population_capture_sha256,
   coverage:{cases_selected:1},cases:[controlCase]},
  refined_stress_world_sha256:hash(JSON.stringify([
   scopedJob.population_capture_sha256,scopedJob.authority_plan_sha256,scopedJob.stress_plan_sha256])),
 });
 verifyStressResult(value,scopedJob,parent(),mechanism(),0);
 for(const change of [
  v=>{v.non_standard_account_control.cases[0].signer_assumed_locally=true;},
  v=>{v.non_standard_account_control.cases[0].execution_supported=true;},
  v=>{v.non_standard_account_control.cases[0].conversion='Proven';},
  v=>{v.non_standard_account_control.population_digest=digest('b');},
  v=>{v.refined_stress_world_sha256=digest('c');},
 ]){
  const mutated=structuredClone(value);
  change(mutated);
  assert.throws(()=>verifyStressResult(mutated,scopedJob,parent(),mechanism(),0),/InvalidEngineResult/);
 }
});

test('a bounded sample can never be reported as population readiness', () => {
 // Two exact accounts proven out of ten thousand observed cannot make the
 // separate population scope Ready.
 check(v => {v.population_rollout_readiness.status = 'Ready';}, 'sample cannot make population Ready');
 // It may become Ready only when every positive-balance account is proven.
 const exhaustive = result();
 exhaustive.coverage_summary.positive_balance_accounts_observed = 2;
 exhaustive.population_rollout_readiness.status = 'Ready';
 verifyStressResult(exhaustive, job(), parent(), mechanism(), 0);
 // The stress scope never carries a population readiness value of its own.
 check(v => {v.readiness.population_readiness = {status: 'Ready'};}, 'stress scope must not publish population readiness');
 check(v => {v.readiness.scope = 'PopulationRolloutReadiness';}, 'scopes must stay distinct');
});

test('proven counts can never exceed what was actually executed or observed', () => {
 check(v => {v.coverage_summary.exact_accounts_proven = 5;}, 'inflated proven count');
 check(v => {
  v.coverage_summary.positive_balance_accounts_observed = 1;
 }, 'more proven than observed positive accounts');
 check(v => {v.results[0].status = 'Proven'; v.results[0].execution_performed = false;}, 'proven without execution');
 check(v => {v.results[0].local_execution_performed = false;}, 'proven without local execution');
});

test('state-shape coverage is never reported as entity coverage', () => {
 check(v => {v.shape_coverage[0].entities_executed = 9000; v.shape_coverage[0].entities_untested = 0;},
  'shape members are not executed entities');
 check(v => {v.shape_coverage[0].entities_selected = 9000;}, 'more selected than there are exact cases');
 check(v => {v.shape_coverage[0].executed_entity_ids = [];}, 'executed ids must match the count');
 check(v => {v.shape_coverage[0].entities_untested = 0;}, 'untested members cannot be written away');
});

test('every result stays bound to its exact frozen case', () => {
 check(v => {v.results[1].token_account = 'account-1';}, 're-pointed account');
 check(v => {v.results[1].selected_amount_raw = '1';}, 'lowered amount');
 check(v => {v.results[1].case_plan_sha256 = digest(1);}, 'swapped case plan');
 check(v => {v.results.pop();}, 'dropped result');
 check(v => {v.results[0].case_id = 'case-99';}, 'renamed case');
 check(v => {v.coverage_summary.exact_accounts_selected = 1;}, 'selection count disagreement');
});

test('no stress case may claim issuer binding, key possession, movement or an official transition', () => {
 check(v => {v.results[0].official_transition = 'Proven';}, 'official transition');
 check(v => {v.results[0].issuer_binding_established = true;}, 'issuer binding');
 check(v => {v.results[0].signer_possession_known = true;}, 'key possession');
 check(v => {v.results[0].funds_moved = true;}, 'funds moved');
 check(v => {v.official_transition = 'Proven';}, 'run-level official transition');
 check(v => {v.funds_moved = true;}, 'run-level funds moved');
 check(v => {v.authorization = true;}, 'authorization');
 check(v => {v.candidate_mechanism.deployed_on_mainnet = true;}, 'claimed deployment');
 check(v => {v.candidate_mechanism.issuer_mechanism = true;}, 'claimed issuer mechanism');
});

test('a result must be bound to its population, frozen plan and candidate program', () => {
 check(v => {v.population_capture.capture_sha256 = digest(7);}, 'other population');
 check(v => {v.selection_plan.stress_plan_sha256 = digest(7);}, 'other plan');
 check(v => {v.candidate_plan_sha256 = digest(7);}, 'other candidate plan');
 check(v => {v.candidate_mechanism.program_sha256 = digest(7);}, 'other program build');
 check(v => {v.results[0].candidate_program_sha256 = digest(7);}, 'other per-case program build');
 check(v => {v.selection_plan.frozen_before_execution = false;}, 'plan not frozen first');
 check(v => {v.stress_id = 'another';}, 'other stress run');
 check(v => {v.run_id = 'another';}, 'other wallet run');
 check(v => {v.asset_mint = 'SoLmint1111111111111111111111111111111111111';}, 'other asset');
});

test('a historical population can never back a current stress result', () => {
 check(v => {v.population_capture.historical_population_used = true;}, 'historical population');
 check(v => {delete v.population_capture.historical_population_used;}, 'unstated population provenance');
});

test('unsupported and indeterminate stay distinct statuses and are never failure', () => {
 for (const status of ['Proven', 'Failed', 'Indeterminate', 'Unsupported']) {
  const value = result();
  const executed = status !== 'Indeterminate';
  value.results[1].status = status;
  value.results[1].execution_performed = executed;
  value.results[1].local_execution_performed = executed;
  value.coverage_summary.exact_accounts_proven = status === 'Proven' ? 2 : 1;
  value.coverage_summary.exact_accounts_executed = executed ? 2 : 1;
  // A case that never executed is not shape coverage either.
  if (!executed) {
   value.shape_coverage[0].entities_executed = 1;
   value.shape_coverage[0].entities_untested = 8999;
   value.shape_coverage[0].executed_entity_ids = [value.selected_cases[0].entity_id];
  }
  verifyStressResult(value, job(), parent(), mechanism(), 0);
 }
 check(v => {v.results[1].status = 'NotTested';}, 'a stress case has no NotTested slot');
 check(v => {v.results[1].status = 'Succeeded';}, 'unknown status');
});

test('a nonzero engine exit code is never accepted as evidence', () => {
 assert.throws(() => verifyStressResult(result(), job(), parent(), mechanism(), 3), /InvalidEngineResult/);
});

const service = async () => {
 const directory = await mkdtemp(resolve(tmpdir(), 'eplyx-stress-'));
 const s = new AnalysisService({root, directory, runner: async () => ({code: 0, stdout: '{}'})});
 await s.initialize();
 return {service: s, directory};
};

test('stress-testing requires a completed candidate conversion owned by this session', async () => {
 const {service: s, directory} = await service();
 try {
  const walletRun = randomUUID(), conversion = randomUUID();
  s.jobs.set(walletRun, {id: walletRun, owner: 'session-a', status: 'Completed',
   selection: {mint: MINT, scope: 'wallet'}, capture_sha256: digest(1)});
  // A wallet run is not a candidate conversion.
  await assert.rejects(() => s.submitStress(walletRun, randomUUID(), 'session-a'), /InvalidParent/);
  s.jobs.set(conversion, {id: conversion, owner: 'session-a', status: 'Running',
   conversion_request: {}, plan_sha256: digest(4), selection: {mint: MINT}, parent_run_id: walletRun});
  // An unfinished conversion is not evidence to stress-test.
  await assert.rejects(() => s.submitStress(conversion, randomUUID(), 'session-a'), /InvalidParent/);
  s.jobs.get(conversion).status = 'Completed';
  // Another session cannot reach it.
  await assert.rejects(() => s.submitStress(conversion, randomUUID(), 'session-b'), /InvalidParent/);
  assert.equal(s.queue.length, 0, 'nothing was queued and no acquisition ran');
 } finally {await rm(directory, {recursive: true, force: true});}
});

test('the browser may request a stress test and nothing else', async () => {
 const {service: s, directory} = await service();
 try {
  const conversion = randomUUID();
  // The API derives the owner from the session cookie, so the fixture uses one.
  const session = 'a'.repeat(64);
  s.jobs.set(conversion, {id: conversion, owner: hash(session), status: 'Completed',
   conversion_request: {}, plan_sha256: digest(4), selection: {mint: MINT},
   parent_run_id: randomUUID()});
  const origin = 'http://127.0.0.1:4173';
  const post = async body => {
   let status = 0, payload = null;
   const request = Readable.from([Buffer.from(JSON.stringify(body))]);
   request.url = `/api/runs/${conversion}/stress`;
   request.method = 'POST';
   request.headers = {host: '127.0.0.1:4173', 'content-type': 'application/json',
    cookie: `eplyx_session=${session}`};
   const response = {writeHead: s => {status = s;}, end: v => {payload = v ? JSON.parse(v) : null;},
    setHeader: () => {}};
   await handleAnalysisAPI(s, request, response, origin);
   return {status, payload};
  };
  // Anything beyond the request key is refused: no budget, mint, account,
  // amount, program, endpoint, path or claimed status may cross the boundary.
  for (const extra of ['budget', 'max_decoded_accounts', 'mint', 'source', 'amount_decimal',
   'program', 'program_id', 'accounts', 'transaction', 'path', 'rpc_url', 'endpoint',
   'status', 'readiness', 'selected_cases', 'authorization', 'population']) {
   const {status, payload} = await post({request_key: randomUUID(), [extra]: 'injected'});
   assert.equal(status, 400, extra);
   assert.equal(payload.error.code, 'InvalidStress', extra);
  }
  const {status} = await post({request_key: 'not-a-uuid'});
  assert.equal(status, 400, 'the request key must be a valid identifier');
  assert.equal(s.queue.length, 0, 'no rejected request reached the queue');
 } finally {await rm(directory, {recursive: true, force: true});}
});

test('a stress run writes its own artifacts and never overwrites the conversion it came from', async () => {
 const {service: s, directory} = await service();
 try {
  const conversion = randomUUID();
  const artifact = JSON.stringify({kind: 'current-conversion', status: 'Proven'});
  await writeFile(resolve(directory, `${conversion}.artifact`), artifact);
  s.jobs.set(conversion, {id: conversion, owner: 'session-a', status: 'Completed', conversion_request: {},
   plan_sha256: digest(4), canonical_sha256: hash(artifact), selection: {mint: MINT},
   parent_run_id: randomUUID()});
  // The parent artifact bytes are verified before anything else happens.
  const bytes = await s.artifact(conversion);
  assert.equal(bytes.toString(), artifact);
  s.jobs.get(conversion).canonical_sha256 = digest(8);
  await assert.rejects(() => s.artifact(conversion), /EvidenceVerificationFailed/);
 } finally {await rm(directory, {recursive: true, force: true});}
});
