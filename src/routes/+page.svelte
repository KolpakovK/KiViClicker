<script lang="ts">
    import { listen } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/core';

    import AppNavigation from "$lib/components/AppNavigation.svelte";
    import PianoRoll from "$lib/components/PianoRoll.svelte";
    import MidiPlayer from "$lib/components/MidiPlayer.svelte";
    import { FrownIcon, PlayIcon, SquareIcon } from "@lucide/svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import type { MidiFileInfo } from "$lib/types/MidiTypes";
    import { DEFAULT_CONFIGS, type MappingConfig } from "$lib/types/Mappings";
    import { onMount } from "svelte";
    import { Button } from "$lib/components/ui/button";

    let midiData: MidiFileInfo | null = $state(null);
    let selectedTrackIndex = $state(0);
    let currentPlaybackTime = $state(0);
    let mappingConfig: MappingConfig = $state(DEFAULT_CONFIGS[0]);
    let isClickerRunning = $state(false)

    let currentTrack = $derived.by(() => {
        if (!midiData || !midiData.tracks || midiData.tracks.length === 0) return null;
        if (selectedTrackIndex < 0 || selectedTrackIndex >= midiData.tracks.length) return null;
        return midiData.tracks[selectedTrackIndex];
    });

    function transformDuration(duration_ms: number): string {
        const minutes = Math.floor(duration_ms / 60000);
        const seconds = Math.floor((duration_ms % 60000) / 1000);
        return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    }

    function handleMidiDataLoaded(data: MidiFileInfo | null) {
        midiData = data;
        selectedTrackIndex = 0;
        console.log('MIDI data received in page:', midiData);
    }

    function handleTimeUpdate(time: number) {
        currentPlaybackTime = time;
    }

    function handleMappingChange(config: MappingConfig) {
        mappingConfig = config;
        console.log('Mapping config changed:', config);
    }

    async function toggleClicker() {
        if (!currentTrack || !mappingConfig) {
            alert('Please select a track and mapping configuration first');
            return;
        }

        if (isClickerRunning) {
            await invoke('stop_clicker');
            isClickerRunning = false;
        } else {
            const noteEvents = prepareNoteEvents();
            
            try {
                await invoke('start_clicker', { notes: noteEvents });
                isClickerRunning = true;
            } catch (e) {
                console.error('Failed to start clicker:', e);
                alert('Failed to start clicker: ' + e);
            }
        }
    }

    function prepareNoteEvents() {
        if (!currentTrack || !mappingConfig) return [];
        
        const noteToKey = new Map(
            mappingConfig.mappings.map(m => [m.midiNote, m.key])
        );
        
        return currentTrack.notes
            .filter(note => noteToKey.has(note.note))
            .map(note => ({
                key: noteToKey.get(note.note)!,
                time_ms: Math.floor(note.time_ms),
                duration_ms: Math.floor(note.duration_ms)
            }))
            .sort((a, b) => a.time_ms - b.time_ms);
    }

    onMount(() => {
        const unlisten = listen('toggle-clicker', () => {
            toggleClicker();
        });

        return () => {
            unlisten.then(fn => fn());
        };
    });
</script>

<main class="h-screen w-screen flex flex-col gap-4 p-4 bg-muted">
    <AppNavigation onMidiDataLoaded={handleMidiDataLoaded} onMappingChange={handleMappingChange} />

    {#if midiData !== null && midiData.tracks.length > 0}
        <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
                <div class="flex flex-col gap-0.5">
                    <p class="text-lg text-foreground truncate max-w-md">{midiData.file_name}</p>
                    <p class="text-xs text-muted-foreground">{transformDuration(midiData.duration_ms)}</p>
                </div>

                <div class="flex items-center gap-2">
                    <Button 
                        size="sm" 
                        variant={isClickerRunning ? "destructive" : "default"}
                        onclick={toggleClicker}
                    >
                        {#if isClickerRunning}
                            <SquareIcon class="size-4" />
                            Stop Clicker
                        {:else}
                            <PlayIcon class="size-4" />
                            Start Clicker
                        {/if}
                    </Button>
                    <kbd class="px-2 py-1 text-xs bg-muted border border-border rounded font-mono">
                        Ctrl+M
                    </kbd>
                </div>
            </div>

            <div class="flex gap-4">
                <div class="flex flex-1 flex-col gap-1 p-3 rounded-md bg-background border border-border">
                    <p class="text-sm font-semibold text-foreground">Audio Player</p>
                    <MidiPlayer {midiData} {selectedTrackIndex} onTimeUpdate={handleTimeUpdate} />
                </div>

                <div class="flex flex-1 flex-col gap-1 p-3 rounded-md bg-background border border-border">
                    <p class="text-sm font-semibold text-foreground">Clicker Settings</p>

                    <div class="flex gap-2">
                        <div class="flex flex-col gap-1 flex-1">
                            <p class="text-xs text-muted-foreground">Select Track</p>
                            <Select.Root 
                                type="single" 
                                name="trackSelect" 
                                value={String(selectedTrackIndex)}
                                onValueChange={(v) => {
                                    const newIndex = parseInt(v);
                                    if (!isNaN(newIndex) && newIndex >= 0 && newIndex < midiData!.tracks.length) {
                                        selectedTrackIndex = newIndex;
                                    }
                                }}
                            >
                                <Select.Trigger class="w-full">
                                    {#if currentTrack}
                                        {currentTrack.name} ({currentTrack.note_count} notes)
                                    {:else}
                                        Select a track
                                    {/if}
                                </Select.Trigger>
                                <Select.Content>
                                    {#each midiData.tracks as track, index}
                                        <Select.Item value={String(index)}>
                                            {track.name} ({track.note_count} notes)
                                        </Select.Item>
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        </div>
                    </div>
                </div>
            </div>

            {#if currentTrack}
                <PianoRoll {midiData} {selectedTrackIndex} playbackTime={currentPlaybackTime} mappingConfig={mappingConfig} />
            {/if}
        </div>
    {:else}
        <div class="w-full px-4 py-8 bg-background border border-border rounded-md flex flex-col items-center justify-center gap-3 mx-auto max-w-sm my-auto">
            <FrownIcon class="size-8 text-primary" />
            <p class="text-foreground text-sm text-center">
                {#if midiData !== null && midiData.tracks.length === 0}
                    This MIDI file has no valid tracks with notes.
                {:else}
                    No MIDI file loaded. Please upload a MIDI file to get started.
                {/if}
            </p>
        </div> 
    {/if}

    <p class="text-xs text-muted-foreground text-center">This app is developed by <a href="https://github.com/KolpakovK/KiViClicker" target="_blank" rel="noopener noreferrer" class="text-primary hover:underline">Kolpakk</a></p>
</main>