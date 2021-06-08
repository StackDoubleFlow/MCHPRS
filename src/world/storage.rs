use crate::blocks::{BlockEntity, BlockPos};
use crate::network::packets::clientbound::{
    CMultiBlockChangeRecord, CChunkData, CChunkDataSection, CMultiBlockChange, ClientBoundPacket,
};
use crate::network::packets::PacketEncoder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::mem;

#[derive(Clone)]
pub struct BitBuffer {
    bits_per_entry: u64,
    entries_per_long: u64,
    entries: usize,
    mask: u64,
    longs: Vec<u64>,
    fast_arr_idx: fn(word_idx: usize) -> usize,
}

impl std::fmt::Debug for BitBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitBuffer")
            .field("bits_per_entry", &self.bits_per_entry)
            .field("entries", &self.entries)
            .field("entries_per_long", &self.entries_per_long)
            .field("mask", &self.mask)
            .finish()
    }
}

impl BitBuffer {
    fn find_fast_arr_idx_fn(entries_per_long: usize) -> fn(word_idx: usize) -> usize {
        fn fast_arr_idx<const N: usize>(word_idx: usize) -> usize {
            word_idx / N
        }

        match entries_per_long {
            16 => fast_arr_idx::<16>,
            12 => fast_arr_idx::<12>,
            10 => fast_arr_idx::<10>,
            9 => fast_arr_idx::<9>,
            8 => fast_arr_idx::<8>,
            7 => fast_arr_idx::<7>,
            4 => fast_arr_idx::<4>,
            _ => unreachable!("entries_per_long cannot be {}", entries_per_long),
        }
    }

    pub fn create(bits_per_entry: u8, entries: usize) -> BitBuffer {
        // 4..9, 15
        let entries_per_long = 64 / bits_per_entry as u64;
        // Rounding up div
        let longs_len = (entries + entries_per_long as usize - 1) / entries_per_long as usize;
        let longs = vec![0; longs_len];
        BitBuffer {
            bits_per_entry: bits_per_entry as u64,
            longs,
            entries,
            entries_per_long,
            mask: (1 << bits_per_entry) - 1,
            fast_arr_idx: BitBuffer::find_fast_arr_idx_fn(entries_per_long as usize),
        }
    }

    fn load(entries: usize, bits_per_entry: u8, longs: Vec<u64>) -> BitBuffer {
        let entries_per_long = 64 / bits_per_entry as u64;
        BitBuffer {
            bits_per_entry: bits_per_entry as u64,
            longs,
            entries,
            entries_per_long,
            mask: (1 << bits_per_entry) - 1,
            fast_arr_idx: BitBuffer::find_fast_arr_idx_fn(entries_per_long as usize),
        }
    }

    pub fn get_entry(&self, word_idx: usize) -> u32 {
        // Find the set of indices.
        let arr_idx = (self.fast_arr_idx)(word_idx);
        let sub_idx =
            (word_idx as u64 - arr_idx as u64 * self.entries_per_long) * self.bits_per_entry;
        // Find the word.
        let word = (self.longs[arr_idx] >> sub_idx) & self.mask;
        word as u32
    }

    pub fn set_entry(&mut self, word_idx: usize, word: u32) {
        // Find the set of indices.
        let arr_idx = (self.fast_arr_idx)(word_idx);
        let sub_idx =
            (word_idx as u64 - arr_idx as u64 * self.entries_per_long) * self.bits_per_entry;
        // Set the word.
        let mask = !(self.mask << sub_idx);
        self.longs[arr_idx] = (self.longs[arr_idx] & mask) | ((word as u64) << sub_idx);
    }
}

#[test]
fn bitbuffer_format() {
    let entries = [
        1, 2, 2, 3, 4, 4, 5, 6, 6, 4, 8, 0, 7, 4, 3, 13, 15, 16, 9, 14, 10, 12, 0, 2,
    ];
    let mut buffer = BitBuffer::create(5, 24);
    for (i, entry) in entries.iter().enumerate() {
        buffer.set_entry(i, *entry);
    }
    assert_eq!(buffer.longs[0], 0x0020863148418841);
    assert_eq!(buffer.longs[1], 0x01018A7260F68C87);
}

#[derive(Debug, Clone)]
pub struct PalettedBitBuffer {
    data: BitBuffer,
    palette: Vec<u32>,
    max_entries: u32,
    use_palette: bool,
}

impl PalettedBitBuffer {
    pub fn with_entries(entries: usize) -> PalettedBitBuffer {
        let palette = vec![0];
        PalettedBitBuffer {
            data: BitBuffer::create(4, entries),
            palette,
            max_entries: 16,
            use_palette: true,
        }
    }

    fn load(
        entries: usize,
        bits_per_entry: u8,
        longs: Vec<u64>,
        palette: Vec<u32>,
    ) -> PalettedBitBuffer {
        PalettedBitBuffer {
            data: BitBuffer::load(entries, bits_per_entry, longs),
            palette,
            use_palette: bits_per_entry < 9,
            max_entries: 1 << bits_per_entry,
        }
    }

    fn resize_buffer(&mut self) {
        assert!(
            self.use_palette,
            "The buffer should never resizing if it's already using global palette"
        );
        let old_bits_per_entry = self.data.bits_per_entry;
        // It is more efficient to use the global palette when the bits reaches 9
        // As of 1.16, the global palette requires 15 bits
        let new_bits = if old_bits_per_entry + 1 >= 9 {
            self.max_entries = 1 << 15;
            self.use_palette = false;
            15
        } else {
            self.max_entries <<= 1;
            old_bits_per_entry as u8 + 1
        };
        // Swap out the old buffer
        let mut old_buffer = BitBuffer::create(new_bits, self.data.entries);
        mem::swap(&mut self.data, &mut old_buffer);
        // Copy entries into new buffer
        if new_bits == 15 {
            for entry_idx in 0..old_buffer.entries {
                let entry = self.palette[old_buffer.get_entry(entry_idx) as usize];
                self.data.set_entry(entry_idx, entry);
            }
            // Deallocate the old palette
            self.palette = Vec::new();
        } else {
            for entry_idx in 0..old_buffer.entries {
                let entry = old_buffer.get_entry(entry_idx);
                self.data.set_entry(entry_idx, entry);
            }
        }
    }

    pub fn get_entry(&self, index: usize) -> u32 {
        if self.use_palette {
            self.palette[self.data.get_entry(index) as usize]
        } else {
            self.data.get_entry(index)
        }
    }

    pub fn set_entry(&mut self, index: usize, val: u32) {
        if self.use_palette {
            if let Some(palette_index) = self.palette.iter().position(|x| x == &val) {
                self.data.set_entry(index, palette_index as u32);
            } else {
                if self.palette.len() + 1 > self.max_entries as usize {
                    self.resize_buffer();
                    self.set_entry(index, val);
                    return;
                }
                let palette_index = self.palette.len();
                self.palette.push(val);
                self.data.set_entry(index, palette_index as u32);
            }
        } else {
            self.data.set_entry(index, val);
        }
    }

    pub fn entries(&self) -> usize {
        self.data.entries
    }
}

impl Default for PalettedBitBuffer {
    fn default() -> Self {
        Self::with_entries(4096)
    }
}

pub struct ChunkSection {
    buffer: PalettedBitBuffer,
    block_count: u32,
    multi_block: Vec<CMultiBlockChangeRecord>,
}

impl ChunkSection {
    fn get_index(x: u32, y: u32, z: u32) -> usize {
        ((y << 8) | (z << 4) | x) as usize
    }

    fn get_block(&self, x: u32, y: u32, z: u32) -> u32 {
        self.buffer.get_entry(ChunkSection::get_index(x, y, z))
    }

    /// Sets a block in the chunk sections. Returns true if a block was changed.
    fn set_block(&mut self, x: u32, y: u32, z: u32, block: u32) -> bool {
        let old_block = self.get_block(x, y, z);
        if old_block == 0 && block != 0 {
            self.block_count += 1;
        } else if old_block != 0 && block == 0 {
            self.block_count -= 1;
        }
        self.buffer
            .set_entry(ChunkSection::get_index(x, y, z), block);
        old_block != block
    }

    fn load(data: ChunkSectionData) -> ChunkSection {
        let loaded_longs = data.data.into_iter().map(|x| x as u64).collect();
        let bits_per_entry = data.bits_per_block as u8;
        let palette = data.palette.into_iter().map(|x| x as u32).collect();
        let buffer = PalettedBitBuffer::load(data.entries, bits_per_entry, loaded_longs, palette);
        ChunkSection {
            buffer,
            block_count: data.block_count as u32,
            multi_block: Vec::new(),
        }
    }

    fn save(&self) -> ChunkSectionData {
        let longs: Vec<i64> = self
            .buffer
            .data
            .longs
            .clone()
            .into_iter()
            .map(|x| x as i64)
            .collect();
        let palette: Vec<i32> = self
            .buffer
            .palette
            .clone()
            .into_iter()
            .map(|x| x as i32)
            .collect();
        ChunkSectionData {
            data: longs,
            palette,
            bits_per_block: self.buffer.data.bits_per_entry as i8,
            block_count: self.block_count as i32,
            entries: self.buffer.entries(),
        }
    }

    fn new() -> ChunkSection {
        ChunkSection {
            buffer: Default::default(),
            block_count: 0,
            multi_block: Vec::new(),
        }
    }

    fn encode_packet(&self) -> CChunkDataSection {
        CChunkDataSection {
            bits_per_block: self.buffer.data.bits_per_entry as u8,
            block_count: self.block_count as i16,
            data_array: self.buffer.data.longs.clone(),
            palette: if self.buffer.use_palette {
                Some(
                    self.buffer
                        .palette
                        .clone()
                        .into_iter()
                        .map(|x| x as i32)
                        .collect(),
                )
            } else {
                None
            },
        }
    }

    fn drain_multi_block(&mut self, chunk_x: i32, chunk_y: u32, chunk_z: i32) -> CMultiBlockChange {
        let multi_block = self.multi_block.drain(..).collect();
        CMultiBlockChange {
            records: multi_block,
            chunk_x,
            chunk_y,
            chunk_z,
        }
    }
}

pub struct Chunk {
    pub sections: BTreeMap<u8, ChunkSection>,
    pub x: i32,
    pub z: i32,
    pub block_entities: HashMap<BlockPos, BlockEntity>,
}

impl Chunk {
    pub fn encode_packet(&self, full_chunk: bool) -> PacketEncoder {
        let mut heightmap_buffer = BitBuffer::create(9, 256);
        for x in 0..16 {
            for z in 0..16 {
                heightmap_buffer
                    .set_entry((x * 16) + z, self.get_top_most_block(x as u32, z as u32));
            }
        }

        let mut chunk_sections = Vec::new();
        let mut bitmask = 0;
        for (section_y, section) in &self.sections {
            bitmask |= 1 << section_y;
            chunk_sections.push(section.encode_packet());
        }
        let mut heightmaps = nbt::Blob::new();
        let heightmap_longs: Vec<i64> = heightmap_buffer
            .longs
            .into_iter()
            .map(|x| x as i64)
            .collect();
        heightmaps
            .insert("MOTION_BLOCKING", heightmap_longs)
            .unwrap();
        let mut block_entities = Vec::new();
        self.block_entities
            .iter()
            .map(|(pos, block_entity)| {
                block_entity
                    .to_nbt(BlockPos::new(
                        pos.x + (self.x << 4),
                        pos.y,
                        pos.z + (self.z << 4),
                    ))
                    .map(|blob| block_entities.push(blob))
            })
            .for_each(drop);
        CChunkData {
            // Use `bool_to_option` feature when stabalized
            // Tracking issue: https://github.com/rust-lang/rust/issues/80967
            biomes: if full_chunk {
                Some(vec![0; 1024])
            } else {
                None
            },
            chunk_sections,
            chunk_x: self.x,
            chunk_z: self.z,
            full_chunk,
            heightmaps,
            primary_bit_mask: bitmask as i32,
            block_entities,
        }
        .encode()
    }

    fn get_top_most_block(&self, x: u32, z: u32) -> u32 {
        let mut top_most = 0;
        for (section_y, section) in &self.sections {
            for y in (0..16).rev() {
                let block_state = section.get_block(x, y, z);
                if block_state != 0 && top_most < y + *section_y as u32 * 16 {
                    top_most = *section_y as u32 * 16;
                }
            }
        }
        top_most
    }

    /// Sets a block in the chunk. Returns true if a block was changed.
    pub fn set_block_raw(&mut self, x: u32, y: u32, z: u32, block_id: u32) -> bool {
        let section_y = (y >> 4) as u8;
        if let Some(section) = self.sections.get_mut(&section_y) {
            section.set_block(x, y & 0xF, z, block_id)
        } else if block_id != 0 {
            let mut section = ChunkSection::new();
            section.set_block(x, y & 0xF, z, block_id);
            self.sections.insert(section_y, section);
            true
        } else {
            // The block was air so a new chunk section does not need to be created.
            false
        }
    }

    pub fn set_block(&mut self, x: u32, y: u32, z: u32, block_id: u32) -> bool {
        let changed = self.set_block_raw(x, y, z, block_id);
        if changed {
            let section_y = (y >> 4) as u8;
            self.sections.get_mut(&section_y).unwrap().multi_block.push(
                CMultiBlockChangeRecord {
                    block_id,
                    x: x as u8,
                    y: y as u8 & 0xF,
                    z: z as u8,
                },
            );
        }
        changed
    }

    pub fn get_block(&self, x: u32, y: u32, z: u32) -> u32 {
        let section_y = (y / 16) as u8;
        if let Some(section) = self.sections.get(&section_y) {
            section.get_block(x, y & 0xF, z)
        } else {
            0
        }
    }

    pub fn get_block_entity(&self, pos: BlockPos) -> Option<&BlockEntity> {
        self.block_entities.get(&pos)
    }

    pub fn delete_block_entity(&mut self, pos: BlockPos) {
        self.block_entities.remove(&pos);
    }

    pub fn set_block_entity(&mut self, pos: BlockPos, block_entity: BlockEntity) {
        self.block_entities.insert(pos, block_entity);
    }

    pub fn save(&self) -> ChunkData {
        ChunkData {
            sections: self.sections.iter().map(|(y, s)| (*y, s.save())).collect(),
            block_entities: self.block_entities.clone(),
        }
    }

    pub fn load(x: i32, z: i32, chunk_data: ChunkData) -> Chunk {
        Chunk {
            x,
            z,
            sections: chunk_data
                .sections
                .into_iter()
                .map(|(y, cs)| (y, ChunkSection::load(cs)))
                .collect(),
            block_entities: chunk_data.block_entities,
        }
    }

    pub fn empty(x: i32, z: i32) -> Chunk {
        Chunk {
            sections: BTreeMap::new(),
            x,
            z,
            block_entities: HashMap::new(),
        }
    }

    pub fn generate(layers: i32, x: i32, z: i32) -> Chunk {
        let mut chunk = Chunk {
            sections: BTreeMap::new(),
            x,
            z,
            block_entities: HashMap::new(),
        };

        for ry in 0..layers {
            for rx in 0..16 {
                for rz in 0..16 {
                    let block_x = (x << 4) | rx;
                    let block_z = (z << 4) | rz;

                    if block_x % 256 == 0
                        || block_z % 256 == 0
                        || (block_x + 1) % 256 == 0
                        || (block_z + 1) % 256 == 0
                    {
                        chunk.set_block(rx as u32, ry as u32, rz as u32, 4495); // Stone Bricks
                    } else {
                        chunk.set_block(rx as u32, ry as u32, rz as u32, 246); // Sandstone
                    }
                }
            }
        }
        chunk
    }

    pub fn drain_multi_block(&mut self) -> Vec<CMultiBlockChange> {
        let mut packets = Vec::new();
        for (y, section) in &mut self.sections {
            let packet = section.drain_multi_block(self.x, *y as u32, self.z);
            if !packet.records.is_empty() {
                packets.push(packet);
            }
        }
        packets
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ChunkSectionData {
    data: Vec<i64>,
    palette: Vec<i32>,
    bits_per_block: i8,
    block_count: i32,
    entries: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkData {
    sections: BTreeMap<u8, ChunkSectionData>,
    block_entities: HashMap<BlockPos, BlockEntity>,
}
