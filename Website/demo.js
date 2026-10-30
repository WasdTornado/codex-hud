const desktop = document.querySelector('.desktop');
if (desktop) {
  const hud = desktop.querySelector('.hud');
  const detail = desktop.querySelector('.detail');
  const controls = document.querySelector('.demo-controls');
  const setExpanded = (open) => {
    detail.classList.toggle('open', open);
    detail.inert = !open;
    hud.setAttribute('aria-expanded', String(open));
  };
  hud.addEventListener('click', () => setExpanded(hud.getAttribute('aria-expanded') !== 'true'));
  document.addEventListener('click', (event) => {
    if (!event.target.closest('.hud-zone')) setExpanded(false);
  });
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && hud.getAttribute('aria-expanded') === 'true') {
      setExpanded(false);
      hud.focus();
    }
  });
  controls.querySelectorAll('.mode').forEach((button) => {
    button.addEventListener('click', () => {
      const single = button.dataset.mode === 'single';
      desktop.classList.toggle('single', single);
      hud.setAttribute('aria-label', single ? '查看额度详情，5 小时剩余 82%' : '查看额度详情，5 小时剩余 82%，每周剩余 47%');
      controls.querySelectorAll('.mode').forEach((item) => {
        item.classList.toggle('active', item === button);
        item.setAttribute('aria-pressed', String(item === button));
      });
    });
  });
  controls.querySelector('.theme-toggle').addEventListener('click', (event) => {
    event.currentTarget.setAttribute('aria-pressed', String(desktop.classList.toggle('night')));
  });
  desktop.querySelector('.refresh').addEventListener('click', (event) => {
    const button = event.currentTarget;
    button.disabled = true;
    button.textContent = '✓';
    button.setAttribute('aria-label', '演示刷新完成，数据为示例');
    setTimeout(() => {
      button.disabled = false;
      button.textContent = '↻';
      button.setAttribute('aria-label', '刷新演示数据');
    }, 900);
  });
}
