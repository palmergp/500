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
    this.musicBus.gain.value = 0.28; // about as loud as the old lounge loop (measured offline)
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

  // ---- Music: a gentle bossa nova -----------------------------------------------------------

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

  /** One bar: bass and soft kick, nylon-guitar comping on the two-bar bossa pattern, rim clicks on
   * the clave, a shaker on the eighths, and now and then a vibraphone phrase. */
  private scheduleBar(at: number, bar: number) {
    const bus = this.musicBus!;
    const chord = PROGRESSION[bar % PROGRESSION.length];
    const next = PROGRESSION[(bar + 1) % PROGRESSION.length];
    const second = bar % 2 === 1;
    const e = (eighth: number) => at + eighth * EIGHTH;

    // Bass: root on 1, the fifth on the and of 2 held through 3, the root again on the and of 4.
    const fifth = chord.root + 7 > 50 ? chord.root - 5 : chord.root + 7;
    this.bass(e(0), midi(chord.root), 3 * EIGHTH, 0.13, bus);
    this.bass(e(3), midi(fifth), 4 * EIGHTH, 0.11, bus);
    this.bass(e(7), midi(chord.root), EIGHTH, 0.08, bus);
    for (const [eighth, level] of [[0, 0.07], [3, 0.04], [4, 0.06], [7, 0.035]] as const) this.kick(e(eighth), level, bus);

    // Guitar: the first bar of each pair on 1, the and of 2, and 4; the second on 2, the and of 3,
    // and the and of 4, which anticipates the next bar's chord.
    const hits: [number, Chord, number][] = second
      ? [[2, chord, 0.032], [5, chord, 0.028], [7, next, 0.034]]
      : [[0, chord, 0.036], [3, chord, 0.028], [6, chord, 0.03]];
    for (const [eighth, c, level] of hits) this.strum(e(eighth), c.notes, level, bus);

    // Cross-stick on the bossa clave (3 + 3 + 4 + 3 + 3 eighths over the two bars).
    for (const eighth of second ? [2, 5] : [0, 3, 6]) this.rim(e(eighth), 0.05, bus);
    // Shaker: every eighth, the offbeats a little louder.
    for (let i = 0; i < 8; i++) this.shaker(e(i), i % 2 ? 0.035 : 0.02, bus);

    // Vibraphone: a short phrase every other bar from the chord's tones, varied each pass.
    const pass = Math.floor(bar / PROGRESSION.length) % MOTIFS.length;
    const motif = MOTIFS[pass][bar % 4];
    for (const [eighth, tone, length] of motif) {
      const n = chord.notes[tone] + 12;
      this.vibes(e(eighth), midi(n), length * EIGHTH, 0.032, bus);
    }
  }

  /** A plucked string: a triangle wave through a low-pass filter, with a quick, soft decay. */
  private pluck(at: number, freq: number, length: number, level: number, bus: GainNode, cutoff = 2200) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    osc.type = "triangle";
    osc.frequency.value = freq;
    const filter = ctx.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.setValueAtTime(cutoff, at);
    filter.frequency.exponentialRampToValueAtTime(cutoff * 0.35, at + length);
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0, at);
    gain.gain.linearRampToValueAtTime(level, at + 0.004);
    gain.gain.exponentialRampToValueAtTime(0.0001, at + length);
    osc.connect(filter).connect(gain).connect(bus);
    osc.start(at);
    osc.stop(at + length + 0.05);
  }

  /** Nylon-string guitar: the chord's notes strummed upward with the thumb, a few ms apart. */
  private strum(at: number, notes: number[], level: number, bus: GainNode) {
    notes.forEach((n, i) => this.pluck(at + i * 0.012, midi(n), 0.42, level, bus));
  }

  /** Upright bass: a round sine with a little plucked edge. */
  private bass(at: number, freq: number, length: number, level: number, bus: GainNode) {
    this.tone(at, freq, length, level, "sine", bus);
    this.pluck(at, freq * 2, Math.min(length, 0.25), level * 0.25, bus, 900);
  }

  /** A soft kick (the surdo's part): a sine falling in pitch. */
  private kick(at: number, level: number, bus: GainNode) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    osc.frequency.setValueAtTime(95, at);
    osc.frequency.exponentialRampToValueAtTime(48, at + 0.12);
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(level, at);
    gain.gain.exponentialRampToValueAtTime(0.0001, at + 0.18);
    osc.connect(gain).connect(bus);
    osc.start(at);
    osc.stop(at + 0.2);
  }

  /** A short burst of filtered noise. */
  private noise(at: number, length: number, level: number, type: BiquadFilterType, freq: number, q: number, bus: GainNode) {
    const ctx = this.ctx!;
    const buffer = ctx.createBuffer(1, Math.ceil(ctx.sampleRate * length), ctx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < data.length; i++) data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / data.length, 2);
    const src = ctx.createBufferSource();
    src.buffer = buffer;
    const filter = ctx.createBiquadFilter();
    filter.type = type;
    filter.frequency.value = freq;
    filter.Q.value = q;
    const gain = ctx.createGain();
    gain.gain.value = level;
    src.connect(filter).connect(gain).connect(bus);
    src.start(at);
  }

  /** Cross-stick on the snare's rim: a woody click. */
  private rim(at: number, level: number, bus: GainNode) {
    this.noise(at, 0.035, level, "bandpass", 1900, 4, bus);
    this.tone(at, 820, 0.04, level * 0.5, "triangle", bus);
  }

  private shaker(at: number, level: number, bus: GainNode) {
    this.noise(at, 0.05, level, "highpass", 6500, 0.7, bus);
  }

  /** Vibraphone: a sine with a soft bell partial and a slow fade. */
  private vibes(at: number, freq: number, length: number, level: number, bus: GainNode) {
    this.tone(at, freq, length + 0.6, level, "sine", bus);
    this.tone(at, freq * 4, 0.3, level * 0.12, "sine", bus);
  }
}

const BEAT = 60 / 128;
const EIGHTH = BEAT / 2;
const BAR = 4 * BEAT;
const midi = (n: number) => 440 * Math.pow(2, (n - 69) / 12);

interface Chord {
  /** The bass note (MIDI). */
  root: number;
  /** The guitar's voicing, low to high (MIDI), the root left to the bass. */
  notes: number[];
}

// Eight bars in F: Fmaj9 – F6/9 – Gm9 – C13 – Am7 – D7♭9 – Gm9 – C7♭9.
const PROGRESSION: Chord[] = [
  { root: 41, notes: [57, 60, 64, 67] },
  { root: 41, notes: [57, 62, 64, 67] },
  { root: 43, notes: [58, 62, 65, 69] },
  { root: 48, notes: [58, 62, 64, 69] },
  { root: 45, notes: [55, 60, 64, 67] },
  { root: 50, notes: [54, 57, 60, 63] },
  { root: 43, notes: [58, 62, 65, 69] },
  { root: 48, notes: [52, 55, 58, 61] },
];

/** Vibraphone phrases, by bar within four: [eighth, which chord tone (0 lowest), length in eighths]. */
type Phrase = [number, number, number][];
const MOTIFS: Phrase[][] = [
  [[[1, 3, 2], [3, 2, 1], [4, 3, 4]], [], [[2, 1, 2], [4, 2, 2], [6, 3, 2]], []],
  [[[0, 2, 3], [3, 3, 3]], [[1, 1, 6]], [[3, 3, 1], [4, 2, 1], [5, 1, 3]], []],
];

export const audio = new AudioEngine();
export type { Effect };
