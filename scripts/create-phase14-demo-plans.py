#!/usr/bin/env python3
"""Freeze four explicit non-issuer candidate cases from retained Phase 10/11/13 data.
This creates submitted assertions, never execution evidence or a policy evaluation.
"""
from pathlib import Path
import copy,hashlib,json
ROOT=Path(__file__).resolve().parents[1]
def read(p): return json.loads((ROOT/p).read_text())
def digest(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def save(p,v):
    path=ROOT/p;path.parent.mkdir(parents=True,exist_ok=True)
    with path.open('x') as f: f.write(json.dumps(v,indent=2)+'\n')
def ref(file,path): return {'file':file,'sha256':digest(path)}
def main():
    policy_path='policies/stocklana-spacex-preflight-v1.json'
    parent=read(policy_path);parent_sha=digest(policy_path)
    frozen=read('reports/spacex-lifecycle-readiness.json')
    views=read('reports/spacex-counterfactual-lifecycle.json')
    requirements={r['id']:r for r in parent['requirements']}
    principal=next(f for f in frozen['path_evidence'] if f['path_type']=='Withdrawal' and f['status']=='Proven')
    transfer_condition=next(c for c in requirements['direct-mobility']['condition']['any_of'] if c['path_type']=='Transfer')
    transfer=next(f for f in frozen['path_evidence'] if f['path_type']=='Transfer' and f['scope']==transfer_condition['scope'])
    failed_condition=requirements['optional-failed-route']['condition']['any_of'][0]
    failed=next(f for f in frozen['path_evidence'] if f['path_type']=='SecondaryMarketExit' and f['scope']==failed_condition['scope'])
    assert failed['status']=='Failed' and failed['rollback_verified'] is True
    assert 'BitmapExtensionAccountIsNotProvided' in failed['reason'] and '6036' in failed['reason']
    position=frozen['position_exit_evidence'][0]
    w=read('reports/spacex-dlmm-withdrawal.json')
    assert w['all_position_liquidity_removed'] is True and w['position_retained'] is True
    assert any(n!='0' for n in position['residual_fees_raw'].values())
    target=next(s['scenario'] for s in views['scenarios'] if s['scenario']['id']=='after_transition')
    route=copy.deepcopy(parent);route['id']='stocklana-phase14-required-exact-route-v1'
    route['description']='Non-issuer demonstration variant: require the exact formerly optional retained failed sale; all population requirements remain unchanged.'
    next(r for r in route['requirements'] if r['id']=='optional-failed-route')['required']=True
    narrow=copy.deepcopy(parent);narrow['id']='stocklana-phase14-principal-removal-only-v1'
    narrow['description']='Non-issuer entity/action demonstration assurance for exact historical principal removal only; not complete exit, conversion, population readiness or future availability.'
    narrow['evaluated_scope']='DemoEntityReadiness';narrow['requirements']=[copy.deepcopy(requirements['lp-principal-unwind'])]
    save('policies/phase14-required-exact-route.json',route)
    save('policies/phase14-principal-removal-only.json',narrow)
    save('probes/phase14-evidence-binding.json',{'schema_version':1,'parent_policy':ref('../'+policy_path,policy_path),'counterfactual':ref('../reports/spacex-counterfactual-lifecycle.json','reports/spacex-counterfactual-lifecycle.json')})
    def binding(path,derivation,rationale): return {'artifact':ref('../../'+path,path),'parent_policy_sha256':parent_sha,'derivation':derivation,'rationale':rationale}
    original=binding(policy_path,{'kind':'Original'},'Use the unchanged original population assurance policy; unsupported assertions do not manufacture a Blocked result.')
    def action(id,f,path_type=None,scope=None): return {'id':id,'path_type':path_type or f['path_type'],'scope':copy.deepcopy(scope or f['scope']),'signer':copy.deepcopy(f['signer']),'runtime_assumption':'CapturedDeployedProgramsInLocalLiteSvm'}
    def assertion(id,action_id,kind,statement,ids,**fields): return {'id':id,'action_id':action_id,'claim':{'kind':kind,**fields},'evidence_ids':ids,'statement':statement}
    pid=principal['evidence_ids'];tid=transfer['evidence_ids'];fid=failed['evidence_ids']
    deltas={k:copy.deepcopy(position[k]) for k in ['principal_removed_raw','owner_received_raw','destination_withheld_raw']}
    def principal_assertions(): return [
        assertion('principal-operation','principal-removal','ExactPathSucceeded','The exact principal-removal operation succeeded locally under the recorded owner-signing and runtime assumptions.',pid),
        assertion('principal-token-deltas','principal-removal','ExactPrincipalTokenDeltas','The exact locally measured principal debits, owner credits and destination withheld transfer fees match the submitted ledgers.',pid,expected=deltas),
        assertion('zero-liquidity-shares','principal-removal','ZeroRemainingLiquidityShares','All liquidity shares became zero in this exact local full-range removal.',pid)]
    def plan(id,label,actions,claims,b,policy):
        ids=sorted({id for c in claims for id in c['evidence_ids']})
        return {'schema_version':1,'id':id,'version':1,'label':label,'provenance':'DemonstrationNonIssuer','target_view':copy.deepcopy(target),'production_state_digest':views['production_world']['production_state_digest'],'assurance_policy':b,'requested_actions':actions,'assertions':claims,'referenced_evidence':[copy.deepcopy(next(r for r in frozen['evidence_refs'] if r['id']==id)) for id in ids],'required_assurance_conditions':sorted(r['id'] for r in policy['requirements'] if r['required'])}
    claims=principal_assertions()+[
        assertion('no-protocol-fees','principal-removal','NoRemainingProtocolFees','No position-level protocol-accrued fee exposure remains.',pid),
        assertion('fee-collection','principal-removal','FeeCollectionCompleted','Fee collection has completed.',pid),
        assertion('position-closure','principal-removal','PositionClosed','The position account has closed.',pid),
        assertion('complete-exit','principal-removal','CompletePositionExit','Because all principal shares were removed, the entire protocol position has completed its exit and no position-level exposure remains.',pid),
        assertion('official-conversion','official-conversion','OfficialConversionCompleted','Principal removal also completed official successor conversion.',pid),
        assertion('lifecycle-completion','official-conversion','LifecycleCompletion','The position has completed its entire asset lifecycle.',pid)]
    lp=plan('lp-complete-exit-claim','Claim complete LP position exit',[action('principal-removal',principal),action('official-conversion',principal,'OfficialTransition',requirements['lp-official-transition']['condition']['any_of'][0]['scope'])],claims,original,parent)
    required=plan('required-failed-route-claim','Require the exact retained failed sale',[action('required-sale',failed)],[assertion('route-success','required-sale','ExactPathSucceeded','This exact requested sale route succeeds in its recorded captured execution context.',fid)],binding('policies/phase14-required-exact-route.json',{'kind':'RequireExistingExactRoute','requirement_id':'optional-failed-route'},'The candidate explicitly relies on this exact sale, making the formerly optional observation mandatory; no other policy condition changes.'),route)
    conversion=plan('transfer-official-conversion-claim','Claim Transfer completed official conversion',[action('token-movement',transfer),action('official-conversion',transfer,'OfficialTransition',requirements['direct-official-transition']['condition']['any_of'][0]['scope'])],[assertion('transfer-operation','token-movement','ExactPathSucceeded','The original exact Transfer succeeded locally under its recorded holder signing assumptions.',tid),assertion('official-from-transfer','official-conversion','OfficialConversionCompleted','The successful Transfer establishes official successor conversion completion.',tid)],original,parent)
    positive=plan('principal-removal-positive-control','Claim only exact local principal removal',[action('principal-removal',principal)],principal_assertions(),binding('policies/phase14-principal-removal-only.json',{'kind':'PrincipalRemovalOnly','requirement_id':'lp-principal-unwind'},'Separate entity/action policy requires only independently measured principal-removal proof; it does not weaken or replace population rollout assurance.'),narrow)
    paths=[]
    for name,p in [('lp-complete-exit',lp),('required-failed-route',required),('transfer-official-conversion',conversion),('principal-removal-positive-control',positive)]:
        path=f'probes/phase14-plans/{name}.json';save(path,p);paths.append(ref(f'phase14-plans/{name}.json',path))
    save('probes/phase14-demo-cases.json',{'schema_version':1,'plans':paths})
    print('Four submitted candidate plans and two separately identified policy variants frozen; no evidence or original policy changed.')
if __name__=='__main__': main()
