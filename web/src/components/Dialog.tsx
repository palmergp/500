import { useEffect, useRef, type ReactNode } from "react";

interface Props {
  title: string;
  children: ReactNode;
  onClose?: () => void;
  wide?: boolean;
}

/** A modal dialog built on <dialog>, so focus and Escape behave natively. */
export function Dialog({ title, children, onClose, wide }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const dialog = ref.current;
    if (dialog && !dialog.open) dialog.showModal();
    return () => dialog?.close();
  }, []);
  return (
    <dialog
      ref={ref}
      className={`dialog${wide ? " wide" : ""}`}
      aria-label={title}
      onCancel={(e) => {
        e.preventDefault();
        onClose?.();
      }}
    >
      <header>
        <h2>{title}</h2>
        {onClose && (
          <button type="button" className="icon" aria-label="Close" onClick={onClose}>
            ×
          </button>
        )}
      </header>
      <div className="dialog-body">{children}</div>
    </dialog>
  );
}
