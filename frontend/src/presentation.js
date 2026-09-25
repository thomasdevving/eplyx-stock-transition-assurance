import { esc } from './format.js';
export const pathLabels = Object.freeze({OfficialTransition:'Convert to the replacement token',SecondaryMarketExit:'Sell through a trading pool',Transfer:'Send tokens to another account',Withdrawal:'Withdraw liquidity',Redemption:'Redeem through the issuer'});
export const statusLabels = Object.freeze({Proven:'Passed in VM simulation',Failed:'Failed in VM simulation',Indeterminate:'Could not complete this check',Unsupported:'Not supported by this checker',NotTested:'Not tested yet',NotApplicable:'Not applicable in this context',Ready:'Selected requirements met',Blocked:'A required condition was not met',Incomplete:'More evidence needed',Supported:'Supported by the saved evidence',Contradicted:'Contradicted by the saved evidence',NotEstablished:'Not established yet',EvidenceSubstitution:'This evidence checks a different action',ScopeMismatch:'This evidence covers a different example or amount',Accepted:'Selected statement supported',NotAccepted:'The full statement is not supported'});
export const checkLabels=Object.freeze({'complete-exit':'Has the liquidity position been fully cleared?','required-sale':'Does this exact sale route work?','transfer-conversion':'Does sending the token complete the official conversion?','principal-removal':'Did the tested liquidity-principal withdrawal succeed?'});
export const stageLabels=Object.freeze({before_transition:'Before the transition',after_transition:'During the transition',after_deadline:'After the stated deadline'});
export function tokenAmount(raw,decimals) {
 if(!/^\d+$/.test(String(raw)) || !Number.isInteger(decimals) || decimals<0 || decimals>255)throw new Error('Invalid amount');
 const n=BigInt(raw), divisor=10n**BigInt(decimals), fraction=(n%divisor).toString().padStart(decimals,'0').replace(/0+$/,'');
 return `${(n/divisor).toLocaleString('en-US')}${fraction?'.'+fraction:''}`;
}
export const readableStatus=value=>statusLabels[value]||'This check needs review';
export const limitations = `<p class="analysis-boundary">Saved mainnet data · Earlier local simulations · No funds moved by this analysis</p><p class="analysis-limits">These are example holdings, not your own. Signing was assumed locally; actual signing access was not verified. Requirements and the effective transition time are demonstration choices, not an issuer’s internal policy. Observations were saved at different times. A later scenario date does not make earlier tests fresh or predict balances or trading conditions.</p>`;
export function conclusion(status,applicability) {
 if(applicability==='PreEvent')return 'The transition requirement has not started in this selected scenario.';
 return status==='Ready'?'The selected principal-removal requirements are met.':status==='Blocked'?'This exact required sale route failed in the saved local simulation.':'More evidence is needed before this transition can be considered ready.';
}
export const plainPaths=paths=>`<div class="requirement-list">${paths.map(p=>`<article><h3>${esc(pathLabels[p.path_type||p.path]||'Selected action')}</h3><span>${esc(readableStatus(p.status))}</span></article>`).join('')}</div>`;
