// spec: https://html.spec.whatwg.org/multipage/urls-and-fetching.html#lazy-load-root-margin
globalThis.__lazyImages = (ak, imageUpdate) => {
  let observer = null;

  return el => {
    if (!observer) {
      observer = new IntersectionObserver(entries => {
        for (const e of entries) {
          if (!e.isIntersecting) continue;
          observer.unobserve(e.target);
          ak.imageReveal(e.target);
          imageUpdate(e.target, true);
        }
      }, { rootMargin: '1250px' });
    }

    observer.observe(el);
  };
};
