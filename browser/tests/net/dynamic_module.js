// tests/browser.rs: window_dynamic_scripts. Records its query (n=I for an
// import(), g=I for an import of dynamic_root.js) when it evaluates.
const q = import.meta.url.slice(import.meta.url.indexOf('?') + 1);
window.evaluated.push(q);
export default q;
