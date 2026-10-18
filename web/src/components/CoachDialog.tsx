import type { Settings } from "../settings";
import { Dialog } from "./Dialog";

interface Props {
  settings: Settings;
  onChange: (settings: Settings) => void;
  onClose: () => void;
}

const OPTIONS: { key: "grade" | "trumpCounter" | "tracker"; label: string; detail: string }[] = [
  { key: "grade", label: "Grade my decisions", detail: "After each bid and card, see how it compares with the expert's choice, and undo mistakes." },
  { key: "trumpCounter", label: "Trump counter", detail: "Show how many trumps you haven't seen yet." },
  { key: "tracker", label: "Card tracker", detail: "List every card played this hand. Turn it off to practise counting." },
];

export function CoachDialog({ settings, onChange, onClose }: Props) {
  return (
    <Dialog title="Coach" onClose={onClose}>
      <form className="new-game" onSubmit={(e) => e.preventDefault()}>
        {OPTIONS.map(({ key, label, detail }) => (
          <label key={key} className="check">
            <input type="checkbox" checked={settings[key]} onChange={(e) => onChange({ ...settings, [key]: e.target.checked })} />
            <span>
              {label}
              <small>{detail}</small>
            </span>
          </label>
        ))}
        <p className="note">The Hint button is always available on your turn.</p>
        <div className="actions">
          <button type="button" className="primary" onClick={onClose}>
            Done
          </button>
        </div>
      </form>
    </Dialog>
  );
}
