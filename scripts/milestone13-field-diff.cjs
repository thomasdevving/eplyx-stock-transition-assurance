const fs = require('fs');
const crypto = require('crypto');

const alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
function b58(bytes) {
  let value = BigInt('0x' + Buffer.from(bytes).toString('hex'));
  let result = '';
  while (value > 0n) {
    result = alphabet[Number(value % 58n)] + result;
    value /= 58n;
  }
  for (const byte of bytes) { if (byte !== 0) break; result = '1' + result; }
  return result || '1';
}
function opt(bytes, offset) {
  return bytes.readUInt32LE(offset) === 0 ? null : b58(bytes.subarray(offset + 4, offset + 36));
}
function tokenFields(bytes) {
  return {
    mint: b58(bytes.subarray(0, 32)),
    recorded_authority: b58(bytes.subarray(32, 64)),
    raw_amount: bytes.readBigUInt64LE(64).toString(),
    delegate: opt(bytes, 72),
    account_state: bytes[108],
    native_reserve: bytes.readUInt32LE(109) === 0 ? null : bytes.readBigUInt64LE(113).toString(),
    delegated_amount: bytes.readBigUInt64LE(121).toString(),
    close_authority: opt(bytes, 129),
  };
}
function extensions(bytes) {
  const output = [];
  if (bytes.length <= 165) return output;
  output.push({field:'account_type', value:bytes[165]});
  let offset = 166;
  while (offset + 4 <= bytes.length) {
    const type = bytes.readUInt16LE(offset), length = bytes.readUInt16LE(offset + 2);
    if (type === 0 && length === 0) break;
    const data = bytes.subarray(offset + 4, offset + 4 + length);
    output.push({field:'tlv', type, length, sha256:sha(data)});
    offset += 4 + length;
  }
  output.push({field:'trailing', length:bytes.length-offset, sha256:sha(bytes.subarray(offset))});
  return output;
}
function sha(bytes) { return crypto.createHash('sha256').update(bytes).digest('hex'); }
function spans(a,b) {
  const ranges = [];
  let start = null;
  for(let i=0;i<Math.max(a.length,b.length);i++) {
    if(a[i] !== b[i]) { if(start === null) start=i; }
    else if(start !== null) { ranges.push([start,i-1]); start=null; }
  }
  if(start !== null) ranges.push([start,Math.max(a.length,b.length)-1]);
  return ranges;
}

const output = [];
for(const name of ['healthy','second','underfunded']) {
  const dir = `target/milestone12-retry-${name}-fastrpc`;
  const population = JSON.parse(fs.readFileSync(`${dir}/population.capture.json`));
  const stress = JSON.parse(fs.readFileSync(`${dir}/stress.cases.json`));
  const plan = JSON.parse(fs.readFileSync(`${dir}/stress.plan.json`));
  const populationRows = new Map(population.observations[2].result.value.map(row=>[row.pubkey,row.account]));
  const originalIndices = new Map(population.observations[2].result.value.map((row,index)=>[row.pubkey,index]));
  const sortedIndices = new Map([...populationRows.keys()].sort().map((address,index)=>[address,index]));
  const selected = new Map(plan.selected.map(row=>[row.token_account,row]));
  const cases=[];
  for(const c of stress.cases) {
    const p = populationRows.get(c.token_account);
    const record = c.observations.at(-1);
    const finalIndex = record.params[0].indexOf(c.token_account);
    const f = record.result.value[finalIndex];
    const a = Buffer.from(p.data[0],'base64'), b=Buffer.from(f.data[0],'base64');
    const ta = tokenFields(a), tb=tokenFields(b);
    const decodedDifferences = Object.keys(ta).filter(key=>ta[key]!==tb[key]).map(key=>({field:key,population:ta[key],final:tb[key]}));
    const runtimeDifferences = ['owner','executable','lamports','rentEpoch','space'].filter(key=>p[key]!==f[key]).map(key=>({field:key,population:p[key],final:f[key]}));
    const selectedCase=selected.get(c.token_account);
    cases.push({
      case_id:c.case_id,token_account:c.token_account,
      selection_reason:selectedCase.selection_reason,selection_bucket:selectedCase.balance_bucket,
      population_rpc_id:2,
      population_pointer:`/value/${sortedIndices.get(c.token_account)}`,
      actual_population_pointer:`/value/${originalIndices.get(c.token_account)}/account`,
      population_context_slot:population.observations[2].result.context.slot,
      final_context_slot:record.result.context.slot,
      elapsed_seconds:(Date.parse(record.completed_at)-Date.parse(population.observations[2].completed_at))/1000,
      population_data_sha256:sha(a),final_data_sha256:sha(b),
      population_data_length:a.length,final_data_length:b.length,
      raw_byte_difference_spans:spans(a,b),
      decoded_differences:decodedDifferences,runtime_differences:runtimeDifferences,
      extension_differences:JSON.stringify(extensions(a))===JSON.stringify(extensions(b))?[]:{population:extensions(a),final:extensions(b)}
    });
  }
  output.push({run:name,cases});
}
const report = {
  schema_version:1,
  kind:'milestone13-saved-fastrpc-field-diff',
  source:'saved milestone12 FastRPC retry captures',
  summary:{
    cases:output.reduce((n,run)=>n+run.cases.length,0),
    changed_token_bytes:output.flatMap(run=>run.cases).filter(c=>c.raw_byte_difference_spans.length).length,
    changed_decoded_fields:output.flatMap(run=>run.cases).filter(c=>c.decoded_differences.length).length,
    changed_runtime_fields:output.flatMap(run=>run.cases).filter(c=>c.runtime_differences.length).length,
    mismatched_population_pointers:output.flatMap(run=>run.cases).filter(c=>c.population_pointer!==c.actual_population_pointer).length,
  },
  runs:output,
};
fs.writeFileSync('reports/milestone13-field-diff.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report.summary));
