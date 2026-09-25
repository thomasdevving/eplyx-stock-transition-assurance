// Optional access code for a hosted analysis service. Without a configured
// code (every local run) nothing changes. With EPLYX_ANALYSIS_ACCESS_CODE set,
// only browsers that entered the code may start or read analyses, so a public
// deployment cannot spend the operator's RPC quota or compute. The code only
// gates who may ask; it never changes what an analysis concludes.
import { createHash, createHmac, timingSafeEqual } from 'node:crypto';

const WINDOW_MS = 15 * 60 * 1000;
const MAX_FAILURES = 10;
const digest = value => createHash('sha256').update(value).digest();

export function readCookie(request, name) {
 return String(request.headers.cookie ?? '').split(';').map(part => part.trim().split('=')).find(([key]) => key === name)?.[1];
}

export class AccessGate {
 constructor(code, { secure = false, now = Date.now } = {}) {
  this.required = typeof code === 'string' && code.trim().length > 0;
  this.codeDigest = this.required ? digest(`eplyx-analysis-access-code:${code.trim()}`) : null;
  this.key = this.required ? digest(`eplyx-analysis-access-key:${code.trim()}`) : null;
  this.secure = secure;
  this.now = now;
  this.failures = new Map();
 }

 token(session) {
  return createHmac('sha256', this.key).update(session).digest('hex');
 }

 /** The access cookie is bound to this browser's session and to the code. */
 unlocked(request, session) {
  if (!this.required) return true;
  const value = readCookie(request, 'eplyx_access');
  if (!value || !/^[a-f0-9]{64}$/.test(value)) return false;
  return timingSafeEqual(Buffer.from(value, 'hex'), Buffer.from(this.token(session), 'hex'));
 }

 client(request) {
  const forwarded = String(request.headers['x-forwarded-for'] ?? '').split(',')[0].trim();
  return forwarded || request.socket?.remoteAddress || 'unknown';
 }

 blocked(client) {
  const now = this.now();
  for (const [key, entry] of this.failures) if (now - entry.start > WINDOW_MS) this.failures.delete(key);
  return (this.failures.get(client)?.count ?? 0) >= MAX_FAILURES;
 }

 /** Returns 'ok', 'invalid' or 'blocked'. */
 check(client, code) {
  if (!this.required) return 'ok';
  if (this.blocked(client)) return 'blocked';
  const ok = typeof code === 'string' && code.length <= 256 && timingSafeEqual(digest(`eplyx-analysis-access-code:${code.trim()}`), this.codeDigest);
  if (ok) { this.failures.delete(client); return 'ok'; }
  const entry = this.failures.get(client) ?? { count: 0, start: this.now() };
  entry.count += 1;
  this.failures.set(client, entry);
  return 'invalid';
 }

 cookie(session) {
  return `eplyx_access=${this.token(session)}; HttpOnly; SameSite=Strict; Path=/; Max-Age=2592000${this.secure ? '; Secure' : ''}`;
 }
}
