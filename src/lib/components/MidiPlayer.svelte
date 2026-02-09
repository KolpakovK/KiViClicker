<script lang="ts">
    import { onDestroy } from 'svelte';
    import * as Tone from 'tone';
    import Button from './ui/button/button.svelte';
    import Slider from './ui/slider/slider.svelte';
    import { PlayIcon, PauseIcon, SquareIcon } from '@lucide/svelte';
    import type { MidiFileInfo } from '$lib/types/MidiTypes';


    let {
        midiData,
        selectedTrackIndex,
        onTimeUpdate
    }: {
        midiData: MidiFileInfo;
        selectedTrackIndex: number;
        onTimeUpdate?: (time: number) => void;
    } = $props();

    let isPlaying = $state(false);
    let currentTime = $state(0);
    let volumeValue = $state(0.5);
    let synth: Tone.PolySynth | null = null;
    let animationFrameId: number | null = null;
    let startTimeMs = 0;
    let pausedAtMs = 0;
    let activeNotes = new Map<string, number>();

    let currentTrack = $derived.by(() => {
        if (!midiData || selectedTrackIndex < 0 || selectedTrackIndex >= midiData.tracks.length) {
            return null;
        }
        return midiData.tracks[selectedTrackIndex];
    });

    function midiToNoteName(midi: number): string {
        const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
        const octave = Math.floor(midi / 12) - 1;
        const noteName = noteNames[midi % 12];
        return `${noteName}${octave}`;
    }

    function transformDuration(duration_ms: number): string {
        const minutes = Math.floor(duration_ms / 60000);
        const seconds = Math.floor((duration_ms % 60000) / 1000);
        return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    }

    async function initAudio() {
        if (!synth) {
            await Tone.start();
            synth = new Tone.PolySynth(Tone.Synth, {
                oscillator: { type: 'triangle' },
                envelope: {
                    attack: 0.005,
                    decay: 0.1,
                    sustain: 0.3,
                    release: 0.5
                }
            }).toDestination();
        }
    }

    async function play() {
        if (!currentTrack) return;
        
        await initAudio();
        if (!synth) return;

        isPlaying = true;
        startTimeMs = Date.now() - pausedAtMs;

        playbackLoop();
    }

    function playbackLoop() {
        if (!isPlaying || !currentTrack || !synth) return;

        const now = Date.now();
        currentTime = now - startTimeMs;

        if (currentTime >= midiData.duration_ms) {
            stop();
            return;
        }

        if (synth) {
            synth.volume.value = volumeValue === 0 ? -60 : (volumeValue * 60) - 60;
        }

        const lookAheadMs = 50;
        const timeWindow = currentTime + lookAheadMs;

        currentTrack.notes.forEach((note) => {
            const noteKey = `${note.note}-${note.time_ms}`;
            const noteEndTime = note.time_ms + note.duration_ms;

            if (note.time_ms >= currentTime && 
                note.time_ms < timeWindow && 
                !activeNotes.has(noteKey)) {
                
                const noteName = midiToNoteName(note.note);
                const velocity = note.velocity / 127;
                const duration = note.duration_ms / 1000;

                synth!.triggerAttack(noteName, undefined, velocity);
                
                const releaseTime = setTimeout(() => {
                    synth!.triggerRelease(noteName);
                    activeNotes.delete(noteKey);
                }, duration * 1000);

                activeNotes.set(noteKey, releaseTime);
            }
        });

        if (onTimeUpdate) {
            onTimeUpdate(currentTime);
        }

        animationFrameId = requestAnimationFrame(playbackLoop);
    }

    function pause() {
        if (!isPlaying) return;
        
        pausedAtMs = currentTime;
        isPlaying = false;

        releaseAllNotes();
        
        if (animationFrameId !== null) {
            cancelAnimationFrame(animationFrameId);
            animationFrameId = null;
        }
    }

    function stop() {
        isPlaying = false;
        currentTime = 0;
        pausedAtMs = 0;
        startTimeMs = 0;

        releaseAllNotes();

        if (animationFrameId !== null) {
            cancelAnimationFrame(animationFrameId);
            animationFrameId = null;
        }

        if (onTimeUpdate) {
            onTimeUpdate(0);
        }
    }

    function releaseAllNotes() {
        activeNotes.forEach((timeoutId) => {
            clearTimeout(timeoutId);
        });
        activeNotes.clear();

        if (synth) {
            synth.releaseAll();
        }
    }

    function togglePlayPause() {
        if (isPlaying) {
            pause();
        } else {
            play();
        }
    }

    $effect(() => {
        if (synth) {
            const dbValue = volumeValue === 0 ? -60 : (volumeValue * 60) - 60;
            synth.volume.value = dbValue;
        }
    });

    let prevTrackIndex = $state(selectedTrackIndex);
    $effect(() => {
        if (selectedTrackIndex !== prevTrackIndex) {
            stop();
            prevTrackIndex = selectedTrackIndex;
        }
    });

    onDestroy(() => {
        stop();
        if (synth) {
            synth.dispose();
            synth = null;
        }
    });
</script>

<div class="flex items-center gap-2">
    <Button size="icon-sm" onclick={togglePlayPause}>
        {#if isPlaying}
            <PauseIcon class="size-4" />
        {:else}
            <PlayIcon class="size-4" />
        {/if}
    </Button>
    <Button size="icon-sm" variant="outline" onclick={stop}>
        <SquareIcon class="size-4" />
    </Button>

    <p class="text-xs text-muted-foreground flex-1">
        {transformDuration(currentTime)} / {transformDuration(midiData.duration_ms)}
    </p>

    <div class="flex items-center gap-1">
        <Slider type="single" min={0} max={1} step={0.01} bind:value={volumeValue} class="w-20" />
        <p class="text-xs w-8 text-right text-foreground">{Math.round(volumeValue * 100)}%</p>
    </div>
</div>