import type { QuizRate, Settings, Speed } from "../settings";
import { Dialog } from "./Dialog";

interface Props {
  settings: Settings;
  onChange: (settings: Settings) => void;
  onClose: () => void;
}

type Toggle = "grade" | "trumpCounter" | "tracker" | "voids" | "quiz" | "sound" | "music" | "fourColor" | "bigCards" | "pauseAfterBidding";

const COACH: { key: Toggle; label: string; detail: string }[] = [
  { key: "grade", label: "Grade my decisions", detail: "After each bid and card, see how it compares with the expert's choice, and undo mistakes." },
  { key: "trumpCounter", label: "Trump counter", detail: "Show how many trumps you haven't seen yet." },
  { key: "tracker", label: "Card tracker", detail: "List every card played this hand and how many of each suit you haven't seen. Turn it off to practice counting." },
  { key: "voids", label: "Show who is out of a suit", detail: "Mark each player with the suits they've failed to follow. A player who throws off is out of that suit for the rest of the hand." },
  { key: "quiz", label: "Counting quiz", detail: "Now and then during play, answer a quick question: how many trumps are still out, who is out of a suit. The aids above are hidden while you answer." },
];

const SOUND: { key: Toggle; label: string; detail: string }[] = [
  { key: "sound", label: "Sound effects", detail: "Cards, bids, tricks and results." },
  { key: "music", label: "Music", detail: "Quiet background music." },
];

const DISPLAY: { key: Toggle; label: string; detail: string }[] = [
  {
    key: "fourColor",
    label: "Four-color deck",
    detail: "Diamonds blue and clubs green, so every suit has its own color. Bowers still pair hearts with diamonds and spades with clubs.",
  },
  { key: "bigCards", label: "Larger cards", detail: "Bigger cards and text on the table." },
  {
    key: "pauseAfterBidding",
    label: "Pause after the bidding",
    detail: "When the bidding ends, show everyone's calls and wait for Continue before the kitty and play.",
  },
];

/** Everything you can change about the game, in one place. Changes apply straight away. */
export function SettingsDialog({ settings, onChange, onClose }: Props) {
  const toggles = (list: typeof COACH) =>
    list.map(({ key, label, detail }) => (
      <label key={key} className="check">
        <input type="checkbox" checked={settings[key]} onChange={(e) => onChange({ ...settings, [key]: e.target.checked })} />
        <span>
          {label}
          <small>{detail}</small>
        </span>
      </label>
    ));
  return (
    <Dialog title="Settings" onClose={onClose}>
      <form className="new-game settings" onSubmit={(e) => e.preventDefault()}>
        <h3>Coach</h3>
        {toggles(COACH)}
        <fieldset disabled={!settings.quiz}>
          <legend>How often the counting quiz asks</legend>
          {(
            [
              ["low", "rarely"],
              ["medium", "sometimes"],
              ["high", "often"],
            ] as [QuizRate, string][]
          ).map(([rate, label]) => (
            <label key={rate} className="inline">
              <input type="radio" name="quizRate" checked={settings.quizRate === rate} onChange={() => onChange({ ...settings, quizRate: rate })} />
              {label}
            </label>
          ))}
        </fieldset>
        <p className="note">The Hint button is always there on your turn.</p>
        <h3>Sound</h3>
        {toggles(SOUND)}
        <h3>Speed</h3>
        <fieldset>
          <legend>How fast the bots play (games on this device)</legend>
          {(["slow", "normal", "fast"] as Speed[]).map((speed) => (
            <label key={speed} className="inline">
              <input type="radio" name="speed" checked={settings.speed === speed} onChange={() => onChange({ ...settings, speed })} />
              {speed}
            </label>
          ))}
        </fieldset>
        <h3>Display</h3>
        {toggles(DISPLAY)}
        <div className="actions">
          <button type="button" className="primary" onClick={onClose}>
            Done
          </button>
        </div>
      </form>
    </Dialog>
  );
}
