window.feedScripts = (window.feedScripts || 0) + 1;
if (window.feedScripts == 6) {
  const ticker = document.getElementById('ticker');
  let n = 0;
  setInterval(() => {
    fetch('/sleep?ms=600').then(r => r.text()).then(() => {
      const row = document.createElement('div');
      row.textContent = 'update ' + (++n);
      ticker.prepend(row);
      if (ticker.children.length > 5) ticker.lastElementChild.remove();
    });
  }, 500);
}
