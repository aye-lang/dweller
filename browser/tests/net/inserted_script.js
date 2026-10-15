// net/inserted_scripts.html: logs the query of its URL.
window.ran.push(new URL(document.currentScript.src).search.slice(1));
