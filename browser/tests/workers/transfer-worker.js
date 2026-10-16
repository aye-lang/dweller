onmessage = e => {
  const { buf, view, kept } = e.data;
  let sum = 0;
  for (const b of new Uint8Array(buf)) sum += b;
  kept[0] = 100;
  postMessage({ sum, buf, view, kept }, { transfer: [buf] });
};
