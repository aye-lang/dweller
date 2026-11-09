const g = document.getElementById("c").getContext("2d");
const label = document.getElementById("label");
let t = 0;
if (document.querySelector("meta[name=observe]")) new MutationObserver(() => {}).observe(document.body, { childList: true, subtree: true });
function step() {
  g.fillStyle = "#234";
  g.fillRect(0, 0, 400, 200);
  g.fillStyle = "#c64";
  g.fillRect(20 + (t % 300), 40, 60, 60);
  label.textContent = "0%";
  t++;
  requestAnimationFrame(step);
}
requestAnimationFrame(step);
