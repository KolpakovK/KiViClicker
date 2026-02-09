<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';

    import { CogIcon, MousePointerClickIcon, UploadIcon } from "@lucide/svelte";
    import Button from "./ui/button/button.svelte";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import * as Sheet from "$lib/components/ui/sheet/index.js";
    import MappingEditor from './MappingEditor.svelte';
    import { DEFAULT_CONFIGS } from '$lib/types/Mappings';

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

    let midiFile: File | null = $state(null);
    let midiData: MidiFileInfo | null = $state(null);
    let uploadDialogOpen = $state(false);
    let currentMappingConfig = $state(DEFAULT_CONFIGS[0]);

    let {
        onMidiDataLoaded,
        onMappingChange
    }:{
        onMidiDataLoaded: (data: MidiFileInfo | null) => void;
        onMappingChange?: (config: typeof currentMappingConfig) => void;
    } = $props();

    async function FileUpload(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files.length > 0) {
            const file = input.files[0];
            console.log("Uploaded file:", file);
            midiFile = file;
            uploadDialogOpen = false;

            try {
                const arrayBuffer = await file.arrayBuffer();
                const bytes = Array.from(new Uint8Array(arrayBuffer));
                
                midiData = await invoke<MidiFileInfo>('parse_midi_data', {
                    fileName: file.name,
                    data: bytes
                });
                
                console.log('MIDI loaded:', midiData);
                uploadDialogOpen = false;
            } catch (e) {
                console.error('Failed to parse MIDI:', e);
            } finally {
                console.log('MIDI file processing completed');
                onMidiDataLoaded(midiData);
            }
        }
    }
</script>

<nav class="px-3 py-1 w-full bg-background border border-border rounded-md flex items-center justify-between gap-2">
    <div class="logo flex items-center gap-3">
        <div class="flex items-center gap-1">
            <MousePointerClickIcon class="size-4 text-primary" />
            <p class="text-sm font-black text-primary">KiVi</p>
        </div>
        <span class="px-2 py-0.5 bg-secondary text-secondary-foreground rounded-full text-xs">By Kolpakk</span>
    </div>

    <div class="flex items-center gap-2">
        
        <Dialog.Root bind:open={uploadDialogOpen}>
            <Dialog.Trigger>
                <Button size="sm" variant="black">
                    <UploadIcon class="size-4" />
                    Load MIDI file
                </Button>
            </Dialog.Trigger>
            <Dialog.Content class="w-sm">
                <Dialog.Header>
                    <Dialog.Title>Upload MIDI file</Dialog.Title>
                    <Dialog.Description>
                        Choose a MIDI file from your device to load it into the application. Supported formats include .mid and .midi. Once uploaded, you can view and interact with the MIDI data within the app.
                    </Dialog.Description>
                </Dialog.Header>
                <div 
                    class="flex flex-col gap-1 p-4 relative border border-border border-dashed hover:bg-muted rounded-md cursor-pointer"
                >
                    <input type="file" accept=".mid,.midi" class="absolute inset-0 opacity-0 cursor-pointer h-full w-full" onchange={FileUpload} />
                    {#if midiFile}
                        <p class="text-sm text-muted-foreground text-center">Selected file: {midiFile.name}</p>
                    {:else}
                        <UploadIcon class="size-4 text-muted-foreground mx-auto" />
                        <p class="text-sm text-muted-foreground text-center">Choose a file</p>
                    {/if}
                </div>
            </Dialog.Content>
        </Dialog.Root>

        <Sheet.Root>
            <Sheet.Trigger>
                <Button size="icon-sm" variant="outline">
                    <CogIcon class="size-4" />
                </Button>
            </Sheet.Trigger>
            <Sheet.Content class="h-screen overflow-y-scroll">
                <Sheet.Header>
                    <Sheet.Title>App Configuration</Sheet.Title>
                </Sheet.Header>

                <div class="px-4 flex flex-col gap-1">
                    <MappingEditor 
                        bind:selectedConfig={currentMappingConfig}
                        onConfigChange={(config) => {
                            onMappingChange?.(config);
                        }}
                    />
                </div>
            </Sheet.Content>
        </Sheet.Root>
    </div>
</nav>