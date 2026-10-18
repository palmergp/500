import { useState } from "react";
import type { Persona } from "../engine";
import type { Settings, Speed } from "../settings";
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
                {personas.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
              {chosen && <small>{chosen.blurb}</small>}
            </label>
          );
        })}
        <fieldset>
          <legend>Speed</legend>
          {(["slow", "normal", "fast"] as Speed[]).map((speed) => (
            <label key={speed} className="inline">
              <input
                type="radio"
                name="speed"
                checked={settings.speed === speed}
                onChange={() => setSettings({ ...settings, speed })}
              />
              {speed}
            </label>
          ))}
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
