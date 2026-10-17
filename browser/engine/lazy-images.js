// spec: https://html.spec.whatwg.org/multipage/urls-and-fetching.html#lazy-load-root-margin
globalThis.__lazyImages = (ak, imageUpdate) => {
  let observer = null;

  return (el, state, currentSrc) => {
    state.currentSrc = currentSrc;
    state.lazy = true;
    if (!observer) {
      observer = new IntersectionObserver(entries => {
        for (const e of entries) {
          if (!e.isIntersecting) continue;
          observer.unobserve(e.target);
          if (!e.target.__imageRequest || !e.target.__imageRequest.lazy) continue;
          ak.imageReveal(e.target);
          imageUpdate(e.target, true);
        }
      }, { rootMargin: '1250px', scrollMargin: '1250px' });
    }

    observer.observe(el);
  };
};
