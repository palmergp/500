import { useEffect, useLayoutEffect, useRef, useState } from "react";

/**
 * Tooltips for touch screens, where nothing hovers. Press and hold anything with a tooltip (a
 * `title`: a card's numbers after a hint, a bid's make chance, a top-bar button) to read it; letting
 * go then doesn't press it. Plain labels that do nothing when tapped (a player's style, a void chip,
 * the trump count) show theirs on a tap. The next touch anywhere puts it away.
 */
const HOLD_MS = 450;
const SLOP = 10;
const INTERACTIVE = "button, a, input, select, textarea, label, [role=button]";

interface Tip {
  text: string;
  /** The element's box, to place the bubble beside it. */
  box: DOMRect;
}

/** The tooltip for a touched element: the nearest `title` at or above it. */
function tipFor(target: EventTarget | null): { element: HTMLElement; text: string } | null {
  const element = target instanceof Element ? target.closest<HTMLElement>("[title]") : null;
  const text = element?.getAttribute("title")?.trim();
  return element && text ? { element, text } : null;
}

export function TouchTips() {
  const [tip, setTip] = useState<Tip | null>(null);
  const bubble = useRef<HTMLDivElement>(null);
  const [place, setPlace] = useState<{ left: number; top: number } | null>(null);

  useEffect(() => {
    let timer: number | undefined;
    let start: { x: number; y: number; id: number } | null = null;
    // After a hold showed a tip, the click that ends it must not press what's under the finger.
    let swallowClick = false;
    let shown = false;

    const cancel = () => {
      window.clearTimeout(timer);
      timer = undefined;
      start = null;
    };
    const show = (element: HTMLElement, text: string) => {
      shown = true;
      setTip({ text, box: element.getBoundingClientRect() });
    };
    const hide = () => {
      if (!shown) return;
      shown = false;
      setTip(null);
    };

    const down = (e: PointerEvent) => {
      if (e.pointerType === "mouse") return;
      hide();
      cancel();
      swallowClick = false;
      const found = tipFor(e.target);
      if (!found) return;
      start = { x: e.clientX, y: e.clientY, id: e.pointerId };
      timer = window.setTimeout(() => {
        timer = undefined;
        if (!start || !found.element.isConnected) return;
        swallowClick = true;
        show(found.element, found.element.getAttribute("title")?.trim() || found.text);
      }, HOLD_MS);
    };
    const move = (e: PointerEvent) => {
      if (start && e.pointerId === start.id && Math.hypot(e.clientX - start.x, e.clientY - start.y) > SLOP) cancel();
    };
    const up = (e: PointerEvent) => {
      if (!start || e.pointerId !== start.id) return;
      const held = timer === undefined;
      cancel();
      if (held) return;
      // A quick tap on a label that does nothing else shows its tip.
      const found = tipFor(e.target);
      if (found && !found.element.closest(INTERACTIVE)) show(found.element, found.text);
    };
    const click = (e: MouseEvent) => {
      if (!swallowClick) return;
      swallowClick = false;
      e.preventDefault();
      e.stopPropagation();
    };
    // A long press on iOS or Android would otherwise open the system's menu or select text.
    const menu = (e: Event) => {
      if (swallowClick || timer !== undefined) e.preventDefault();
    };

    document.addEventListener("pointerdown", down, true);
    document.addEventListener("pointermove", move, true);
    document.addEventListener("pointerup", up, true);
    document.addEventListener("pointercancel", cancel, true);
    document.addEventListener("click", click, true);
    document.addEventListener("contextmenu", menu, true);
    window.addEventListener("scroll", hide, true);
    window.addEventListener("resize", hide);
    return () => {
      cancel();
      document.removeEventListener("pointerdown", down, true);
      document.removeEventListener("pointermove", move, true);
      document.removeEventListener("pointerup", up, true);
      document.removeEventListener("pointercancel", cancel, true);
      document.removeEventListener("click", click, true);
      document.removeEventListener("contextmenu", menu, true);
      window.removeEventListener("scroll", hide, true);
      window.removeEventListener("resize", hide);
    };
  }, []);

  // Above the element if there's room, else below; kept on screen. A popover, so it shows above an
  // open dialog (dialogs sit in the browser's top layer).
  useLayoutEffect(() => {
    const el = bubble.current;
    if (!tip || !el) {
      setPlace(null);
      return;
    }
    try {
      if (!el.matches(":popover-open")) el.showPopover();
    } catch {
      // No popover support: it shows in the page.
    }
    const { width, height } = el.getBoundingClientRect();
    const margin = 8;
    const left = Math.min(Math.max(margin, tip.box.left + tip.box.width / 2 - width / 2), window.innerWidth - width - margin);
    const above = tip.box.top - height - margin;
    const top = above >= margin + 40 ? above : Math.min(tip.box.bottom + margin, window.innerHeight - height - margin);
    setPlace({ left, top });
  }, [tip]);

  if (!tip) return null;
  return (
    <div
      ref={bubble}
      className="touch-tip"
      popover="manual"
      role="tooltip"
      style={place ? { left: place.left, top: place.top } : { left: 0, top: 0, visibility: "hidden" }}
    >
      {tip.text}
    </div>
  );
}
