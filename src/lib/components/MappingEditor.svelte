<script lang="ts">
    import { onMount } from 'svelte';
    import Button from "./ui/button/button.svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import Input from "./ui/input/input.svelte";
    import Textarea from "./ui/textarea/textarea.svelte";
    import {
        ChevronUpIcon,
        ChevronDownIcon,
        PlusIcon,
        TrashIcon,
        EditIcon,
    } from "@lucide/svelte";
    import { DEFAULT_CONFIGS, type MappingConfig, type KeyMapping } from "$lib/types/Mappings";

    const STORAGE_KEY = 'kivi-custom-configs';

    let {
        selectedConfig = $bindable(),
        onConfigChange,
    }: {
        selectedConfig: MappingConfig;
        onConfigChange?: (config: MappingConfig) => void;
    } = $props();

    let configs = $state([...DEFAULT_CONFIGS]);
    let customConfigs = $state<MappingConfig[]>([]);
    let allConfigs = $derived([...configs, ...customConfigs]);
    let octaveShift = $state(0);
    
    let createDialogOpen = $state(false);
    let editDialogOpen = $state(false);
    let editingKey = $state<KeyMapping | null>(null);
    let editingKeyIndex = $state(-1);
    
    let newConfigName = $state('');
    let newConfigDescription = $state('');
    let newConfigMappings = $state<KeyMapping[]>([]);
    
    let editKey = $state('');
    let editMidiNote = $state(60);

    function getMidiNoteName(midiNote: number): string {
        const noteNames = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        const octave = Math.floor(midiNote / 12) - 1;
        const noteName = noteNames[midiNote % 12];
        return `${noteName}${octave}`;
    }

    function shiftOctave(direction: number) {
        octaveShift += direction;
        selectedConfig = {
            ...selectedConfig,
            octaveShift,
            mappings: selectedConfig.mappings.map((m) => ({
                ...m,
                midiNote: m.midiNote + direction * 12,
            })),
        };

        if (onConfigChange) {
            onConfigChange(selectedConfig);
        }
    }

    function selectConfig(configId: string) {
        const config = allConfigs.find((c) => c.id === configId);
        if (config) {
            selectedConfig = { ...config };
            octaveShift = config.octaveShift;
            if (onConfigChange) {
                onConfigChange(selectedConfig);
            }
        }
    }

    function loadCustomConfigs() {
        try {
            const stored = localStorage.getItem(STORAGE_KEY);
            if (stored) {
                customConfigs = JSON.parse(stored);
            }
        } catch (e) {
            console.error('Failed to load custom configs:', e);
        }
    }

    function saveCustomConfigs() {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(customConfigs));
        } catch (e) {
            console.error('Failed to save custom configs:', e);
        }
    }

    function openCreateDialog() {
        newConfigName = '';
        newConfigDescription = '';
        newConfigMappings = [];
        createDialogOpen = true;
    }

    function addKeyMapping() {
        editKey = '';
        editMidiNote = 60;
        editingKey = null;
        editingKeyIndex = -1;
        editDialogOpen = true;
    }

    function editKeyMapping(mapping: KeyMapping, index: number) {
        editKey = mapping.key;
        editMidiNote = mapping.midiNote;
        editingKey = mapping;
        editingKeyIndex = index;
        editDialogOpen = true;
    }

    function saveKeyMapping() {
        const newMapping: KeyMapping = {
            key: editKey.toUpperCase(),
            midiNote: editMidiNote,
        };

        if (editingKeyIndex >= 0) {
            newConfigMappings[editingKeyIndex] = newMapping;
        } else {
            newConfigMappings = [...newConfigMappings, newMapping];
        }

        editDialogOpen = false;
    }

    function removeKeyMapping(index: number) {
        newConfigMappings = newConfigMappings.filter((_, i) => i !== index);
    }

    function createConfig() {
        if (!newConfigName.trim() || newConfigMappings.length === 0) {
            alert('Please provide a name and at least one key mapping');
            return;
        }

        const newConfig: MappingConfig = {
            id: `custom-${Date.now()}`,
            name: newConfigName,
            description: newConfigDescription,
            octaveShift: 0,
            mappings: newConfigMappings,
        };

        customConfigs = [...customConfigs, newConfig];
        saveCustomConfigs();
        
        selectedConfig = newConfig;
        if (onConfigChange) {
            onConfigChange(newConfig);
        }

        createDialogOpen = false;
    }

    function deleteConfig(configId: string) {
        if (!configId.startsWith('custom-')) {
            alert('Cannot delete default configs');
            return;
        }

        if (!confirm('Are you sure you want to delete this configuration?')) {
            return;
        }

        customConfigs = customConfigs.filter(c => c.id !== configId);
        saveCustomConfigs();

        if (selectedConfig.id === configId) {
            selectConfig(DEFAULT_CONFIGS[0].id);
        }
    }

    onMount(() => {
        loadCustomConfigs();
    });
</script>

<div class="flex flex-col gap-4">
    <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between">
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <label class="text-sm font-medium">Mapping Configuration</label>
            <Button size="sm" variant="outline" onclick={openCreateDialog}>
                <PlusIcon class="size-3 mr-1" />
                New Config
            </Button>
        </div>
        
        <Select.Root type="single" value={selectedConfig.id} onValueChange={selectConfig}>
            <Select.Trigger class="w-full">
                {selectedConfig.name}
            </Select.Trigger>
            <Select.Content>
                <div class="px-2 py-1 text-xs font-semibold text-muted-foreground">Default Configs</div>
                {#each configs as config}
                    <Select.Item value={config.id}>
                        {config.name}
                        {#if config.description}
                            <span class="text-xs text-muted-foreground ml-2">
                                {config.description}
                            </span>
                        {/if}
                    </Select.Item>
                {/each}
                
                {#if customConfigs.length > 0}
                    <div class="px-2 py-1 text-xs font-semibold text-muted-foreground mt-2">Custom Configs</div>
                    {#each customConfigs as config}
                        <Select.Item value={config.id}>
                            <div class="flex items-center justify-between w-full">
                                <span>{config.name}</span>
                                <Button 
                                    size="icon-sm" 
                                    variant="ghost"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        deleteConfig(config.id);
                                    }}
                                >
                                    <TrashIcon class="size-3" />
                                </Button>
                            </div>
                        </Select.Item>
                    {/each}
                {/if}
            </Select.Content>
        </Select.Root>
    </div>

    <div class="flex flex-col gap-2">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="text-sm font-medium">Octave Shift</label>
        <div class="flex items-center gap-2">
            <Button
                size="icon-sm"
                variant="outline"
                onclick={() => shiftOctave(-1)}
                disabled={octaveShift <= -2}
            >
                <ChevronDownIcon class="size-4" />
            </Button>

            <div class="flex-1 text-center">
                <span class="text-sm font-mono">
                    {octaveShift > 0 ? "+" : ""}{octaveShift} octave{Math.abs(octaveShift) !== 1 ? "s" : ""}
                </span>
            </div>

            <Button
                size="icon-sm"
                variant="outline"
                onclick={() => shiftOctave(1)}
                disabled={octaveShift >= 2}
            >
                <ChevronUpIcon class="size-4" />
            </Button>
        </div>
    </div>

    <div class="flex flex-col gap-2">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="text-sm font-medium">
            Key Mappings ({selectedConfig.mappings.length} keys)
        </label>
        <div class="max-h-64 overflow-y-auto border border-border rounded-md p-2 bg-muted/30">
            <div class="grid grid-cols-2 gap-2">
                {#each selectedConfig.mappings as mapping}
                    <div class="flex items-center gap-2 p-2 bg-background rounded border border-border">
                        <kbd class="px-2 py-1 bg-secondary text-secondary-foreground rounded text-xs font-mono">
                            {mapping.key}
                        </kbd>
                        <span class="text-xs">→</span>
                        <span class="text-xs font-mono">
                            {getMidiNoteName(mapping.midiNote)}
                        </span>
                    </div>
                {/each}
            </div>
        </div>
    </div>
</div>

<Dialog.Root bind:open={createDialogOpen}>
    <Dialog.Content>
        <Dialog.Header>
            <Dialog.Title>Create New Mapping Configuration</Dialog.Title>
        </Dialog.Header>
        
        <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <label class="text-sm font-medium">Name</label>
                <Input bind:value={newConfigName} placeholder="My Custom Layout" />
            </div>
            
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <div class="flex flex-col gap-2">
                <label class="text-sm font-medium">Description (optional)</label>
                <Textarea bind:value={newConfigDescription} placeholder="Description of your layout..." />
            </div>
            
            <div class="flex flex-col gap-2">
                <div class="flex items-center justify-between">
                    <!-- svelte-ignore a11y_label_has_associated_control -->
                    <label class="text-sm font-medium">Key Mappings ({newConfigMappings.length})</label>
                    <Button size="sm" variant="outline" onclick={addKeyMapping}>
                        <PlusIcon class="size-3 mr-1" />
                        Add Key
                    </Button>
                </div>
                
                <div class="max-h-48 overflow-y-auto border border-border rounded-md p-2">
                    {#if newConfigMappings.length === 0}
                        <p class="text-xs text-muted-foreground text-center py-4">No keys mapped yet</p>
                    {:else}
                        <div class="flex flex-col gap-1">
                            {#each newConfigMappings as mapping, index}
                                <div class="flex items-center gap-2 p-2 bg-muted rounded">
                                    <kbd class="px-2 py-1 bg-background rounded text-xs font-mono">
                                        {mapping.key}
                                    </kbd>
                                    <span class="text-xs">→</span>
                                    <span class="text-xs font-mono flex-1">
                                        {getMidiNoteName(mapping.midiNote)}
                                    </span>
                                    <Button 
                                        size="icon-sm" 
                                        variant="ghost"
                                        onclick={() => editKeyMapping(mapping, index)}
                                    >
                                        <EditIcon class="size-3" />
                                    </Button>
                                    <Button 
                                        size="icon-sm" 
                                        variant="ghost"
                                        onclick={() => removeKeyMapping(index)}
                                    >
                                        <TrashIcon class="size-3" />
                                    </Button>
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            </div>
            
            <div class="flex gap-2 justify-end">
                <Button variant="outline" onclick={() => createDialogOpen = false}>Cancel</Button>
                <Button onclick={createConfig}>Create Configuration</Button>
            </div>
        </div>
    </Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={editDialogOpen}>
    <Dialog.Content class="sm:max-w-md">
        <Dialog.Header>
            <Dialog.Title>{editingKeyIndex >= 0 ? 'Edit' : 'Add'} Key Mapping</Dialog.Title>
        </Dialog.Header>
        
        <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <label class="text-sm font-medium">Keyboard Key</label>
                <Input 
                    bind:value={editKey} 
                    placeholder="Q, W, E, 1, 2, etc."
                    maxlength={1}
                />
            </div>
            
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <div class="flex flex-col gap-2">
                <label class="text-sm font-medium">
                    MIDI Note ({getMidiNoteName(editMidiNote)})
                </label>
                <Input 
                    type="number" 
                    bind:value={editMidiNote} 
                    min={21}
                    max={108}
                />
                <p class="text-xs text-muted-foreground">
                    Range: 21 (A0) to 108 (C8)
                </p>
            </div>
            
            <div class="flex gap-2 justify-end">
                <Button variant="outline" onclick={() => editDialogOpen = false}>Cancel</Button>
                <Button onclick={saveKeyMapping}>Save</Button>
            </div>
        </div>
    </Dialog.Content>
</Dialog.Root>