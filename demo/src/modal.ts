const FOCUSABLE_SELECTOR = [
  "button:not([disabled])",
  "[href]",
  'input:not([disabled]):not([type="hidden"])',
  "select:not([disabled])",
  "textarea:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(", ");

let activeModal: HTMLDivElement | null = null;
let modalTrigger: HTMLElement | null = null;
let isInitialized = false;

function getFocusableElements(modal: HTMLDivElement): HTMLElement[] {
  return Array.from(modal.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
}

function setModalVisibility(modal: HTMLDivElement, isOpen: boolean): void {
  modal.style.display = isOpen ? "flex" : "none";
  modal.setAttribute("aria-hidden", isOpen ? "false" : "true");
  if (!modal.hasAttribute("tabindex")) {
    modal.tabIndex = -1;
  }
}

function focusInitialElement(modal: HTMLDivElement): void {
  const [firstFocusable] = getFocusableElements(modal);
  (firstFocusable ?? modal).focus();
}

function handleModalKeydown(event: KeyboardEvent): void {
  if (!activeModal) {
    return;
  }

  if (event.key === "Escape") {
    event.preventDefault();
    closeModal(activeModal);
    return;
  }

  if (event.key !== "Tab") {
    return;
  }

  const focusableElements = getFocusableElements(activeModal);
  if (focusableElements.length === 0) {
    event.preventDefault();
    activeModal.focus();
    return;
  }

  const firstFocusable = focusableElements[0];
  const lastFocusable = focusableElements[focusableElements.length - 1];
  const currentFocus = document.activeElement as HTMLElement | null;

  if (event.shiftKey) {
    if (currentFocus === firstFocusable || currentFocus === activeModal) {
      event.preventDefault();
      lastFocusable.focus();
    }
    return;
  }

  if (currentFocus === lastFocusable) {
    event.preventDefault();
    firstFocusable.focus();
  }
}

export function initializeModalAccessibility(): void {
  if (isInitialized) {
    return;
  }

  document.addEventListener("keydown", handleModalKeydown);
  isInitialized = true;
}

export function openModal(
  modal: HTMLDivElement,
  trigger?: HTMLElement | null,
): void {
  if (activeModal && activeModal !== modal) {
    setModalVisibility(activeModal, false);
  }

  modalTrigger =
    trigger ??
    (document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null);
  activeModal = modal;
  setModalVisibility(modal, true);
  focusInitialElement(modal);
}

export function closeModal(modal: HTMLDivElement): void {
  const shouldRestoreFocus = activeModal === modal;

  setModalVisibility(modal, false);

  if (!shouldRestoreFocus) {
    return;
  }

  activeModal = null;
  const trigger = modalTrigger;
  modalTrigger = null;

  if (trigger && document.contains(trigger)) {
    trigger.focus();
  }
}
