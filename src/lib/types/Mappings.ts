export interface KeyMapping {
    key: string;          
    midiNote: number;     
    noteName?: string;    
}

export interface MappingConfig {
    id: string;
    name: string;         
    description?: string;
    mappings: KeyMapping[];
    octaveShift: number;
}

export const DEFAULT_CONFIGS: MappingConfig[] = [
    {
        id: 'heartopia-37',
        name: 'Heartopia 37',
        description: 'Heartopia 37 key layout',
        octaveShift: 0,
        mappings: [
            // Верхний ряд (ноты с точками - высокая октава)
            { key: 'Q', midiNote: 72 },  // İ (DO высокая)
            { key: 'W', midiNote: 62 },  // Ż (RE)
            { key: 'E', midiNote: 64 },  // Ż (MI)
            { key: 'R', midiNote: 65 },  // 4̇ (FA высокая)
            { key: 'T', midiNote: 67 },  // 5̇ (SOL высокая)
            { key: 'Y', midiNote: 69 },  // 6̇ (LA высокая)
            { key: 'U', midiNote: 71 },  // 7̇ (SI высокая)
            { key: 'I', midiNote: 72 },  // İ (DO высокая)
            
            // Средний ряд (основные белые ноты)
            { key: 'Z', midiNote: 60 },  // 1 (DO)
            { key: 'X', midiNote: 62 },  // 2 (RE)
            { key: 'C', midiNote: 64 },  // 3 (MI)
            { key: 'V', midiNote: 65 },  // 4 (FA)
            { key: 'B', midiNote: 67 },  // 5 (SOL)
            { key: 'N', midiNote: 69 },  // 6 (LA)
            { key: 'M', midiNote: 71 },  // 7 (SI)
            
            // Нижний ряд (низкая октава)
            { key: ',', midiNote: 48 },  // 1̣ (DO низкая)
            { key: '.', midiNote: 50 },  // 2̣ (RE низкая)
            { key: '/', midiNote: 52 },  // 3̣ (MI низкая)
            { key: 'O', midiNote: 53 },  // 4̣ (FA низкая)
            { key: 'P', midiNote: 55 },  // 5̣ (SOL низкая)
            { key: '[', midiNote: 57 },  // 6̣ (LA низкая)
            { key: ']', midiNote: 59 },  // 7̣ (SI низкая)
            
            // Черные клавиши верхнего ряда (высокая октава)
            { key: '2', midiNote: 73 },  
            { key: '3', midiNote: 63 },  
            { key: '5', midiNote: 66 },  
            { key: '6', midiNote: 68 },  
            { key: '7', midiNote: 70 },  
            
            // Черные клавиши среднего ряда
            { key: 'S', midiNote: 61 },  
            { key: 'D', midiNote: 63 },  
            { key: 'G', midiNote: 66 },  
            { key: 'H', midiNote: 68 },  
            { key: 'J', midiNote: 70 },  
            
            // Черные клавиши нижнего ряда (низкая октава)
            { key: 'L', midiNote: 49 },  
            { key: ';', midiNote: 51 },  
            { key: '0', midiNote: 54 },  
            { key: '-', midiNote: 56 },  
            { key: '=', midiNote: 58 },  
        ]
    },
    {
        id: 'heartopia-15',
        name: 'Heartopia 15',
        description: 'Heartopia 2 octave layout',
        octaveShift: 0,
        mappings: [
            // Верхний ряд
            { key: 'Q', midiNote: 60 },  // C
            { key: 'W', midiNote: 62 },  // D
            { key: 'E', midiNote: 64 },  // E
            { key: 'R', midiNote: 65 },  // F
            { key: 'T', midiNote: 67 },  // G

            // Средний ряд
            { key: 'Y', midiNote: 69 },  // A
            { key: 'U', midiNote: 71 },  // B
            { key: 'I', midiNote: 72 },  // C
            { key: 'A', midiNote: 74 },  // D
            { key: 'S', midiNote: 76 },  // E

            // Нижний ряд
            { key: 'D', midiNote: 65 },  // F
            { key: 'F', midiNote: 67 },  // G
            { key: 'G', midiNote: 69 },  // A
            { key: 'H', midiNote: 71 },  // B
            { key: 'J', midiNote: 72 },  // C
        ]
    }
];