// The story's sound: a score that follows the scene's mood, the sound of the place (rain, a crowd,
// waves, a clock), and effects at moments in the story (a door bell, a phone, the raid). All
// synthesized live with the Web Audio API, like the table's music (audio.ts), whose output it shares;
// the table's music rests while a scene plays. The score follows the story-music setting; the place
// and the effects follow the sound setting.

import { audio } from "./audio";

export type Mood = "noir" | "tension" | "sad" | "dawn" | "silence";
export type Ambience = "rain" | "crowd" | "waves" | "hum" | "pool" | "clock" | "city" | "car" | "birds" | "none";
export type Sfx =
  | "bell" | "phone" | "buzz" | "knock" | "crash" | "sirens" | "footsteps" | "foghorn" | "lighter" | "glass" | "elevator" | "applause" | "card" | "cuffs" | "sting" | "thunder";

const midi = (n: number) => 440 * Math.pow(2, (n - 69) / 12);
const rand = (lo: number, hi: number) => lo + Math.random() * (hi - lo);

interface Chord {
  root: number;
  notes: number[];
}
type Phrase = [number, number, number][];

// Noir: a slow minor blues in D, brushes and upright bass, Rhodes chords and a muted trumpet.
const NOIR: Chord[] = [
  { root: 38, notes: [53, 57, 60, 64] }, // Dm9
  { root: 38, notes: [53, 57, 60, 64] },
  { root: 43, notes: [58, 62, 65, 69] }, // Gm9
  { root: 43, notes: [58, 62, 65, 69] },
  { root: 46, notes: [57, 62, 65, 69] }, // B♭maj7
  { root: 45, notes: [55, 58, 61, 64] }, // A7♭9
  { root: 38, notes: [53, 57, 60, 64] },
  { root: 45, notes: [55, 58, 61, 64] },
];
const TRUMPET: Record<number, Phrase> = {
  1: [[0.67, 69, 0.33], [1, 72, 1], [2, 69, 0.67], [2.67, 67, 0.33], [3, 65, 1.6]],
  3: [[1, 62, 0.67], [1.67, 65, 0.33], [2, 67, 0.67], [2.67, 68, 0.33], [3, 69, 2.2]],
  5: [[0, 74, 1.6], [1.67, 72, 0.33], [2, 69, 1], [3, 67, 0.67], [3.67, 65, 1.6]],
  7: [[0.67, 65, 0.33], [1, 62, 2.8]],
};
// Sad: piano arpeggios in A minor.
const SAD: { root: number; arp: number[] }[] = [
  { root: 45, arp: [52, 57, 59, 60, 64, 60, 59, 57] },
  { root: 41, arp: [48, 53, 57, 60, 64, 60, 57, 53] },
  { root: 48, arp: [52, 55, 60, 62, 64, 62, 60, 55] },
  { root: 40, arp: [52, 56, 59, 62, 64, 62, 59, 56] },
];
const SAD_MELODY: Phrase[] = [
  [[0, 76, 2], [2, 74, 1], [3, 72, 1]],
  [[0, 72, 3], [3, 69, 1]],
  [[0, 67, 2], [2, 72, 2]],
  [[0, 71, 4]],
];
// Dawn: warm and hopeful, in F.
const DAWN: { root: number; arp: number[] }[] = [
  { root: 41, arp: [53, 57, 60, 64, 65, 64, 60, 57] },
  { root: 40, arp: [52, 55, 60, 62, 64, 62, 60, 55] },
  { root: 38, arp: [50, 53, 57, 60, 62, 60, 57, 53] },
  { root: 46, arp: [53, 57, 60, 62, 65, 62, 60, 57] },
];
const DAWN_MELODY: Phrase[] = [
  [[0, 72, 1], [1, 74, 1], [2, 76, 2]],
  [[0, 79, 3], [3, 77, 1]],
  [[0, 76, 2], [2, 74, 2]],
  [[0, 72, 4]],
];

const BEAT: Record<Exclude<Mood, "silence">, number> = { noir: 60 / 64, tension: 60 / 92, sad: 60 / 58, dawn: 60 / 76 };

class StoryAudio {
  private ctx: AudioContext | null = null;
  private out!: GainNode;
  private musicOut!: GainNode;
  private sfxOut!: GainNode;
  private ambOut!: GainNode;
  private reverb!: ConvolverNode;
  private noiseBuffer!: AudioBuffer;
  private timer: number | null = null;
  private session = 0;
  private mood: Mood = "silence";
  private moodGain: GainNode | null = null;
  private bar = 0;
  private nextBar = 0;
  private amb: { kind: Ambience; gain: GainNode; sources: AudioScheduledSourceNode[]; event?: (at: number) => number } | null = null;
  private nextEvent = 0;
  private musicOn = true;
  private soundOn = true;

  /** A scene begins: the table's music rests, and the story's sound starts up. */
  start() {
    const o = audio.output;
    if (!o) return;
    this.session++;
    audio.rest(true);
    if (this.ctx !== o.ctx) this.build(o.ctx, o.out);
    const ctx = this.ctx!;
    this.out.gain.cancelScheduledValues(ctx.currentTime);
    this.out.gain.setTargetAtTime(1, ctx.currentTime, 0.1);
    if (this.timer === null) this.timer = window.setInterval(() => this.tick(), 200);
  }

  /** The scene is over: everything fades out, and the table's music comes back. */
  stop() {
    const ctx = this.ctx;
    audio.rest(false);
    if (!ctx) return;
    const session = this.session;
    this.out.gain.cancelScheduledValues(ctx.currentTime);
    this.out.gain.setTargetAtTime(0, ctx.currentTime, 0.25);
    window.setTimeout(() => {
      if (session !== this.session) return; // another scene started meanwhile
      if (this.timer !== null) window.clearInterval(this.timer);
      this.timer = null;
      this.setMood("silence");
      this.setAmbience("none");
    }, 1500);
  }

  setMusic(on: boolean) {
    this.musicOn = on;
    if (this.ctx) this.musicOut.gain.setTargetAtTime(on ? 0.34 : 0, this.ctx.currentTime, 0.2);
  }

  setSound(on: boolean) {
    this.soundOn = on;
    if (!this.ctx) return;
    this.sfxOut.gain.setTargetAtTime(on ? 0.9 : 0, this.ctx.currentTime, 0.1);
    this.ambOut.gain.setTargetAtTime(on ? 0.9 : 0, this.ctx.currentTime, 0.3);
  }

  private build(ctx: AudioContext, master: GainNode) {
    this.ctx = ctx;
    this.out = ctx.createGain();
    this.out.gain.value = 0;
    this.out.connect(master);
    // A hall's reverb: two seconds and a half of decaying noise.
    this.reverb = ctx.createConvolver();
    const length = Math.floor(ctx.sampleRate * 2.6);
    const ir = ctx.createBuffer(2, length, ctx.sampleRate);
    for (let ch = 0; ch < 2; ch++) {
      const data = ir.getChannelData(ch);
      for (let i = 0; i < length; i++) data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / length, 3);
    }
    this.reverb.buffer = ir;
    const wet = ctx.createGain();
    wet.gain.value = 0.32;
    this.reverb.connect(wet).connect(this.out);
    this.musicOut = ctx.createGain();
    this.musicOut.gain.value = this.musicOn ? 0.34 : 0;
    this.musicOut.connect(this.out);
    this.musicOut.connect(this.reverb);
    this.sfxOut = ctx.createGain();
    this.sfxOut.gain.value = this.soundOn ? 0.9 : 0;
    this.sfxOut.connect(this.out);
    const send = ctx.createGain();
    send.gain.value = 0.4;
    this.sfxOut.connect(send).connect(this.reverb);
    this.ambOut = ctx.createGain();
    this.ambOut.gain.value = this.soundOn ? 0.9 : 0;
    this.ambOut.connect(this.out);
    this.noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 2, ctx.sampleRate);
    const noise = this.noiseBuffer.getChannelData(0);
    for (let i = 0; i < noise.length; i++) noise[i] = Math.random() * 2 - 1;
  }

  private tick() {
    const ctx = this.ctx;
    if (!ctx) return;
    if (this.mood !== "silence" && this.moodGain) {
      while (this.nextBar < ctx.currentTime + 1.2) {
        this.scheduleBar(this.mood, this.bar++, this.nextBar, this.moodGain);
        this.nextBar += 4 * BEAT[this.mood];
      }
    }
    if (this.amb?.event) {
      while (this.nextEvent < ctx.currentTime + 0.6) this.nextEvent += this.amb.event(Math.max(this.nextEvent, ctx.currentTime + 0.05));
    }
  }

  // ---- The score ---------------------------------------------------------------------------

  /** Changes the score's mood, crossfading from the last. */
  setMood(mood: Mood) {
    const ctx = this.ctx;
    if (!ctx || mood === this.mood) return;
    this.mood = mood;
    const old = this.moodGain;
    if (old) {
      old.gain.cancelScheduledValues(ctx.currentTime);
      old.gain.setTargetAtTime(0, ctx.currentTime, 0.5);
      window.setTimeout(() => old.disconnect(), 4000);
    }
    this.moodGain = null;
    if (mood === "silence") return;
    const gain = ctx.createGain();
    gain.gain.value = 0;
    gain.gain.setTargetAtTime(1, ctx.currentTime + 0.3, 0.4);
    gain.connect(this.musicOut);
    this.moodGain = gain;
    this.bar = 0;
    this.nextBar = ctx.currentTime + 0.3;
  }

  private scheduleBar(mood: Mood, bar: number, at: number, bus: GainNode) {
    if (mood === "noir") this.noirBar(bar, at, bus);
    else if (mood === "tension") this.tensionBar(bar, at, bus);
    else if (mood === "sad") this.arpBar(bar, at, bus, SAD, SAD_MELODY, BEAT.sad, false);
    else if (mood === "dawn") this.arpBar(bar, at, bus, DAWN, DAWN_MELODY, BEAT.dawn, true);
  }

  private noirBar(bar: number, at: number, bus: GainNode) {
    const beat = BEAT.noir;
    const place = bar % 8;
    const c = NOIR[place];
    const next = NOIR[(place + 1) % 8];
    const b = (n: number) => at + n * beat;
    // Upright bass: the root, the fifth, then a step into the next chord.
    this.bassNote(b(0), midi(c.root), 1.9 * beat, 0.13, bus);
    this.bassNote(b(2), midi(c.root + 7 > 50 ? c.root - 5 : c.root + 7), 0.95 * beat, 0.1, bus);
    this.bassNote(b(3), midi(next.root - 1), 0.9 * beat, 0.09, bus);
    // Rhodes: the chord held, with a soft push on the and of two every other bar.
    c.notes.forEach((n, i) => this.rhodes(b(0) + i * 0.018, midi(n), 3 * beat, 0.03, bus));
    if (bar % 2 === 1) c.notes.forEach((n) => this.rhodes(b(1.67), midi(n), beat, 0.014, bus));
    // Brushes and ride, swung.
    this.kick(b(0), 0.05, bus);
    for (const n of [1, 3]) this.brush(b(n), 0.035, bus);
    for (const n of [0, 1, 2, 3]) this.burst(b(n), 0.22, 0.014, "highpass", 7000, 0.7, bus, 3);
    for (const n of [1.67, 3.67]) this.burst(b(n), 0.15, 0.009, "highpass", 7000, 0.7, bus, 3);
    // The trumpet, every other time round; the Rhodes answers in between.
    const phrase = TRUMPET[place];
    if (phrase && Math.floor(bar / 8) % 2 === 0) for (const [n, note, len] of phrase) this.trumpet(b(n), midi(note), len * beat, 0.04, bus);
    else if (phrase && place % 4 === 1) for (const [n, note, len] of phrase) this.rhodes(b(n), midi(note + 12), len * beat + 0.5, 0.022, bus);
  }

  private tensionBar(bar: number, at: number, bus: GainNode) {
    const beat = BEAT.tension;
    const b = (n: number) => at + n * beat;
    if (bar % 2 === 0) {
      this.pad(b(0), [midi(26), midi(33)], 8 * beat + 0.6, 0.06, bus, 320);
      const high = [[69, 70], [72, 73], [69, 70], [74, 75]][(bar / 2) % 4];
      for (const n of high) this.strings(b(0), midi(n), 8 * beat, 0.011, bus);
    }
    // The ostinato: low, insistent eighths.
    const pattern = [38, 38, 41, 38, 40, 38, 36, 37];
    pattern.forEach((n, i) => this.bassNote(b(i / 2), midi(n), 0.3, 0.06, bus));
    // A heartbeat.
    this.kick(b(0), 0.1, bus);
    this.kick(b(0) + 0.28, 0.065, bus);
    if (bar % 4 === 3) {
      this.timpani(b(2), midi(26), 0.12, bus);
      for (const n of [26, 38, 39]) this.piano(b(2), midi(n), 2.5, 0.05, bus);
    }
  }

  private arpBar(bar: number, at: number, bus: GainNode, chords: { root: number; arp: number[] }[], melody: Phrase[], beat: number, warm: boolean) {
    const c = chords[bar % 4];
    this.piano(at, midi(c.root - 12), 4 * beat, 0.08, bus);
    c.arp.forEach((n, i) => this.piano(at + (i * beat) / 2, midi(n), 1.8, warm ? 0.038 : 0.045, bus));
    this.pad(at, c.arp.slice(0, 4).map(midi), 4 * beat + 0.4, 0.016, bus, 1200);
    if (Math.floor(bar / 4) % 2 === 1) {
      for (const [n, note, len] of melody[bar % 4]) {
        if (warm) this.rhodes(at + n * beat, midi(note), len * beat + 0.4, 0.045, bus);
        else this.piano(at + n * beat, midi(note), len * beat + 1, 0.055, bus);
      }
    }
  }

  // ---- The place ---------------------------------------------------------------------------

  /** The sound of the place, looping until the place changes. */
  setAmbience(kind: Ambience) {
    const ctx = this.ctx;
    if (!ctx || this.amb?.kind === kind || (!this.amb && kind === "none")) return;
    const old = this.amb;
    if (old) {
      old.gain.gain.cancelScheduledValues(ctx.currentTime);
      old.gain.gain.setTargetAtTime(0, ctx.currentTime, 0.4);
      window.setTimeout(() => {
        for (const s of old.sources) s.stop();
        old.gain.disconnect();
      }, 2500);
    }
    this.amb = null;
    if (kind === "none") return;
    const gain = ctx.createGain();
    gain.gain.value = 0;
    gain.gain.setTargetAtTime(1, ctx.currentTime, 0.5);
    gain.connect(this.ambOut);
    const sources: AudioScheduledSourceNode[] = [];
    const loop = (filter: BiquadFilterType, freq: number, q: number, level: number, wobble?: [number, number]) => {
      const src = ctx.createBufferSource();
      src.buffer = this.noiseBuffer;
      src.loop = true;
      src.loopStart = rand(0, 1);
      const f = ctx.createBiquadFilter();
      f.type = filter;
      f.frequency.value = freq;
      f.Q.value = q;
      const g = ctx.createGain();
      g.gain.value = level;
      if (wobble) {
        const lfo = ctx.createOscillator();
        lfo.frequency.value = wobble[0];
        const depth = ctx.createGain();
        depth.gain.value = wobble[1];
        lfo.connect(depth).connect(g.gain);
        lfo.start();
        sources.push(lfo);
      }
      src.connect(f).connect(g).connect(gain);
      src.start(ctx.currentTime, rand(0, 1.5));
      sources.push(src);
    };
    const hum = (freq: number, level: number, type: OscillatorType = "sine", cutoff = 20000) => {
      const osc = ctx.createOscillator();
      osc.type = type;
      osc.frequency.value = freq;
      const f = ctx.createBiquadFilter();
      f.type = "lowpass";
      f.frequency.value = cutoff;
      const g = ctx.createGain();
      g.gain.value = level;
      osc.connect(f).connect(g).connect(gain);
      osc.start();
      sources.push(osc);
    };
    let event: ((at: number) => number) | undefined;
    switch (kind) {
      case "rain":
        loop("highpass", 900, 0.5, 0.016);
        loop("lowpass", 420, 0.7, 0.012);
        break;
      case "car":
        loop("highpass", 1200, 0.5, 0.008);
        loop("lowpass", 300, 0.7, 0.01);
        hum(41, 0.02, "sawtooth", 110);
        break;
      case "crowd":
        loop("bandpass", 480, 0.7, 0.03, [0.23, 0.012]);
        loop("bandpass", 950, 1, 0.016, [0.37, 0.008]);
        event = (at) => {
          this.clink(at, 0.35, gain);
          return rand(3, 9);
        };
        break;
      case "waves":
        loop("lowpass", 520, 0.6, 0.035, [0.09, 0.03]);
        loop("highpass", 3000, 0.5, 0.006, [0.09, 0.005]);
        break;
      case "hum":
        hum(60, 0.007);
        hum(120, 0.004);
        loop("lowpass", 180, 0.7, 0.012);
        break;
      case "pool":
        hum(60, 0.01);
        loop("bandpass", 450, 0.6, 0.012, [0.2, 0.005]);
        event = (at) => {
          this.tone(at, 3200, 0.05, 0.04, "sine", gain);
          this.tone(at + 0.003, 2600, 0.05, 0.03, "sine", gain);
          if (Math.random() < 0.6) this.tone(at + 0.16, 2900, 0.04, 0.015, "sine", gain);
          return rand(4, 10);
        };
        break;
      case "clock": {
        let tock = false;
        event = (at) => {
          this.burst(at, 0.025, 0.06, "bandpass", tock ? 1900 : 2600, 6, gain, 4);
          tock = !tock;
          return 1;
        };
        loop("lowpass", 200, 0.5, 0.006);
        break;
      }
      case "city":
        loop("lowpass", 350, 0.6, 0.03, [0.07, 0.015]);
        event = (at) => {
          if (Math.random() < 0.5) {
            this.tone(at, 415, 0.45, 0.006, "square", gain, 1000);
            this.tone(at, 523, 0.45, 0.005, "square", gain, 1000);
          }
          return rand(9, 22);
        };
        break;
      case "birds":
        loop("lowpass", 300, 0.5, 0.005);
        event = (at) => {
          const n = 2 + Math.floor(Math.random() * 3);
          const base = rand(2400, 3200);
          for (let i = 0; i < n; i++) this.chirp(at + i * 0.11, base, 0.016, gain);
          return rand(1.8, 5.5);
        };
        break;
    }
    this.amb = { kind, gain, sources, event };
    this.nextEvent = ctx.currentTime + 0.5;
  }

  // ---- Effects -----------------------------------------------------------------------------

  sfx(name: Sfx) {
    const ctx = this.ctx;
    if (!ctx) return;
    const at = ctx.currentTime + 0.02;
    const bus = this.sfxOut;
    switch (name) {
      case "bell":
        [[0, 2350, 0.05], [0.11, 2100, 0.04], [0.24, 2350, 0.03], [0.42, 2100, 0.02]].forEach(([t, f, l]) => this.bell(at + t, f, 1.4, l, bus));
        break;
      case "phone":
        for (const ring of [0, 1.7]) for (let i = 0; i < 18; i++) this.bell(at + ring + i * 0.05, i % 2 ? 1500 : 1300, 0.1, 0.055, bus);
        break;
      case "buzz":
        for (const t of [0, 0.5]) {
          this.tone(at + t, 160, 0.35, 0.05, "square", bus, 420, 0.02);
          this.tone(at + t, 163, 0.35, 0.03, "square", bus, 420, 0.02);
        }
        break;
      case "knock":
        for (const t of [0, 0.22, 0.44]) {
          this.burst(at + t, 0.09, 0.3, "lowpass", 900, 0.7, bus);
          this.tone(at + t, 140, 0.09, 0.15, "sine", bus);
        }
        break;
      case "crash":
        this.drop(at, 72, 34, 0.5, 0.35, bus);
        this.burst(at, 0.9, 0.4, "lowpass", 1500, 0.6, bus, 1.5);
        for (let i = 0; i < 6; i++) this.burst(at + rand(0, 0.3), 0.06, 0.12, "bandpass", rand(2500, 4500), 2, bus);
        for (let i = 0; i < 24; i++) this.tone(at + rand(0.02, 0.6), rand(3000, 6500), rand(0.05, 0.2), 0.015, "sine", bus);
        break;
      case "sirens":
        this.siren(at, 5.5, 0, bus);
        this.siren(at + 0.5, 5, 30, bus);
        break;
      case "footsteps":
        for (let i = 0; i < 5; i++) {
          this.burst(at + i * 0.46, 0.09, 0.22, "lowpass", 600, 0.7, bus);
          this.burst(at + i * 0.46 + 0.01, 0.03, 0.06, "bandpass", 2200, 2, bus);
        }
        break;
      case "foghorn":
        this.horn(at, 92, 3.2, 0.06, bus);
        break;
      case "lighter":
        this.burst(at, 0.02, 0.2, "highpass", 3000, 0.7, bus, 4);
        this.burst(at + 0.06, 0.06, 0.1, "bandpass", 4200, 2, bus);
        this.burst(at + 0.1, 0.7, 0.06, "bandpass", 900, 0.5, bus, 1);
        break;
      case "glass":
        this.clink(at, 1, bus);
        this.clink(at + 0.13, 0.8, bus);
        break;
      case "elevator":
        this.bell(at, 1319, 2.2, 0.05, bus);
        this.burst(at + 0.5, 1.4, 0.05, "lowpass", 300, 0.7, bus, 1);
        break;
      case "applause":
        for (let i = 0; i < 7; i++) this.burst(at + i * 0.58 + rand(-0.02, 0.02), 0.05, 0.28, "bandpass", rand(1200, 1600), 1, bus);
        break;
      case "card":
        this.burst(at, 0.07, 0.35, "bandpass", 2500, 0.9, bus, 3);
        break;
      case "cuffs":
        for (let i = 0; i < 8; i++) this.burst(at + i * 0.035, 0.02, 0.12, "bandpass", 3600, 6, bus, 4);
        this.bell(at + 0.32, 2600, 0.4, 0.02, bus);
        break;
      case "sting":
        // A title card's hit: a low piano octave and a dark chord, a timpani, a cymbal's wash.
        for (const n of [26, 38]) this.piano(at, midi(n), 4, 0.13, bus);
        for (const n of [50, 53, 57, 58]) this.piano(at + 0.01, midi(n), 3.5, 0.035, bus);
        this.timpani(at, midi(26), 0.16, bus);
        this.burst(at, 3.2, 0.035, "highpass", 4500, 0.6, bus, 1.4);
        break;
      case "thunder":
        this.burst(at, 0.35, 0.15, "highpass", 1500, 0.6, bus, 2);
        this.burst(at + 0.1, 3.5, 0.45, "lowpass", 220, 0.6, bus, 1.2);
        break;
    }
  }

  // ---- Instruments -------------------------------------------------------------------------

  private tone(at: number, freq: number, length: number, level: number, type: OscillatorType, bus: AudioNode, cutoff = 20000, attack = 0.005) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    osc.type = type;
    osc.frequency.value = freq;
    const filter = ctx.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = cutoff;
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0, at);
    gain.gain.linearRampToValueAtTime(level, at + attack);
    gain.gain.exponentialRampToValueAtTime(0.0001, at + length);
    osc.connect(filter).connect(gain).connect(bus);
    osc.start(at);
    osc.stop(at + length + 0.05);
  }

  /** A held note: rises over `attack`, holds, and dies away over `release`. */
  private held(at: number, freq: number, length: number, level: number, type: OscillatorType, bus: AudioNode, attack: number, release: number, detune = 0) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    osc.type = type;
    osc.frequency.value = freq;
    osc.detune.value = detune;
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0, at);
    gain.gain.linearRampToValueAtTime(level, at + attack);
    gain.gain.setValueAtTime(level, at + Math.max(attack, length - release));
    gain.gain.linearRampToValueAtTime(0, at + length);
    osc.connect(gain).connect(bus);
    osc.start(at);
    osc.stop(at + length + 0.05);
    return osc;
  }

  private burst(at: number, length: number, level: number, type: BiquadFilterType, freq: number, q: number, bus: AudioNode, curve = 2) {
    const ctx = this.ctx!;
    const src = ctx.createBufferSource();
    src.buffer = this.noiseBuffer;
    const filter = ctx.createBiquadFilter();
    filter.type = type;
    filter.frequency.value = freq;
    filter.Q.value = q;
    const gain = ctx.createGain();
    const steps = 32;
    const shape = new Float32Array(steps);
    for (let i = 0; i < steps; i++) shape[i] = level * Math.pow(1 - i / (steps - 1), curve);
    gain.gain.setValueCurveAtTime(shape, at, length);
    src.connect(filter).connect(gain).connect(bus);
    src.start(at, rand(0, 1.4), length + 0.05);
  }

  private piano(at: number, freq: number, length: number, level: number, bus: AudioNode) {
    this.tone(at, freq, length, level, "triangle", bus, Math.min(5000, freq * 6));
    this.tone(at, freq * 2, length * 0.5, level * 0.25, "sine", bus);
  }

  private rhodes(at: number, freq: number, length: number, level: number, bus: AudioNode) {
    this.tone(at, freq, length, level, "sine", bus);
    this.tone(at, freq * 2, length * 0.6, level * 0.18, "sine", bus);
    this.tone(at, freq * 7, 0.18, level * 0.08, "sine", bus);
  }

  private bassNote(at: number, freq: number, length: number, level: number, bus: AudioNode) {
    this.tone(at, freq, length, level, "sine", bus, 20000, 0.01);
    this.tone(at, freq * 2, Math.min(length, 0.22), level * 0.3, "triangle", bus, 800);
  }

  private kick(at: number, level: number, bus: AudioNode) {
    this.drop(at, 90, 45, 0.2, level, bus);
  }

  /** A sine falling in pitch: kicks, thuds and the body of a crash. */
  private drop(at: number, from: number, to: number, length: number, level: number, bus: AudioNode) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    osc.frequency.setValueAtTime(from, at);
    osc.frequency.exponentialRampToValueAtTime(to, at + length * 0.7);
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(level, at);
    gain.gain.exponentialRampToValueAtTime(0.0001, at + length);
    osc.connect(gain).connect(bus);
    osc.start(at);
    osc.stop(at + length + 0.05);
  }

  private brush(at: number, level: number, bus: AudioNode) {
    this.burst(at, 0.32, level, "bandpass", 3400, 0.6, bus, 1);
  }

  private timpani(at: number, freq: number, level: number, bus: AudioNode) {
    this.drop(at, freq * 1.06, freq * 0.94, 1.8, level, bus);
    this.burst(at, 0.25, level * 0.6, "lowpass", 220, 0.7, bus);
  }

  /** A slow pad: detuned saws through a low-pass filter. */
  private pad(at: number, freqs: number[], length: number, level: number, bus: AudioNode, cutoff: number) {
    const ctx = this.ctx!;
    const filter = ctx.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = cutoff;
    filter.connect(bus);
    for (const f of freqs) for (const d of [-7, 7]) this.held(at, f, length, level / 2, "sawtooth", filter, Math.min(1.2, length / 3), Math.min(1.5, length / 3), d);
  }

  /** Strings, trembling: two detuned saws with a fast tremolo. */
  private strings(at: number, freq: number, length: number, level: number, bus: AudioNode) {
    const ctx = this.ctx!;
    const filter = ctx.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = 2400;
    const trem = ctx.createGain();
    trem.gain.value = 0.6;
    const lfo = ctx.createOscillator();
    lfo.frequency.value = 9;
    const depth = ctx.createGain();
    depth.gain.value = 0.4;
    lfo.connect(depth).connect(trem.gain);
    lfo.start(at);
    lfo.stop(at + length + 0.1);
    trem.connect(filter).connect(bus);
    for (const d of [-6, 6]) this.held(at, freq, length, level, "sawtooth", trem, length * 0.4, length * 0.3, d);
  }

  /** A muted trumpet: a saw through a narrow band, breathing in, with a late vibrato. */
  private trumpet(at: number, freq: number, length: number, level: number, bus: AudioNode) {
    const ctx = this.ctx!;
    const end = at + length;
    const osc = ctx.createOscillator();
    osc.type = "sawtooth";
    osc.frequency.value = freq;
    const lfo = ctx.createOscillator();
    lfo.frequency.value = 5.3;
    const depth = ctx.createGain();
    depth.gain.setValueAtTime(0, at);
    depth.gain.linearRampToValueAtTime(freq * 0.009, at + Math.min(0.4, length));
    lfo.connect(depth).connect(osc.frequency);
    const band = ctx.createBiquadFilter();
    band.type = "bandpass";
    band.frequency.value = 1150;
    band.Q.value = 2.4;
    const low = ctx.createBiquadFilter();
    low.type = "lowpass";
    low.frequency.value = 2600;
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0, at);
    gain.gain.linearRampToValueAtTime(level, at + 0.07);
    gain.gain.linearRampToValueAtTime(level * 0.8, end);
    gain.gain.linearRampToValueAtTime(0, end + 0.12);
    osc.connect(band).connect(low).connect(gain).connect(bus);
    for (const o of [osc, lfo]) {
      o.start(at);
      o.stop(end + 0.15);
    }
  }

  /** A struck bell: the note and two bright, inharmonic partials. */
  private bell(at: number, freq: number, length: number, level: number, bus: AudioNode) {
    this.tone(at, freq, length, level, "sine", bus);
    this.tone(at, freq * 2.76, length * 0.5, level * 0.35, "sine", bus);
    this.tone(at, freq * 5.4, length * 0.25, level * 0.15, "sine", bus);
  }

  private clink(at: number, level: number, bus: AudioNode) {
    const f = rand(2900, 3400);
    this.tone(at, f, 0.5, 0.035 * level, "sine", bus);
    this.tone(at, f * 1.48, 0.3, 0.018 * level, "sine", bus);
  }

  private chirp(at: number, base: number, level: number, bus: AudioNode) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    osc.frequency.setValueAtTime(base, at);
    osc.frequency.exponentialRampToValueAtTime(base * 1.45, at + 0.06);
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0, at);
    gain.gain.linearRampToValueAtTime(level, at + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, at + 0.08);
    osc.connect(gain).connect(bus);
    osc.start(at);
    osc.stop(at + 0.1);
  }

  /** A police siren wailing up and down, far off and coming closer. */
  private siren(at: number, length: number, detune: number, bus: AudioNode) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    osc.type = "triangle";
    osc.detune.value = detune;
    for (let t = 0; t < length; t += 1.4) {
      osc.frequency.setValueAtTime(650, at + t);
      osc.frequency.linearRampToValueAtTime(950, at + t + 0.7);
      osc.frequency.linearRampToValueAtTime(650, at + t + 1.4);
    }
    const filter = ctx.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = 2200;
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0, at);
    gain.gain.linearRampToValueAtTime(0.035, at + 1.6);
    gain.gain.setValueAtTime(0.035, at + length - 1);
    gain.gain.linearRampToValueAtTime(0, at + length);
    osc.connect(filter).connect(gain).connect(bus);
    osc.start(at);
    osc.stop(at + length + 0.1);
  }

  /** A ship's horn, far out in the fog. */
  private horn(at: number, freq: number, length: number, level: number, bus: AudioNode) {
    const ctx = this.ctx!;
    const filter = ctx.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = 380;
    filter.connect(bus);
    for (const [f, d] of [[freq, 0], [freq, 9], [freq * 1.5, 0]] as const) this.held(at, f, length, f === freq ? level : level * 0.4, "sawtooth", filter, 0.7, 1.3, d);
  }
}

export const storyAudio = new StoryAudio();
