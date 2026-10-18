// Sound effects and background music, synthesized live with the Web Audio API: no audio files to
// download or license. Browsers only allow audio after the player interacts with the page, so the
// context is created on first use from a click.

type Effect =
  | "card"
  | "deal"
  | "bid"
  | "pass"
  | "trickUs"
  | "trickThem"
  | "contract"
  | "made"
  | "set"
  | "win"
  | "lose"
  | "hint";

class AudioEngine {
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private sfxBus: GainNode | null = null;
  private musicBus: GainNode | null = null;
  private musicTimer: number | null = null;
  private nextBar = 0;
  private bar = 0;
  sfxOn = true;
  musicOn = false;

  /** Starts the audio context; call from a user gesture. */
  unlock() {
    if (this.ctx) {
      if (this.ctx.state === "suspended") void this.ctx.resume();
      return;
    }
    try {
      this.ctx = new AudioContext();
    } catch {
      return; // no audio support: stay silent
    }
    this.master = this.ctx.createGain();
    this.master.gain.value = 0.6;
    this.master.connect(this.ctx.destination);
    this.sfxBus = this.ctx.createGain();
    this.sfxBus.connect(this.master);
    this.musicBus = this.ctx.createGain();
    this.musicBus.gain.value = 0.22;
    this.musicBus.connect(this.master);
    if (this.musicOn) this.startMusic();
  }

  setVolume(v: number) {
    if (this.master) this.master.gain.value = v;
  }

  setMusic(on: boolean) {
    this.musicOn = on;
    if (on) this.startMusic();
    else this.stopMusic();
  }

  play(effect: Effect) {
    if (!this.sfxOn || !this.ctx || !this.sfxBus) return;
    const t = this.ctx.currentTime;
    switch (effect) {
      case "card":
        this.snap(t, 0.5);
        break;
      case "deal":
        for (let i = 0; i < 10; i++) this.snap(t + i * 0.06, 0.25 + Math.random() * 0.1);
        break;
      case "bid":
        this.tone(t, 660, 0.08, 0.12, "triangle");
        this.tone(t + 0.06, 880, 0.1, 0.1, "triangle");
        break;
      case "pass":
        this.tone(t, 330, 0.08, 0.07, "sine");
        break;
      case "trickUs":
        this.chord(t, [784, 988, 1175], 0.35, 0.09);
        break;
      case "trickThem":
        this.chord(t, [392, 466], 0.3, 0.06);
        break;
      case "contract":
        this.chord(t, [523, 659, 784], 0.4, 0.08);
        break;
      case "made":
        [523, 659, 784, 1047].forEach((f, i) => this.tone(t + i * 0.11, f, 0.35, 0.12, "triangle"));
        break;
      case "set":
        [392, 330, 262].forEach((f, i) => this.tone(t + i * 0.16, f, 0.4, 0.1, "sine"));
        break;
      case "win":
        [523, 659, 784, 1047, 784, 1047, 1319].forEach((f, i) => this.tone(t + i * 0.13, f, 0.45, 0.13, "triangle"));
        break;
      case "lose":
        [440, 415, 392, 330].forEach((f, i) => this.tone(t + i * 0.22, f, 0.6, 0.09, "sine"));
        break;
      case "hint":
        this.tone(t, 1319, 0.5, 0.06, "sine");
        this.tone(t + 0.08, 1760, 0.5, 0.04, "sine");
        break;
    }
  }

  // ---- Synthesis helpers ------------------------------------------------------------------

  private tone(at: number, freq: number, length: number, level: number, type: OscillatorType, bus?: GainNode) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = type;
    osc.frequency.value = freq;
    gain.gain.setValueAtTime(0, at);
    gain.gain.linearRampToValueAtTime(level, at + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, at + length);
    osc.connect(gain).connect(bus ?? this.sfxBus!);
    osc.start(at);
    osc.stop(at + length + 0.05);
  }

  private chord(at: number, freqs: number[], length: number, level: number) {
    freqs.forEach((f) => this.tone(at, f, length, level, "triangle"));
  }

  /** A card hitting the felt: a short burst of filtered noise. */
  private snap(at: number, level: number, bus?: GainNode) {
    const ctx = this.ctx!;
    const length = 0.07;
    const buffer = ctx.createBuffer(1, Math.ceil(ctx.sampleRate * length), ctx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < data.length; i++) data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / data.length, 3);
    const src = ctx.createBufferSource();
    src.buffer = buffer;
    const filter = ctx.createBiquadFilter();
    filter.type = "bandpass";
    filter.frequency.value = 2200 + Math.random() * 800;
    filter.Q.value = 0.9;
    const gain = ctx.createGain();
    gain.gain.value = level;
    src.connect(filter).connect(gain).connect(bus ?? this.sfxBus!);
    src.start(at);
  }

  // ---- Music: a slow jazz-lounge loop -----------------------------------------------------

  private startMusic() {
    if (!this.ctx || this.musicTimer !== null) return;
    this.nextBar = this.ctx.currentTime + 0.1;
    const schedule = () => {
      while (this.ctx && this.nextBar < this.ctx.currentTime + 1.5) {
        this.scheduleBar(this.nextBar, this.bar++);
        this.nextBar += BAR;
      }
    };
    schedule();
    this.musicTimer = window.setInterval(schedule, 400);
  }

  private stopMusic() {
    if (this.musicTimer !== null) window.clearInterval(this.musicTimer);
    this.musicTimer = null;
  }

  private scheduleBar(at: number, bar: number) {
    const bus = this.musicBus!;
    const chord = PROGRESSION[bar % PROGRESSION.length];
    // Soft electric-piano chord on beats 1 and 3 (a little later on 3 for swing).
    for (const beat of [0, 2.15]) {
      chord.notes.forEach((n) => this.epiano(at + beat * BEAT, midi(n), BEAT * 1.8, 0.05, bus));
    }
    // Walking bass: root, third, fifth, approach note.
    const walk = [chord.root, chord.root + 4 - (chord.minor ? 1 : 0), chord.root + 7, chord.root + 6];
    walk.forEach((n, i) => this.tone(at + i * BEAT, midi(n - 12), BEAT * 0.9, 0.09, "sine", bus));
    // Brushed hi-hat on the offbeats.
    for (let i = 0; i < 4; i++) this.snap(at + (i + 0.66) * BEAT, 0.06, bus);
    // An occasional melody note.
    if (bar % 2 === 1) {
      const top = chord.notes[chord.notes.length - 1] + 12;
      this.epiano(at + 1.5 * BEAT, midi(top), BEAT * 1.5, 0.035, bus);
      this.epiano(at + 3 * BEAT, midi(top - 3), BEAT, 0.03, bus);
    }
  }

  /** A mellow keys tone: two slightly detuned sines with a bell-like attack. */
  private epiano(at: number, freq: number, length: number, level: number, bus: GainNode) {
    this.tone(at, freq, length, level, "sine", bus);
    this.tone(at, freq * 1.003, length * 0.8, level * 0.6, "sine", bus);
    this.tone(at, freq * 2, length * 0.25, level * 0.25, "sine", bus);
  }
}

const BEAT = 60 / 76;
const BAR = 4 * BEAT;
const midi = (n: number) => 440 * Math.pow(2, (n - 69) / 12);

// Fmaj7 – Dm7 – Gm7 – C7 (I–vi–ii–V in F), voiced around middle C.
const PROGRESSION = [
  { root: 53, minor: false, notes: [57, 60, 64, 65] },
  { root: 50, minor: true, notes: [57, 60, 62, 65] },
  { root: 55, minor: true, notes: [58, 62, 65, 67] },
  { root: 48, minor: false, notes: [58, 60, 64, 67] },
];

export const audio = new AudioEngine();
export type { Effect };
