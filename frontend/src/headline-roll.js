// Every term sits on the same single line of the headline: the slot is exactly
// one line tall and never wraps, so only one term is ever visible and the lines
// around it never move. If the widest term cannot fit, all terms shrink by the
// same factor rather than wrapping onto a second line.
export function attachHeadlineRoll() {
 const slot = document.querySelector('.transition-roll');
 if (!slot) return () => {};
 const track = slot.querySelector('.transition-roll__track');
 const terms = [...track.children];
 const motion = matchMedia('(prefers-reduced-motion: reduce)');
 let animation, width = 0, disposed = false, onscreen = false;
 const visibility = () => {
  if (!animation) return;
  if (onscreen && !document.hidden) animation.play();
  else animation.pause();
 };
 const measure = () => {
  if (disposed) return;
  const time = animation?.currentTime || 0;
  animation?.cancel();
  animation = null;
  slot.style.removeProperty('--roll-fit');
  width = slot.getBoundingClientRect().width;
  const widest = Math.max(...terms.map(term => term.scrollWidth));
  if (widest > width) slot.style.setProperty('--roll-fit', (width / widest).toFixed(4));
  const line = terms[0].getBoundingClientRect().height;
  slot.style.height = `${line}px`;
  if (motion.matches) return;
  const count = terms.length - 1;
  const positionFrames = [];
  terms.forEach((term, index) => {
   const offsets = index === count ? [1] : [index / count, (index + .8) / count];
   offsets.forEach(offset => {
    positionFrames.push({ transform: `translateY(-${(index * line).toFixed(2)}px)`, offset, easing: 'cubic-bezier(.65, 0, .35, 1)' });
   });
  });
  const timing = { duration: count * 3500, iterations: Infinity };
  animation = track.animate(positionFrames, timing);
  animation.currentTime = time;
  visibility();
 };
 const observer = new ResizeObserver(() => {
  if (Math.abs(slot.getBoundingClientRect().width - width) > .5) measure();
 });
 observer.observe(slot);
 const screenObserver = new IntersectionObserver(([entry]) => {
  onscreen = entry.isIntersecting;
  visibility();
 });
 screenObserver.observe(slot);
 document.addEventListener('visibilitychange', visibility);
 motion.addEventListener('change', measure);
 measure();
 document.fonts?.ready.then(measure);
 return () => {
  disposed = true;
  observer.disconnect();
  screenObserver.disconnect();
  document.removeEventListener('visibilitychange', visibility);
  motion.removeEventListener('change', measure);
  animation?.cancel();
 };
}
