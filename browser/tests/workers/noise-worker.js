import { valueNoise } from './noise.js';

self.onmessage = e => {
  const { size, scale, seed } = e.data;
  const data = new Float32Array(size * size);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      data[y * size + x] = valueNoise(x / scale, y / scale, seed);
    }
  }
  self.postMessage({ size, data }, [data.buffer]);
  console.log('worker detached', data.buffer.byteLength === 0, data.length);
};
