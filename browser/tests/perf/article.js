window.articleScripts = (window.articleScripts || 0) + 1;
document.querySelectorAll('p').forEach((p, i) => { if (i % 10 == 0) p.classList.add('lead'); });
