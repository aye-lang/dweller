document.getElementById("log").textContent += " external";
addEventListener("wheel", () => {
    const wheel = document.getElementById("wheel");
    if (wheel.textContent == "wheel: none") wheel.textContent = "wheel after" + document.getElementById("log").textContent.slice(4);
}, { passive: true });
