onmessage = e => {
  const gl = e.data.canvas.getContext('webgl');
  requestAnimationFrame(() => {
    gl.clearColor(1, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.enable(gl.SCISSOR_TEST);
    gl.scissor(20, 10, 40, 20);
    gl.clearColor(0, 0.5, 1, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
  });
};
