use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

const BASE_BITSTREAM_STORAGE_ADDRESS_KB: u32 = 544;
const RESERVED_KB_FOR_BITSTREAM_SLOT: u32 = 512;
const METADATA_START1_OFFSET: u32 = 0x42;
const METADATA_PAYLOAD_HEADER: &str = "RFMETA";
const METADATA_PAYLOAD_VERSION: &str = "01";
const METADATA_PROJ_NAME_MAXLEN: usize = 23;
const PAGE_BLOCKS: u32 = 4; // 4 kB per page
const BLOCK_PAYLOAD_SIZE: usize = 256;

const MAGIC_START0: u32 = 0x0A32_4655;
const MAGIC_START1: u32 = 0x951C_0634;
const MAGIC_END: u32 = 0x1C73_C401;
const FAMILY_ID: u32 = 0xEFAB_1E55;

const FLAGS_NOT_MAIN_FLASH: u32 = 0x0000_0001;
const FLAGS_FAMILY_ID_PRESENT: u32 = 0x0000_2000;

const MIN_AUTOCLOCK_HZ: u32 = 10;
const MAX_AUTOCLOCK_HZ: u32 = 60_000_000;
const MIN_SLOT: u8 = 1;
const MAX_SLOT: u8 = 4;

#[derive(Debug)]
struct UF2Block {
    start1: u32,
    flags: u32,
    target_addr: u32,
    payload: Vec<u8>,
}

#[wasm_bindgen]
pub struct ConversionOutput {
    data: Vec<u8>,
    start_offset: u32,
    slot: u8,
}

#[wasm_bindgen]
impl ConversionOutput {
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Uint8Array {
        Uint8Array::from(self.data.as_slice())
    }

    #[wasm_bindgen(getter, js_name = "startOffset")]
    pub fn start_offset(&self) -> u32 {
        self.start_offset
    }

    #[wasm_bindgen(getter)]
    pub fn slot(&self) -> u8 {
        self.slot
    }
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn convert_bin_to_uf2(
    bin: Uint8Array,
    slot: u8,
    name: String,
    autoclock_hz: u32,
) -> Result<ConversionOutput, JsValue> {
    let mut payload = vec![0u8; bin.length() as usize];
    bin.copy_to(&mut payload);

    let opts = Options {
        slot,
        bitstream_name: name,
        autoclock_hz,
    };

    convert(&payload, &opts).map_err(|e| JsValue::from_str(&e))
}

struct Options {
    slot: u8,
    bitstream_name: String,
    autoclock_hz: u32,
}

fn convert(data: &[u8], opts: &Options) -> Result<ConversionOutput, String> {
    validate_options(opts)?;

    let slot_index = opts.slot - 1;
    let reserved_pages_for_bitstream_slot = RESERVED_KB_FOR_BITSTREAM_SLOT / PAGE_BLOCKS;
    let base_page = BASE_BITSTREAM_STORAGE_ADDRESS_KB / PAGE_BLOCKS;
    let pages_required = (data.len() as u32 / (4 * 1024)) + 4;

    if pages_required >= reserved_pages_for_bitstream_slot {
        return Err("Bitstream is too large for the selected slot".to_string());
    }

    let lowest_page_for_slot = base_page + (reserved_pages_for_bitstream_slot * slot_index as u32);
    let upper_random = reserved_pages_for_bitstream_slot.saturating_sub(pages_required);
    let random_offset_pages = rand_between(4, upper_random)?;
    let start_page = lowest_page_for_slot + random_offset_pages;
    let start_offset = start_page * PAGE_BLOCKS * 1024;

    let mut blocks = Vec::new();
    let metadata_block = build_metadata_block(start_offset, data.len() as u32, opts);
    blocks.push(metadata_block);
    blocks.extend(build_payload_blocks(start_offset, data));

    let num_blocks = blocks.len() as u32;
    let mut output = Vec::with_capacity(num_blocks as usize * 512);

    for (index, block) in blocks.into_iter().enumerate() {
        output.extend(serialize_block(index as u32, num_blocks, &block));
    }

    Ok(ConversionOutput {
        data: output,
        start_offset,
        slot: opts.slot,
    })
}

fn serialize_block(block_no: u32, total_blocks: u32, block: &UF2Block) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(512);
    buffer.extend_from_slice(&MAGIC_START0.to_le_bytes());
    buffer.extend_from_slice(&block.start1.to_le_bytes());
    buffer.extend_from_slice(&block.flags.to_le_bytes());
    buffer.extend_from_slice(&block.target_addr.to_le_bytes());
    buffer.extend_from_slice(&(block.payload.len() as u32).to_le_bytes());
    buffer.extend_from_slice(&block_no.to_le_bytes());
    buffer.extend_from_slice(&total_blocks.to_le_bytes());
    buffer.extend_from_slice(&FAMILY_ID.to_le_bytes());

    let mut padded = [0u8; 476];
    let copy_len = block.payload.len().min(476);
    padded[..copy_len].copy_from_slice(&block.payload[..copy_len]);
    buffer.extend_from_slice(&padded);
    buffer.extend_from_slice(&MAGIC_END.to_le_bytes());

    buffer
}

fn build_metadata_block(start_offset: u32, bitstream_size: u32, opts: &Options) -> UF2Block {
    let mut name = opts.bitstream_name.clone();
    let mut effective_len = name.len();
    if effective_len > METADATA_PROJ_NAME_MAXLEN {
        name.truncate(METADATA_PROJ_NAME_MAXLEN);
        effective_len = METADATA_PROJ_NAME_MAXLEN;
    }

    let mut name_bytes = name.into_bytes();
    if name_bytes.len() < METADATA_PROJ_NAME_MAXLEN {
        name_bytes
            .extend(std::iter::repeat(0).take(METADATA_PROJ_NAME_MAXLEN - name_bytes.len()));
    }

    let mut payload = Vec::new();
    payload.extend_from_slice(
        format!("{}{}", METADATA_PAYLOAD_HEADER, METADATA_PAYLOAD_VERSION).as_bytes(),
    );
    payload.extend_from_slice(&bitstream_size.to_le_bytes());
    payload.push(effective_len as u8);
    payload.extend_from_slice(&name_bytes);
    payload.extend_from_slice(&opts.autoclock_hz.to_le_bytes());

    UF2Block {
        start1: MAGIC_START1 + METADATA_START1_OFFSET,
        flags: FLAGS_NOT_MAIN_FLASH | FLAGS_FAMILY_ID_PRESENT,
        target_addr: start_offset,
        payload,
    }
}

fn build_payload_blocks(start_offset: u32, data: &[u8]) -> Vec<UF2Block> {
    data
        .chunks(BLOCK_PAYLOAD_SIZE)
        .enumerate()
        .map(|(index, chunk)| UF2Block {
            start1: MAGIC_START1,
            flags: FLAGS_FAMILY_ID_PRESENT,
            target_addr: start_offset + (index as u32 * BLOCK_PAYLOAD_SIZE as u32),
            payload: chunk.to_vec(),
        })
        .collect()
}

fn validate_options(opts: &Options) -> Result<(), String> {
    if opts.slot < MIN_SLOT || opts.slot > MAX_SLOT {
        return Err("Select a slot between 1-4".to_string());
    }

    if opts.autoclock_hz != 0
        && (opts.autoclock_hz < MIN_AUTOCLOCK_HZ || opts.autoclock_hz > MAX_AUTOCLOCK_HZ)
    {
        return Err("Auto-clocking only supports rates between 10Hz and 60MHz".to_string());
    }

    Ok(())
}

fn rand_between(min: u32, max: u32) -> Result<u32, String> {
    if max < min {
        return Err("Bitstream is too large for the available slot".to_string());
    }

    let range = max - min + 1;
    let sample = js_sys::Math::random();
    Ok(min + (sample * range as f64).floor() as u32)
}
