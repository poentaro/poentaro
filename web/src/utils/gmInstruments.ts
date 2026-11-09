export type InstrumentCategory =
  | 'piano'
  | 'chromatic-perc'
  | 'organ'
  | 'guitar'
  | 'bass'
  | 'strings'
  | 'ensemble'
  | 'brass'
  | 'reed'
  | 'pipe'
  | 'synth-lead'
  | 'synth-pad'
  | 'synth-fx'
  | 'ethnic'
  | 'percussive'
  | 'sound-fx'

export type InstrumentSource = 'acoustic' | 'synth'
export type InstrumentRole = 'melody' | 'bass' | 'pad' | 'rhythm'
export type InstrumentArticulation =
  | 'plucked'
  | 'bowed'
  | 'blown'
  | 'struck'
  | 'electronic'
  | 'vocal'

/** One GM instrument catalog entry, passed to every Rust call that reads
 * `parts` sound labels (as the WIT `instrument-info`). `name` is the bare
 * instrument name; Rust alone builds the quoted `"N: Name"` label from it. */
export interface InstrumentEntry {
  name: string
  program: number
  category: InstrumentCategory
  source: InstrumentSource
  role: InstrumentRole
  articulation: InstrumentArticulation
}

interface RawInstrumentEntry {
  name: string
  category: InstrumentCategory
  source: InstrumentSource
  role: InstrumentRole
  articulation: InstrumentArticulation
}

// GM program numbers are the table's index (0–127), not a hand-typed field.
// biome-ignore format: large data table — one entry per line for readability
const RAW_GM_INSTRUMENTS: RawInstrumentEntry[] = [
  // Piano (0–7)
  { name: 'Acoustic Grand Piano', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Bright Acoustic Piano', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Electric Grand Piano', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Honky-tonk Piano', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Electric Piano 1', category: 'piano', source: 'synth', role: 'melody', articulation: 'struck' },
  { name: 'Electric Piano 2', category: 'piano', source: 'synth', role: 'melody', articulation: 'struck' },
  { name: 'Harpsichord', category: 'piano', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Clavi', category: 'piano', source: 'synth', role: 'melody', articulation: 'struck' },
  // Chromatic Perc (8–15)
  { name: 'Celesta', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Glockenspiel', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Music Box', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Vibraphone', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Marimba', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Xylophone', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Tubular Bells', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Dulcimer', category: 'chromatic-perc', source: 'acoustic', role: 'melody', articulation: 'struck' },
  // Organ (16–23)
  { name: 'Drawbar Organ', category: 'organ', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Percussive Organ', category: 'organ', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Rock Organ', category: 'organ', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Church Organ', category: 'organ', source: 'acoustic', role: 'pad', articulation: 'blown' },
  { name: 'Reed Organ', category: 'organ', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Accordion', category: 'organ', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Harmonica', category: 'organ', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Tango Accordion', category: 'organ', source: 'acoustic', role: 'melody', articulation: 'blown' },
  // Guitar (24–31)
  { name: 'Acoustic Guitar (nylon)', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Acoustic Guitar (steel)', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Electric Guitar (jazz)', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Electric Guitar (clean)', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Electric Guitar (muted)', category: 'guitar', source: 'acoustic', role: 'rhythm', articulation: 'plucked' },
  { name: 'Overdriven Guitar', category: 'guitar', source: 'synth', role: 'melody', articulation: 'plucked' },
  { name: 'Distortion Guitar', category: 'guitar', source: 'synth', role: 'melody', articulation: 'plucked' },
  { name: 'Guitar Harmonics', category: 'guitar', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  // Bass (32–39)
  { name: 'Acoustic Bass', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { name: 'Electric Bass (finger)', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { name: 'Electric Bass (pick)', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { name: 'Fretless Bass', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { name: 'Slap Bass 1', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { name: 'Slap Bass 2', category: 'bass', source: 'acoustic', role: 'bass', articulation: 'plucked' },
  { name: 'Synth Bass 1', category: 'bass', source: 'synth', role: 'bass', articulation: 'electronic' },
  { name: 'Synth Bass 2', category: 'bass', source: 'synth', role: 'bass', articulation: 'electronic' },
  // Strings (40–47)
  { name: 'Violin', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'bowed' },
  { name: 'Viola', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'bowed' },
  { name: 'Cello', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'bowed' },
  { name: 'Contrabass', category: 'strings', source: 'acoustic', role: 'bass', articulation: 'bowed' },
  { name: 'Tremolo Strings', category: 'strings', source: 'acoustic', role: 'pad', articulation: 'bowed' },
  { name: 'Pizzicato Strings', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Orchestral Harp', category: 'strings', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Timpani', category: 'strings', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  // Ensemble (48–55)
  { name: 'String Ensemble 1', category: 'ensemble', source: 'acoustic', role: 'pad', articulation: 'bowed' },
  { name: 'String Ensemble 2', category: 'ensemble', source: 'acoustic', role: 'pad', articulation: 'bowed' },
  { name: 'Synth Strings 1', category: 'ensemble', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Synth Strings 2', category: 'ensemble', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Choir Aahs', category: 'ensemble', source: 'acoustic', role: 'pad', articulation: 'vocal' },
  { name: 'Voice Oohs', category: 'ensemble', source: 'acoustic', role: 'pad', articulation: 'vocal' },
  { name: 'Synth Voice', category: 'ensemble', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Orchestra Hit', category: 'ensemble', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  // Brass (56–63)
  { name: 'Trumpet', category: 'brass', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Trombone', category: 'brass', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Tuba', category: 'brass', source: 'acoustic', role: 'bass', articulation: 'blown' },
  { name: 'Muted Trumpet', category: 'brass', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'French Horn', category: 'brass', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Brass Section', category: 'brass', source: 'acoustic', role: 'pad', articulation: 'blown' },
  { name: 'Synth Brass 1', category: 'brass', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Synth Brass 2', category: 'brass', source: 'synth', role: 'melody', articulation: 'electronic' },
  // Reed (64–71)
  { name: 'Soprano Sax', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Alto Sax', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Tenor Sax', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Baritone Sax', category: 'reed', source: 'acoustic', role: 'bass', articulation: 'blown' },
  { name: 'Oboe', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'English Horn', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Bassoon', category: 'reed', source: 'acoustic', role: 'bass', articulation: 'blown' },
  { name: 'Clarinet', category: 'reed', source: 'acoustic', role: 'melody', articulation: 'blown' },
  // Pipe (72–79)
  { name: 'Piccolo', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Flute', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Recorder', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Pan Flute', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Blown Bottle', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Shakuhachi', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Whistle', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Ocarina', category: 'pipe', source: 'acoustic', role: 'melody', articulation: 'blown' },
  // Synth Lead (80–87)
  { name: 'Lead 1 (square)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Lead 2 (sawtooth)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Lead 3 (calliope)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Lead 4 (chiff)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Lead 5 (charang)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Lead 6 (voice)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Lead 7 (fifths)', category: 'synth-lead', source: 'synth', role: 'melody', articulation: 'electronic' },
  { name: 'Lead 8 (bass + lead)', category: 'synth-lead', source: 'synth', role: 'bass', articulation: 'electronic' },
  // Synth Pad (88–95)
  { name: 'Pad 1 (new age)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Pad 2 (warm)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Pad 3 (polysynth)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Pad 4 (choir)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Pad 5 (bowed)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Pad 6 (metallic)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Pad 7 (halo)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'Pad 8 (sweep)', category: 'synth-pad', source: 'synth', role: 'pad', articulation: 'electronic' },
  // Synth FX (96–103)
  { name: 'FX 1 (rain)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'FX 2 (soundtrack)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'FX 3 (crystal)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'FX 4 (atmosphere)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'FX 5 (brightness)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'FX 6 (goblins)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'FX 7 (echoes)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  { name: 'FX 8 (sci-fi)', category: 'synth-fx', source: 'synth', role: 'pad', articulation: 'electronic' },
  // Ethnic (104–111)
  { name: 'Sitar', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Banjo', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Shamisen', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Koto', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Kalimba', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'plucked' },
  { name: 'Bag Pipe', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'blown' },
  { name: 'Fiddle', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'bowed' },
  { name: 'Shanai', category: 'ethnic', source: 'acoustic', role: 'melody', articulation: 'blown' },
  // Percussive (112–119)
  { name: 'Tinkle Bell', category: 'percussive', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Agogo', category: 'percussive', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  { name: 'Steel Drums', category: 'percussive', source: 'acoustic', role: 'melody', articulation: 'struck' },
  { name: 'Woodblock', category: 'percussive', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  { name: 'Taiko Drum', category: 'percussive', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  { name: 'Melodic Tom', category: 'percussive', source: 'acoustic', role: 'rhythm', articulation: 'struck' },
  { name: 'Synth Drum', category: 'percussive', source: 'synth', role: 'rhythm', articulation: 'electronic' },
  { name: 'Reverse Cymbal', category: 'percussive', source: 'synth', role: 'rhythm', articulation: 'electronic' },
  // Sound FX (120–127)
  { name: 'Guitar Fret Noise', category: 'sound-fx', source: 'acoustic', role: 'rhythm', articulation: 'plucked' },
  { name: 'Breath Noise', category: 'sound-fx', source: 'acoustic', role: 'rhythm', articulation: 'blown' },
  { name: 'Seashore', category: 'sound-fx', source: 'acoustic', role: 'pad', articulation: 'electronic' },
  { name: 'Bird Tweet', category: 'sound-fx', source: 'acoustic', role: 'melody', articulation: 'vocal' },
  { name: 'Telephone Ring', category: 'sound-fx', source: 'acoustic', role: 'rhythm', articulation: 'electronic' },
  { name: 'Helicopter', category: 'sound-fx', source: 'acoustic', role: 'pad', articulation: 'electronic' },
  { name: 'Applause', category: 'sound-fx', source: 'acoustic', role: 'pad', articulation: 'vocal' },
  { name: 'Gunshot', category: 'sound-fx', source: 'acoustic', role: 'rhythm', articulation: 'electronic' },
]

export const GM_INSTRUMENTS: InstrumentEntry[] = RAW_GM_INSTRUMENTS.map(
  (entry, program) => ({ ...entry, program }),
)
