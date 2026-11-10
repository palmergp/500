import { useState } from "react";
import type { Persona } from "../engine";
import type { Settings } from "../settings";
import { Dialog } from "./Dialog";

interface Props {
  personas: Persona[];
  initial: Settings;
  onStart: (settings: Settings) => void;
  onCancel?: () => void;
}

const SEATS: { key: "north" | "west" | "east"; label: string }[] = [
  { key: "north", label: "Your partner (North)" },
  { key: "west", label: "Left opponent (West)" },
  { key: "east", label: "Right opponent (East)" },
];

export const RANDOM_BLURB = "A different player each game, picked at random from the list.";

/** The game's length: to 500, or a short game of four hands. */
export const LENGTHS: { hands: number; label: string; detail: string }[] = [
  { hands: 0, label: "First to 500", detail: "The usual game: win by making a bid that takes you to 500. −500 loses." },
  { hands: 4, label: "4 hands", detail: "Play four hands; the higher score wins. A tie plays one more hand." },
];

export function NewGameDialog({ personas, initial, onStart, onCancel }: Props) {
  const [settings, setSettings] = useState(initial);
  return (
    <Dialog title="New game" onClose={onCancel}>
      <form
        className="new-game"
        onSubmit={(e) => {
          e.preventDefault();
          onStart(settings);
        }}
      >
        {SEATS.map(({ key, label }) => {
          const chosen = personas.find((p) => p.id === settings[key]);
          return (
            <label key={key}>
              <span>{label}</span>
              <select value={settings[key]} onChange={(e) => setSettings({ ...settings, [key]: e.target.value })}>
                <option value="any">🎲 Random</option>
                {personas.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
              <small>{chosen ? chosen.blurb : RANDOM_BLURB}</small>
            </label>
          );
        })}
        <fieldset>
          <legend>Game</legend>
          {LENGTHS.map(({ hands, label, detail }) => (
            <label key={hands} className="inline" title={detail}>
              <input type="radio" name="hands" checked={settings.hands === hands} onChange={() => setSettings({ ...settings, hands })} />
              {label}
            </label>
          ))}
          <small>{LENGTHS.find((l) => l.hands === settings.hands)?.detail}</small>
        </fieldset>
        <div className="actions">
          {onCancel && (
            <button type="button" onClick={onCancel}>
              Cancel
            </button>
          )}
          <button type="submit" className="primary">
            Deal
          </button>
        </div>
      </form>
    </Dialog>
  );
}
