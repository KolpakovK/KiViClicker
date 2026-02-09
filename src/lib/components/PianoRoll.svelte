<script lang="ts">
    import { onMount, tick } from "svelte";
    import type { MappingConfig } from "$lib/types/Mappings";

    interface MidiNote {
        note: number;
        velocity: number;
        time_ms: number;
        duration_ms: number;
    }
    
    interface MidiTrack {
        index: number;
        name: string;
        note_count: number;
        notes: MidiNote[];
    }

    interface MidiFileInfo {
        file_name: string;
        duration_ms: number;
        tempo_bpm: number;
        tracks: MidiTrack[];
    }

    let {
        midiData,
        selectedTrackIndex,
        playbackTime = 0,
        mappingConfig
    }: {
        midiData: MidiFileInfo | null;
        selectedTrackIndex: number;
        playbackTime?: number;
        mappingConfig?: MappingConfig | null;
    } = $props();

    let containerWidth = $state(0);
    let containerRef: HTMLDivElement;
    let scrollContainer: HTMLDivElement;
    let isLoading = $state(false);

    const PIANO_KEY_WIDTH = 80;
    const NOTE_HEIGHT = 12;
    const MIN_NOTE = 21;
    const MAX_NOTE = 108;
    const MIN_PIXELS_PER_SECOND = 50;
    
    let scrollLeft = $state(0);
    let scrollWidth = $state(0);
    const RENDER_PADDING = 200;

    let mappedNotes = $derived.by(() => {
        if (!mappingConfig) return new Set<number>();
        return new Set(mappingConfig.mappings.map(m => m.midiNote));
    });

    let selectedTrack = $derived.by(() => {
        if (!midiData || selectedTrackIndex === null || selectedTrackIndex < 0) return null;
        return midiData.tracks[selectedTrackIndex];
    });

    let noteRange = $derived.by(() => {
        if (!selectedTrack || selectedTrack.notes.length === 0) {
            return { min: 48, max: 84 };
        }
        const notes = selectedTrack.notes.map(n => n.note);
        return {
            min: Math.max(MIN_NOTE, Math.min(...notes) - 2),
            max: Math.min(MAX_NOTE, Math.max(...notes) + 2)
        };
    });

    let totalNotes = $derived(noteRange.max - noteRange.min + 1);
    let svgHeight = $derived(totalNotes * NOTE_HEIGHT);
    
    let minRollWidth = $derived.by(() => {
        if (!midiData) return 0;
        const durationSeconds = midiData.duration_ms / 1000;
        return durationSeconds * MIN_PIXELS_PER_SECOND;
    });

    let rollWidth = $derived(Math.max(containerWidth - PIANO_KEY_WIDTH, minRollWidth));
    let svgWidth = $derived(PIANO_KEY_WIDTH + rollWidth);
    
    let pixelsPerMs = $derived.by(() => {
        if (!midiData || rollWidth <= 0) return 0.1;
        return rollWidth / midiData.duration_ms;
    });

    let playheadX = $derived(PIANO_KEY_WIDTH + (playbackTime * pixelsPerMs));

    $effect(() => {
        if (playbackTime > 0 && scrollContainer) {
            const playheadPos = playbackTime * pixelsPerMs;
            const scrollPos = scrollContainer.scrollLeft;
            const visibleWidth = scrollContainer.clientWidth - PIANO_KEY_WIDTH;
            
            if (playheadPos < scrollPos || playheadPos > scrollPos + visibleWidth) {
                scrollContainer.scrollLeft = Math.max(0, playheadPos - visibleWidth / 2);
            }
        }
    });

    let visibleTimeRange = $derived.by(() => {
        const startMs = Math.max(0, (scrollLeft - RENDER_PADDING) / pixelsPerMs);
        const endMs = Math.min(
            midiData?.duration_ms || 0, 
            (scrollLeft + scrollWidth + RENDER_PADDING) / pixelsPerMs
        );
        return { startMs, endMs };
    });

    let timeMarkers = $derived.by(() => {
        if (!midiData) return [];
        const markers = [];
        const totalSeconds = Math.ceil(midiData.duration_ms / 1000);
        for (let i = 0; i <= totalSeconds; i++) {
            markers.push({
                time: i * 1000,
                x: PIANO_KEY_WIDTH + (i * 1000 * pixelsPerMs),
                label: `${i}s`
            });
        }
        return markers;
    });

    let displayNotes = $derived.by(() => {
        if (!selectedTrack) return [];
        
        const { startMs, endMs } = visibleTimeRange;
        
        return selectedTrack.notes
            .filter(note => {
                const noteEndMs = note.time_ms + note.duration_ms;
                return noteEndMs >= startMs && note.time_ms <= endMs;
            })
            .map(note => {
                const isActive = playbackTime >= note.time_ms && playbackTime <= (note.time_ms + note.duration_ms);
                const isMapped = mappedNotes.has(note.note);
                return {
                    ...note,
                    y: (noteRange.max - note.note) * NOTE_HEIGHT,
                    x: PIANO_KEY_WIDTH + (note.time_ms * pixelsPerMs),
                    width: Math.max(2, note.duration_ms * pixelsPerMs),
                    height: NOTE_HEIGHT - 1,
                    isActive,
                    isMapped
                };
            });
    });

    function getNoteName(midiNote: number): string {
        const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
        const octave = Math.floor(midiNote / 12) - 1;
        const noteName = noteNames[midiNote % 12];
        return `${noteName}${octave}`;
    }

    function isBlackKey(note: number): boolean {
        const blackKeys = [1, 3, 6, 8, 10];
        return blackKeys.includes(note % 12);
    }

    function getNoteColor(velocity: number, isActive: boolean, isMapped: boolean): string {
        if (isActive) {
            const alpha = 0.8 + (velocity / 127) * 0.2;
            return `hsla(120, 80%, 60%, ${alpha})`;
        }
        
        if (isMapped) {
            const alpha = 0.7 + (velocity / 127) * 0.3;
            return `hsla(280, 70%, 65%, ${alpha})`;
        }
        
        const alpha = 0.4 + (velocity / 127) * 0.2;
        return `hsla(0, 0%, 50%, ${alpha})`;
    }

    function getNoteStroke(isActive: boolean, isMapped: boolean): string {
        if (isActive) {
            return 'hsla(120, 80%, 70%, 1)';
        }
        if (isMapped) {
            return 'hsla(280, 70%, 60%, 0.8)';
        }
        return 'hsla(0, 0%, 40%, 0.5)';
    }

    function handleScroll(e: Event) {
        const target = e.target as HTMLDivElement;
        scrollLeft = target.scrollLeft;
        scrollWidth = target.clientWidth;
    }

    let prevTrackIndex = $state(selectedTrackIndex);
    $effect(() => {
        if (selectedTrackIndex !== prevTrackIndex) {
            isLoading = true;
            prevTrackIndex = selectedTrackIndex;
            
            setTimeout(async () => {
                await tick();
                isLoading = false;
                
                if (scrollContainer) {
                    scrollContainer.scrollLeft = 0;
                }
            }, 50);
        }
    });

    onMount(() => {
        const resizeObserver = new ResizeObserver((entries) => {
            for (let entry of entries) {
                containerWidth = entry.contentRect.width;
            }
        });

        if (containerRef) {
            resizeObserver.observe(containerRef);
            containerWidth = containerRef.clientWidth;
        }

        if (scrollContainer) {
            scrollWidth = scrollContainer.clientWidth;
        }

        return () => {
            resizeObserver.disconnect();
        };
    });
</script>

<div bind:this={containerRef} class="w-full border border-border rounded-md bg-background overflow-hidden">
    {#if selectedTrack}
        <div class="p-4">
            <div class="flex items-center justify-between mb-2">
                <div>
                    <h3 class="text-sm font-semibold">{selectedTrack.name}</h3>
                    <p class="text-xs text-muted-foreground">
                        {selectedTrack.note_count} notes | 
                        Range: {getNoteName(noteRange.min)} - {getNoteName(noteRange.max)}
                    </p>
                </div>
                <div class="flex items-center gap-3">
                    {#if mappingConfig}
                        <div class="flex items-center gap-2 text-xs">
                            <div class="flex items-center gap-1">
                                <div class="w-3 h-3 rounded-sm" style="background: hsla(280, 70%, 65%, 0.8);"></div>
                                <span class="text-muted-foreground">Mapped</span>
                            </div>
                            <div class="flex items-center gap-1">
                                <div class="w-3 h-3 rounded-sm" style="background: hsla(0, 0%, 50%, 0.5);"></div>
                                <span class="text-muted-foreground">Unmapped</span>
                            </div>
                        </div>
                    {/if}
                    <div class="text-xs text-muted-foreground">
                        Duration: {Math.floor(midiData!.duration_ms / 1000 / 60)}:{String(Math.floor((midiData!.duration_ms / 1000) % 60)).padStart(2, '0')}
                    </div>
                </div>
            </div>
            
            {#if isLoading}
                <div class="h-50 flex items-center justify-center border border-border rounded bg-black">
                    <p class="text-muted-foreground text-sm">Loading track...</p>
                </div>
            {:else}
                <div 
                    bind:this={scrollContainer}
                    class="overflow-auto border border-border rounded bg-black" 
                    style="max-height: 200px;"
                    onscroll={handleScroll}
                >
                    <svg width={svgWidth} height={svgHeight} class="block">
                        
                        <rect width={svgWidth} height={svgHeight} fill="#0a0a0a" />
                        
                        {#each Array(totalNotes) as _, i}
                            {@const note = noteRange.max - i}
                            {@const y = i * NOTE_HEIGHT}
                            {@const isC = note % 12 === 0}
                            <line 
                                x1={PIANO_KEY_WIDTH} 
                                y1={y} 
                                x2={svgWidth} 
                                y2={y} 
                                stroke={isC ? '#333333' : '#1a1a1a'} 
                                stroke-width={isC ? 1.5 : 0.5} 
                            />
                        {/each}

                        {#each timeMarkers as marker}
                            <line 
                                x1={marker.x} 
                                y1={0} 
                                x2={marker.x} 
                                y2={svgHeight} 
                                stroke="#1a1a1a" 
                                stroke-width={0.5} 
                            />
                            {#if marker.time % 5000 === 0}
                                <text 
                                    x={marker.x + 2} 
                                    y={12} 
                                    fill="#666666" 
                                    font-size="10" 
                                    font-family="monospace"
                                >
                                    {marker.label}
                                </text>
                            {/if}
                        {/each}

                        <g>
                            <rect 
                                x={0} 
                                y={0} 
                                width={PIANO_KEY_WIDTH} 
                                height={svgHeight} 
                                fill="#0a0a0a" 
                            />
                            
                            {#each Array(totalNotes) as _, i}
                                {@const note = noteRange.max - i}
                                {@const y = i * NOTE_HEIGHT}
                                {@const isBlack = isBlackKey(note)}
                                {@const isC = note % 12 === 0}
                                {@const isMapped = mappedNotes.has(note)}
                                
                                <rect 
                                    x={0} 
                                    y={y} 
                                    width={PIANO_KEY_WIDTH} 
                                    height={NOTE_HEIGHT} 
                                    fill={isBlack ? '#2a2a2a' : '#fafafa'} 
                                    stroke={isMapped ? '#9333ea' : '#666666'}
                                    stroke-width={isMapped ? 1.5 : 0.5} 
                                />
                                
                                {#if isC}
                                    <text 
                                        x={PIANO_KEY_WIDTH / 2} 
                                        y={y + NOTE_HEIGHT / 2} 
                                        fill={isMapped ? '#9333ea' : '#000000'}
                                        font-size="10" 
                                        font-family="monospace" 
                                        text-anchor="middle" 
                                        dominant-baseline="middle"
                                        font-weight={isMapped ? 'bold' : 'normal'}
                                    >
                                        {getNoteName(note)}
                                    </text>
                                {/if}
                            {/each}
                        </g>

                        {#each displayNotes as note (note.time_ms + '-' + note.note)}
                            <rect 
                                x={note.x} 
                                y={note.y} 
                                width={note.width} 
                                height={note.height} 
                                fill={getNoteColor(note.velocity, note.isActive, note.isMapped)} 
                                stroke={getNoteStroke(note.isActive, note.isMapped)}
                                stroke-width={note.isActive ? 1.5 : (note.isMapped ? 1 : 0.5)} 
                                rx={2}
                            />
                        {/each}

                        {#if playbackTime > 0}
                            <line 
                                x1={playheadX} 
                                y1={0} 
                                x2={playheadX} 
                                y2={svgHeight} 
                                stroke="#ff4444" 
                                stroke-width={2} 
                                opacity={0.8}
                            />
                            <path
                                d="M {playheadX - 6} 0 L {playheadX + 6} 0 L {playheadX} 8 Z"
                                fill="#ff4444"
                                opacity={0.8}
                            />
                        {/if}
                    </svg>
                </div>
            {/if}
        </div>
    {:else}
        <div class="p-8 text-center text-muted-foreground">
            <p>No track selected</p>
        </div>
    {/if}
</div>