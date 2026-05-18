export function triggerConfetti(): void {
  const container = document.createElement("div");
  container.style.position = "fixed";
  container.style.top = "0";
  container.style.left = "0";
  container.style.width = "100%";
  container.style.height = "100%";
  container.style.pointerEvents = "none";
  container.style.zIndex = "9999";
  container.style.overflow = "hidden";
  document.body.appendChild(container);

  const colors = [
    "#ff5964",
    "#35a7ff",
    "#38b000",
    "#ffc857",
    "#e056fd",
    "#ff7979",
    "#22a6b3",
  ];
  const particleCount = 120;

  for (let i = 0; i < particleCount; i++) {
    const particle = document.createElement("div");
    particle.style.position = "absolute";
    particle.style.width = `${Math.random() * 8 + 6}px`;
    particle.style.height = `${Math.random() * 8 + 6}px`;
    particle.style.backgroundColor =
      colors[Math.floor(Math.random() * colors.length)];
    particle.style.borderRadius = Math.random() > 0.5 ? "50%" : "2px";

    const startX = Math.random() * window.innerWidth;
    particle.style.left = `${startX}px`;
    particle.style.top = "-20px";

    const drift = (Math.random() - 0.5) * 300;
    const duration = Math.random() * 2 + 1.5;
    const delay = Math.random() * 0.4;

    particle.style.transition = `transform ${duration}s cubic-bezier(0.25, 0.46, 0.45, 0.94) ${delay}s, opacity ${duration}s ease ${delay}s`;
    container.appendChild(particle);

    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        particle.style.transform = `translate3d(${drift}px, ${window.innerHeight + 50}px, 0) rotate(${Math.random() * 720}deg)`;
        particle.style.opacity = "0";
      });
    });
  }

  setTimeout(() => {
    container.remove();
  }, 3000);
}
