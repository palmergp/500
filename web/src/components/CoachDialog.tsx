import type { Settings } from "../settings";
import { Dialog } from "./Dialog";

interface Props {
  settings: Settings;
  onChange: (settings: Settings) => void;
  onClose: () => void;
}

const OPTIONS: { key: "grade" | "trumpCounter" | "tracker" | "voids" | "quiz"; label: string; detail: string }[] = [
  { key: "grade", label: "Grade my decisions", detail: "After each bid and card, see how it compares with the expert's choice, and undo mistakes." },
  { key: "trumpCounter", label: "Trump counter", detail: "Show how many trumps you haven't seen yet." },
  { key: "tracker", label: "Card tracker", detail: "List every card played this hand and how many of each suit you haven't seen. Turn it off to practice counting." },
  { key: "voids", label: "Show who is out of a suit", detail: "Mark each player with the suits they've failed to follow. A player who throws off is out of that suit for the rest of the hand." },
  { key: "quiz", label: "Counting quiz", detail: "Now and then during play, answer a quick question: how many trumps are still out, who is out of a suit. The aids above are hidden while you answer." },
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
        <h3>Display</h3>
        <label className="check">
          <input type="checkbox" checked={settings.fourColor} onChange={(e) => onChange({ ...settings, fourColor: e.target.checked })} />
          <span>
            Four-color deck
            <small>Diamonds blue and clubs green, so every suit has its own color. Bowers still pair hearts with diamonds and spades with clubs.</small>
          </span>
        </label>
        <label className="check">
          <input type="checkbox" checked={settings.bigCards} onChange={(e) => onChange({ ...settings, bigCards: e.target.checked })} />
          <span>
            Larger cards
            <small>Bigger cards and text on the table.</small>
          </span>
        </label>
        <div className="actions">
          <button type="button" className="primary" onClick={onClose}>
            Done
          </button>
        </div>
      </form>
    </Dialog>
  );
}
