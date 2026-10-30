const downloadDialog = document.querySelector('#download-dialog');
let downloadTrigger;

document.querySelectorAll('[data-download]').forEach((link) => {
  link.addEventListener('click', (event) => {
    // 保留原生附件下载，也尊重用户的新标签页、保存链接等操作。
    if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    downloadTrigger = link;
    if (!downloadDialog.open) downloadDialog.showModal();
  });
});
downloadDialog.querySelectorAll('.dialog-close, .dialog-dismiss').forEach((button) => {
  button.addEventListener('click', () => downloadDialog.close());
});
downloadDialog.addEventListener('click', (event) => {
  if (event.target !== downloadDialog) return;
  const bounds = downloadDialog.getBoundingClientRect();
  if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) downloadDialog.close();
});
downloadDialog.addEventListener('close', () => downloadTrigger?.focus({ preventScroll: true }));
document.querySelector('#download-help').addEventListener('click', (event) => {
  event.preventDefault();
  downloadDialog.close();
  const help = document.querySelector('#first-open');
  help.scrollIntoView({ block: 'start' });
  // 关闭事件先归还触发点焦点，再将焦点移到用户选择的安装帮助。
  requestAnimationFrame(() => help.focus({ preventScroll: true }));
});
