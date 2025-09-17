<script lang="ts">
  import { onDestroy } from 'svelte';
  import { convertBinToUf2 } from '$lib/wasm';

  const MAX_NAME = 23;

  let file: File | null = null;
  let slot = 1;
  let bitstreamName = '';
  let autoclockHz = '';
  let appendSlotSuffix = false;

  let isConverting = false;
  let errorMessage = '';
  let success: {
    url: string;
    fileName: string;
    startOffset: number;
    size: number;
  } | null = null;

  function handleFileChange(event: Event) {
    const input = event.target as HTMLInputElement;
    file = input.files?.[0] ?? null;
  }

  onDestroy(() => {
    if (success?.url) {
      URL.revokeObjectURL(success.url);
    }
  });

  function resetOutcome() {
    if (success?.url) {
      URL.revokeObjectURL(success.url);
    }
    success = null;
    errorMessage = '';
  }

  async function runConversion() {
    resetOutcome();

    if (!file) {
      errorMessage = 'Select a .bin file to convert.';
      return;
    }

    isConverting = true;

    try {
      const buffer = new Uint8Array(await file.arrayBuffer());
      const inferredName = deriveName(bitstreamName, file.name);
      const parsedAutoClock = parseAutoclock(autoclockHz);

      const slotNumber = Number(slot);
      if (!Number.isInteger(slotNumber) || slotNumber < 1 || slotNumber > 4) {
        throw new Error('Slot must be an integer between 1 and 4.');
      }

      const conversion = await convertBinToUf2(buffer, {
        slot: slotNumber,
        name: inferredName,
        autoclockHz: parsedAutoClock,
      });

      const bytes = conversion.data;
      const arrayBuffer = new ArrayBuffer(bytes.length);
      new Uint8Array(arrayBuffer).set(bytes);
      const blob = new Blob([arrayBuffer], { type: 'application/octet-stream' });
      const url = URL.createObjectURL(blob);
      const uf2Name = buildOutputName(inferredName, slotNumber, appendSlotSuffix);

      success = {
        url,
        fileName: uf2Name,
        startOffset: conversion.startOffset,
        size: conversion.data.length,
      };
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      isConverting = false;
    }
  }

  function deriveName(userName: string, fallbackFileName: string) {
    const trimmed = userName.trim();
    if (trimmed.length > 0) {
      return trimmed.slice(0, MAX_NAME);
    }
    const base = fallbackFileName.replace(/\.[^.]+$/, '');
    return base.slice(0, MAX_NAME);
  }

  function parseAutoclock(value: string) {
    if (value.trim().length === 0) {
      return 0;
    }
    const parsed = Number(value);
    if (!Number.isFinite(parsed)) {
      throw new Error('Provide a numeric auto-clock value in Hz.');
    }
    const truncated = Math.trunc(parsed);
    if (truncated !== 0 && (truncated < 10 || truncated > 60_000_000)) {
      throw new Error('Auto-clock must be between 10 and 60,000,000 Hz.');
    }
    return truncated;
  }

  function buildOutputName(base: string, slotNumber: number, shouldAppend: boolean) {
    const safeBase = base.length ? base : 'bitstream';
    const suffix = shouldAppend ? `_slot${slotNumber}` : '';
    return `${safeBase}${suffix}.uf2`;
  }
</script>

<svelte:head>
  <title>BIN → UF2 Converter</title>
</svelte:head>

<div class="min-h-screen bg-base-200">
  <div class="mx-auto flex w-full max-w-4xl flex-col gap-6 px-4 py-10">
    <div class="text-center">
      <h1 class="text-3xl font-bold">BIN → UF2 Converter</h1>
      <p class="mt-2 text-base-content/80">
        Convert FPGA bitstream binaries into UF2 packets that match the efabless target profile.
      </p>
    </div>

    <div class="card bg-base-100 shadow-xl">
      <div class="card-body space-y-6">
        <div class="grid gap-6 md:grid-cols-2">
          <label class="form-control w-full">
            <div class="label">
              <span class="label-text font-semibold">Bitstream (.bin) file</span>
            </div>
            <input
              class="file-input file-input-bordered w-full"
              type="file"
              accept=".bin,application/octet-stream"
              on:change={handleFileChange}
            />
          </label>

          <label class="form-control w-full">
            <div class="label">
              <span class="label-text font-semibold">Display name</span>
              <span class="label-text-alt">max {MAX_NAME} characters</span>
            </div>
            <input
              class="input input-bordered"
              maxlength={MAX_NAME}
              placeholder="Optional project name"
              bind:value={bitstreamName}
            />
          </label>

          <label class="form-control w-full">
            <div class="label">
              <span class="label-text font-semibold">Slot</span>
            </div>
            <select class="select select-bordered" bind:value={slot}>
              <option value={1}>Slot 1</option>
              <option value={2}>Slot 2</option>
              <option value={3}>Slot 3</option>
              <option value={4}>Slot 4</option>
            </select>
          </label>

          <label class="form-control w-full">
            <div class="label">
              <span class="label-text font-semibold">Auto-clock (Hz)</span>
              <span class="label-text-alt">0 for default</span>
            </div>
            <input
              class="input input-bordered"
              type="number"
              min="0"
              max="60000000"
              step="1"
              placeholder="10 - 60000000"
              bind:value={autoclockHz}
            />
          </label>
        </div>

        <label class="label cursor-pointer justify-start gap-3">
          <input type="checkbox" class="checkbox" bind:checked={appendSlotSuffix} />
          <span class="label-text">Append slot number to the downloaded filename</span>
        </label>

        <div class="flex flex-wrap items-center justify-between gap-3">
          <button class="btn btn-primary" on:click={runConversion} disabled={isConverting}>
            {#if isConverting}
              <span class="loading loading-spinner"></span>
              Converting…
            {:else}
              Convert to UF2
            {/if}
          </button>

          {#if file}
            <span class="text-sm text-base-content/70">Loaded file: {file.name}</span>
          {/if}
        </div>

        {#if errorMessage}
          <div class="alert alert-error">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              class="h-6 w-6 shrink-0 stroke-current"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 9v3m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span>{errorMessage}</span>
          </div>
        {/if}

        {#if success}
          <div class="alert alert-success items-center justify-between gap-4">
            <div class="flex flex-col">
              <h2 class="font-semibold">Conversion ready</h2>
              <p class="text-sm">
                Start offset <code>{`0x${success.startOffset.toString(16)}`}</code>, UF2 size {success.size}
                bytes.
              </p>
            </div>
            <a
              class="btn btn-sm btn-neutral ml-auto"
              href={success.url}
              download={success.fileName}
            >
              Download {success.fileName}
            </a>
          </div>
        {/if}
      </div>
    </div>

    <div class="card bg-base-100 shadow">
      <div class="card-body space-y-3 text-sm text-base-content/70">
        <h2 class="card-title text-base">Notes</h2>
        <ul class="list-disc space-y-1 pl-5">
          <li>The converter always targets the efabless UF2 profile.</li>
          <li>
            Bitstream names longer than {MAX_NAME} characters are truncated to match the metadata header
            limit.
          </li>
          <li>
            Auto-clock values must fall between 10 Hz and 60 MHz; leave blank to retain the board default.
          </li>
          <li>Generated UF2 files stay in-browser; refresh to clear cached links.</li>
        </ul>
      </div>
    </div>
  </div>
</div>
