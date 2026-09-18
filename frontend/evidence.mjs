import { readFile, writeFile, mkdir, copyFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';

// These are published report bytes, not a fresh execution or assurance attestation.
export const pinnedReports = {
  rollout: ['reports/spacex-rollout-assumptions.json', 'f2bf9ffb52d742e2f36f0bae99a6916e90fd74d75c9d6a046e7d465d65039978'],
  readiness: ['reports/spacex-notice-readiness.json', 'fb2b093170d5b4c85461c2d83b4d99569b2d80d9f6b62c3783bbfd2a843a4953'],
  direct: ['reports/spacex-lifecycle-path-resolution-phase9.json', '993989bcdad106d7eef79895e258f4ce14bad18ec810503932688e0ebeb8e06d'],
  withdrawal: ['reports/spacex-dlmm-withdrawal.json', '5f74a2b70946efbc937fba9e9bcd028ce6b6586184104424c8f28e7ba39ddff5'],
  event: ['reports/spacex-lifecycle-event.json', 'b8f91dd0aba091787d8a1368e3c5b0887778f72732e7e9d977ba8f4d7edf3824'],
};
const root = new URL('../', import.meta.url);
export async function prepareEvidence() {
  const directory = new URL('./public/evidence/', import.meta.url);
  await mkdir(directory, { recursive: true });
  const data = {}, artifacts = [];
  for (const [key, [file, expected]] of Object.entries(pinnedReports)) {
    const bytes = await readFile(new URL(file, root));
    const sha256 = createHash('sha256').update(bytes).digest('hex');
    if (sha256 !== expected) throw new Error(`Published evidence changed: ${file}. Review the new report before updating its frontend pin.`);
    data[key] = JSON.parse(bytes);
    const name = file.split('/').at(-1);
    await writeFile(new URL(name, directory), bytes);
    artifacts.push({ key, file, sha256, href: `/public/evidence/${name}` });
  }
  for (const phase of [10, 11, 12, 14, 15]) {
    const name = `lifecycle-phase-${phase}-production-report.md`;
    await copyFile(new URL(`docs/${name}`, root), new URL(name, directory));
  }
  const { readiness: r, direct: d, withdrawal: w, event: e } = data;
  const summary = {
    rolloutCases: data.rollout?.cases || [],
    status: r.overall_status, policy: r.policy.id, evaluatedAt: r.policy_evaluated_at,
    population: r.population_summary,
    findings: r.findings.map(({requirement_id, label, effect, required}) => ({id: requirement_id, label, effect, required})),
    direct: { entity: d.entity_id, authority: d.owner_authority, amount: d.observed_public_balance_raw,
      paths: d.paths.map(({path_type, status, reason, contexts}) => ({path: path_type, status, reason,
        attempts: contexts.flatMap(c => c.attempts).filter(a => a.exact_input_raw === d.observed_public_balance_raw && !a.invalid_control)})) },
    position: { entity: w.position.position_id, scope: w.scope, signer: w.position.signer,
      ...r.position_exit_evidence[0], paths: w.paths },
    notice: { source: e.source_url, deadline: e.deadline.value, deadlineWording: e.deadline_wording.value,
      mechanism: e.official_mechanism.value, ratio: e.conversion_ratio.value,
      officialStatus: e.official_execution_status.value, effectiveAt: e.effective_at.value,
      sourceIdentity: e.source_identity, successorIdentity: e.successor_identity,
      sourceDigest: e.source_content_sha256 },
    artifacts,
  };
  await writeFile(new URL('summary.js', directory), `// Generated from pinned published reports by frontend/evidence.mjs.\nexport default ${JSON.stringify(summary, null, 2)};\n`);
  return summary;
}
