const cells = document.getElementById('grid').children;
let frame = 0;
function step() {
  const prev = cells[frame % cells.length];
  prev.classList.remove('on');
  prev.style.transform = '';
  frame++;
  const cell = cells[frame % cells.length];
  cell.classList.add('on');
  cell.style.transform = 'translateY(' + (frame % 4) + 'px)';
  requestAnimationFrame(step);
}
requestAnimationFrame(step);
