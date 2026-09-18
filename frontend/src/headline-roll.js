// Measure each term at the inherited headline size so multiline terms stay visible.
export function attachHeadlineRoll() {
 const slot = document.querySelector('.transition-roll');
 if (!slot) return () => {};
 const track = slot.querySelector('.transition-roll__track');
 const terms = [...track.children];
 const motion = matchMedia('(prefers-reduced-motion: reduce)');
 let animations = [], width = 0, disposed = false;
 const measure = () => {
  if (disposed) return;
  const time = animations[0]?.currentTime || 0;
  animations.forEach(animation => animation.cancel());
  animations = [];
  width = slot.getBoundingClientRect().width;
  if (motion.matches) { slot.style.height = `${terms[0].getBoundingClientRect().height}px`; return; }
  const heights = terms.map(term => term.getBoundingClientRect().height);
  const count = terms.length - 1;
  const positionFrames = [], heightFrames = [];
  let top = 0;
  terms.forEach((term, index) => {
   const offsets = index === count ? [1] : [index / count, (index + .8) / count];
   offsets.forEach(offset => {
    positionFrames.push({ transform: `translateY(-${top}px)`, offset, easing: 'cubic-bezier(.65, 0, .35, 1)' });
    heightFrames.push({ height: `${heights[index]}px`, offset, easing: 'cubic-bezier(.65, 0, .35, 1)' });
   });
   top += heights[index];
  });
  const timing = { duration: count * 3500, iterations: Infinity };
  animations = [track.animate(positionFrames, timing), slot.animate(heightFrames, timing)];
  animations.forEach(animation => { animation.currentTime = time; });
 };
 const observer = new ResizeObserver(() => {
  if (Math.abs(slot.getBoundingClientRect().width - width) > .5) measure();
 });
 observer.observe(slot);
 motion.addEventListener('change', measure);
 measure();
 document.fonts?.ready.then(measure);
 return () => {
  disposed = true;
  observer.disconnect();
  motion.removeEventListener('change', measure);
  animations.forEach(animation => animation.cancel());
 };
}
