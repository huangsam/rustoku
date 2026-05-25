import { toastContainer } from "./elements";

export function showToast(
  message: string,
  type: "success" | "error" | "info" = "info",
): void {
  if (!toastContainer) return;
  const toast = document.createElement("div");
  toast.className = `toast ${type}`;
  toast.textContent = message;
  toast.setAttribute("role", type === "error" ? "alert" : "status");
  toast.setAttribute("aria-live", type === "error" ? "assertive" : "polite");
  toast.setAttribute("aria-atomic", "true");
  toastContainer.appendChild(toast);

  // Force reflow
  void toast.offsetHeight;
  toast.classList.add("show");

  setTimeout(() => {
    toast.classList.remove("show");
    setTimeout(() => {
      toast.remove();
    }, 300);
  }, 3000);
}
