export const esc = value => String(value ?? 'Unknown').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
export const number = value => BigInt(value).toLocaleString('en-US');
export const label = value => value.replace(/([a-z])([A-Z])/g, '$1 $2');
export const badge = status => `<span class="evidence-badge ${['Proven','Satisfied'].includes(status) ? 'evidence-badge--proven' : ''}">${esc(label(status))}</span>`;
