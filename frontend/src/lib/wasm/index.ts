import initWasm, { convert_bin_to_uf2 as wasmConvert, ConversionOutput } from './converter.js';

let initPromise: Promise<void> | null = null;

async function ensureReady() {
  if (!initPromise) {
    initPromise = initWasm().then(() => undefined);
  }
  return initPromise;
}

export interface ConversionParams {
  slot: number;
  name: string;
  autoclockHz: number;
}

export interface ConversionResult {
  data: Uint8Array;
  startOffset: number;
  slot: number;
}

export async function convertBinToUf2(bin: Uint8Array, params: ConversionParams): Promise<ConversionResult> {
  await ensureReady();

  let output: ConversionOutput;
  try {
    output = wasmConvert(bin, params.slot, params.name, params.autoclockHz);
  } catch (error) {
    throw new Error(typeof error === 'string' ? error : (error as Error).message);
  }

  const data = new Uint8Array(output.data);
  const result: ConversionResult = {
    data,
    startOffset: output.startOffset,
    slot: output.slot,
  };

  output.free();
  return result;
}
