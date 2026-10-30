const ring = document.querySelector('.dial-fill');
const number = document.querySelector('#dial-value');
const motionPreference = matchMedia('(prefers-reduced-motion: reduce)');
const colors = [[239, 102, 102], [244, 198, 86], [140, 233, 174]];
let animationFrame;

function renderQuota(progress) {
  const segment = progress < 0.5 ? 0 : 1;
  const blend = progress < 0.5 ? progress * 2 : (progress - 0.5) * 2;
  const color = colors[segment].map((channel, index) =>
    Math.round(channel + (colors[segment + 1][index] - channel) * blend));
  number.textContent = String(Math.round(82 * progress));
  ring.style.strokeDasharray = `${82 * progress} 100`;
  ring.style.stroke = `rgb(${color.join(',')})`;
  ring.style.opacity = progress === 0 ? '0' : '1';
  ring.style.filter = `drop-shadow(0 0 7px rgba(${color.join(',')},0.2))`;
}

if (motionPreference.matches) {
  renderQuota(1);
} else {
  renderQuota(0);
  let startedAt;
  const tick = (time) => {
    startedAt ??= time;
    const elapsed = Math.min((time - startedAt) / 1000, 1);
    // 数字、弧长与颜色共用缓动进度，避免三者错位。
    renderQuota(elapsed * elapsed * (3 - 2 * elapsed));
    if (elapsed < 1) animationFrame = requestAnimationFrame(tick);
  };
  animationFrame = requestAnimationFrame(tick);
}
motionPreference.addEventListener('change', (event) => {
  if (event.matches) {
    cancelAnimationFrame(animationFrame);
    renderQuota(1);
  }
});
