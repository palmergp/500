import { useEffect, useRef, useState, type ReactNode } from "react";

/** A top-bar "More" button whose menu holds the buttons a small screen has no room for. The menu
 * closes on a choice, a tap outside or Escape. Items are ordinary buttons (`MoreItem`). */
export function MoreMenu({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const outside = (e: PointerEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    const escape = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", outside, true);
    document.addEventListener("keydown", escape);
    return () => {
      document.removeEventListener("pointerdown", outside, true);
      document.removeEventListener("keydown", escape);
    };
  }, [open]);
  return (
    <div className="more-menu" ref={ref}>
      <button type="button" className="nav-button" aria-label="More" aria-expanded={open} aria-haspopup="menu" title="More: sound, music, settings" onClick={() => setOpen(!open)}>
        <span aria-hidden="true">⋯</span>
      </button>
      {open && (
        // A choice closes the menu (the click reaches the item first).
        <div className="more-items" role="menu" aria-label="More" onClick={() => setOpen(false)}>
          {children}
        </div>
      )}
    </div>
  );
}

/** A button in the "More" menu: icon, label, and an optional state ("on", "off"). */
export function MoreItem({ icon, label, state, tip, onClick, off }: { icon: string; label: string; state?: string; tip: string; onClick: () => void; off?: boolean }) {
  return (
    <button type="button" role="menuitem" className={off ? "off" : ""} title={tip} onClick={onClick}>
      <span aria-hidden="true" className="more-icon">
        {icon}
      </span>
      {label}
      {state && <span className="more-state">{state}</span>}
    </button>
  );
}
