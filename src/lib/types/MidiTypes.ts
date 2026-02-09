export interface MidiNote {
    note: number;
    velocity: number;
    time_ms: number;
    duration_ms: number;
}

export interface MidiTrack {
    index: number;
    name: string;
    note_count: number;
    notes: MidiNote[];
}

export interface MidiFileInfo {
    file_name: string;
    duration_ms: number;
    tempo_bpm: number;
    tracks: MidiTrack[];
}