use super::{PacketEncoder, PacketEncoderExt, SlotData};
use crate::player::Gamemode;
use crate::utils::NBTMap;
use serde::Serialize;
use std::collections::HashMap;

pub trait ClientBoundPacket {
    fn encode(&self) -> PacketEncoder;
}

// Server List Ping Packets

pub struct CResponse {
    pub json_response: String,
}

impl ClientBoundPacket for CResponse {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_string(32767, &self.json_response);
        PacketEncoder::new(buf, 0x00)
    }
}

// Login Packets

pub struct CDisconnectLogin {
    pub reason: String,
}

impl ClientBoundPacket for CDisconnectLogin {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_string(32767, &self.reason);
        PacketEncoder::new(buf, 0x00)
    }
}

pub struct CPong {
    pub payload: i64,
}

impl ClientBoundPacket for CPong {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_long(self.payload);
        PacketEncoder::new(buf, 0x01)
    }
}

pub struct CLoginSuccess {
    pub uuid: u128,
    pub username: String,
}

impl ClientBoundPacket for CLoginSuccess {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_uuid(self.uuid);
        buf.write_string(16, &self.username);
        PacketEncoder::new(buf, 0x02)
    }
}

pub struct CSetCompression {
    pub threshold: i32,
}

impl ClientBoundPacket for CSetCompression {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.threshold);
        PacketEncoder::new(buf, 0x03)
    }
}

pub struct CSpawnEntity {
    pub entity_id: i32,
    pub object_uuid: u128,
    pub entity_type: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pitch: f32,
    pub yaw: f32,
    pub data: i32,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

impl ClientBoundPacket for CSpawnEntity {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_uuid(self.object_uuid);
        buf.write_varint(self.entity_type);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_byte(((self.yaw / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_byte(((self.pitch / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_int(self.data);
        buf.write_short(self.velocity_x);
        buf.write_short(self.velocity_y);
        buf.write_short(self.velocity_z);
        PacketEncoder::new(buf, 0x00)
    }
}

pub struct CSpawnLivingEntity {
    pub entity_id: i32,
    pub entity_uuid: u128,
    pub entity_type: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub head_pitch: f32,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

impl ClientBoundPacket for CSpawnLivingEntity {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_uuid(self.entity_uuid);
        buf.write_varint(self.entity_type);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_byte(((self.yaw / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_byte(((self.pitch / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_byte(((self.head_pitch / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_short(self.velocity_x);
        buf.write_short(self.velocity_y);
        buf.write_short(self.velocity_z);
        PacketEncoder::new(buf, 0x02)
    }
}

pub struct CSpawnPlayer {
    pub entity_id: i32,
    pub uuid: u128,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl ClientBoundPacket for CSpawnPlayer {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_uuid(self.uuid);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_byte(((self.yaw / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_byte(((self.pitch / 360f32 * 256f32) as i32 % 256) as i8);
        PacketEncoder::new(buf, 0x04)
    }
}

// Play Packets

pub struct CEntityAnimation {
    pub entity_id: i32,
    pub animation: u8,
}

impl ClientBoundPacket for CEntityAnimation {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_unsigned_byte(self.animation);
        PacketEncoder::new(buf, 0x05)
    }
}

pub struct CBlockEntityData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub action: u8,
    pub nbt: nbt::Blob,
}

impl ClientBoundPacket for CBlockEntityData {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_position(self.x, self.y, self.z);
        buf.write_unsigned_byte(self.action);
        buf.write_nbt_blob(&self.nbt);
        PacketEncoder::new(buf, 0x09)
    }
}

pub struct CBlockChange {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_id: i32,
}

impl ClientBoundPacket for CBlockChange {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_position(self.x, self.y, self.z);
        buf.write_varint(self.block_id);
        PacketEncoder::new(buf, 0x0B)
    }
}

pub struct CChatMessage {
    pub message: String,
    pub position: i8,
    pub sender: u128,
}

impl ClientBoundPacket for CChatMessage {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_string(32767, &self.message);
        buf.write_byte(self.position);
        buf.write_uuid(self.sender);
        PacketEncoder::new(buf, 0x0E)
    }
}

pub enum CDeclareCommandsNodeParser {
    Entity(i8),
    Vec2,
    Vec3,
    Integer(i32, i32),
    Float(f32, f32),
    BlockPos,
    BlockState,
}

impl CDeclareCommandsNodeParser {
    fn write(&self, buf: &mut Vec<u8>) {
        use CDeclareCommandsNodeParser::*;
        match self {
            Entity(flags) => {
                buf.write_string(32767, "minecraft:entity");
                buf.write_byte(*flags);
            }
            Vec2 => buf.write_string(32767, "minecraft:vec2"),
            Vec3 => buf.write_string(32767, "minecraft:vec3"),
            BlockPos => buf.write_string(32767, "minecraft:block_pos"),
            BlockState => buf.write_string(32767, "minecraft:block_state"),
            Integer(min, max) => {
                buf.write_string(32767, "brigadier:integer");
                buf.write_byte(3); // Supply min and max value
                buf.write_int(*min);
                buf.write_int(*max);
            }
            Float(min, max) => {
                buf.write_string(32767, "brigadier:float");
                buf.write_byte(3);
                buf.write_float(*min);
                buf.write_float(*max);
            }
        }
    }
}

pub struct CDeclareCommandsNode {
    pub flags: i8,
    pub children: Vec<i32>,
    pub redirect_node: Option<i32>,
    pub name: Option<&'static str>,
    pub parser: Option<CDeclareCommandsNodeParser>,
}

pub struct CDeclareCommands {
    pub nodes: Vec<CDeclareCommandsNode>,
    pub root_index: i32,
}

impl ClientBoundPacket for CDeclareCommands {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.nodes.len() as i32);
        for node in &self.nodes {
            buf.write_byte(node.flags);
            buf.write_varint(node.children.len() as i32);
            for child in &node.children {
                buf.write_varint(*child);
            }
            if let Some(redirect_node) = node.redirect_node {
                buf.write_varint(redirect_node);
            }
            if let Some(name) = node.name {
                buf.write_string(32767, name);
            }
            if let Some(parser) = &node.parser {
                parser.write(&mut buf);
            }
        }
        buf.write_varint(self.root_index);
        PacketEncoder::new(buf, 0x10)
    }
}

pub struct CWindowItems {
    pub window_id: u8,
    pub slot_data: Vec<Option<SlotData>>,
}

impl ClientBoundPacket for CWindowItems {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_unsigned_byte(self.window_id);
        buf.write_short(self.slot_data.len() as i16);
        for slot_data in &self.slot_data {
            if let Some(slot) = slot_data {
                buf.write_bool(true);
                buf.write_varint(slot.item_id);
                buf.write_byte(slot.item_count);
                if let Some(nbt) = &slot.nbt {
                    buf.write_nbt_blob(nbt);
                } else {
                    buf.write_byte(0); // End tag
                }
            } else {
                buf.write_bool(false);
            }
        }
        PacketEncoder::new(buf, 0x13)
    }
}

pub struct CSetSlot {
    pub slot: i16,
    pub slot_data: Option<SlotData>,
}

impl ClientBoundPacket for CSetSlot {
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_byte(0);
        buf.write_short(self.slot);
        if let Some(slot) = self.slot_data {
            buf.write_bool(true);
            buf.write_varint(slot.item_id);
            buf.write_byte(slot.item_count);
            if let Some(nbt) = slot.nbt {
                buf.write_nbt_blob(nbt);
            } else {
                buf.write_byte(0); // End tag
            }
        } else {
            buf.write_bool(false);
        }
        PacketEncoder::new(buf, 0x15)
    }
}

pub struct CPluginMessage {
    pub channel: String,
    pub data: Vec<u8>,
}

impl ClientBoundPacket for CPluginMessage {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_string(32767, &self.channel);
        buf.write_bytes(&self.data);
        PacketEncoder::new(buf, 0x17)
    }
}

pub struct CDisconnect {
    pub reason: String,
}

impl ClientBoundPacket for CDisconnect {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_string(32767, &self.reason);
        PacketEncoder::new(buf, 0x19)
    }
}

#[derive(Debug)]
pub struct CUnloadChunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl ClientBoundPacket for CUnloadChunk {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_int(self.chunk_x);
        buf.write_int(self.chunk_z);
        PacketEncoder::new(buf, 0x1C)
    }
}

pub enum CChangeGameStateReason {
    ChangeGamemode,
}

pub struct CChangeGameState {
    pub reason: CChangeGameStateReason,
    pub value: f32,
}

impl ClientBoundPacket for CChangeGameState {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        match self.reason {
            CChangeGameStateReason::ChangeGamemode => buf.write_unsigned_byte(3),
        }
        buf.write_float(self.value);
        PacketEncoder::new(buf, 0x1D)
    }
}

pub struct CKeepAlive {
    pub id: i64,
}

impl ClientBoundPacket for CKeepAlive {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_long(self.id);
        PacketEncoder::new(buf, 0x1F)
    }
}

pub struct CChunkDataSection {
    pub block_count: i16,
    pub bits_per_block: u8,
    pub palette: Option<Vec<i32>>,
    pub data_array: Vec<u64>,
}

pub struct CChunkData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub full_chunk: bool,
    pub primary_bit_mask: i32,
    pub heightmaps: nbt::Blob,
    pub biomes: Option<Vec<i32>>,
    pub chunk_sections: Vec<CChunkDataSection>,
    pub block_entities: Vec<nbt::Blob>,
}

impl ClientBoundPacket for CChunkData {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_int(self.chunk_x);
        buf.write_int(self.chunk_z);
        buf.write_boolean(self.full_chunk);
        buf.write_varint(self.primary_bit_mask);
        let mut heightmaps = Vec::new();
        self.heightmaps.to_writer(&mut heightmaps).unwrap();
        buf.write_bytes(&heightmaps);
        if let Some(biomes) = &self.biomes {
            buf.write_varint(biomes.len() as i32);
            for biome in biomes {
                buf.write_varint(*biome);
            }
        }
        let mut data = Vec::new();
        for chunk_section in &self.chunk_sections {
            data.write_short(chunk_section.block_count);
            data.write_unsigned_byte(chunk_section.bits_per_block);
            if let Some(palette) = &chunk_section.palette {
                data.write_varint(palette.len() as i32);
                for palette_entry in palette {
                    data.write_varint(*palette_entry);
                }
            }
            data.write_varint(chunk_section.data_array.len() as i32);
            for long in &chunk_section.data_array {
                data.write_long(*long as i64);
            }
        }
        buf.write_varint(data.len() as i32);
        buf.write_bytes(&data);
        // Number of block entities
        buf.write_varint(self.block_entities.len() as i32);
        for block_entity in &self.block_entities {
            buf.write_nbt_blob(block_entity);
        }
        PacketEncoder::new(buf, 0x20)
    }
}

pub struct CEffect {
    pub effect_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub data: i32,
    pub disable_relative_volume: bool,
}

impl ClientBoundPacket for CEffect {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_int(self.effect_id);
        buf.write_position(self.x, self.y, self.z);
        buf.write_int(self.data);
        buf.write_bool(self.disable_relative_volume);
        PacketEncoder::new(buf, 0x21)
    }
}

#[derive(Serialize, Clone)]
pub struct CJoinGameDimensionElement {
    pub natural: i8,
    pub ambient_light: f32,
    pub has_ceiling: i8,
    pub has_skylight: i8,
    pub fixed_time: i64,
    pub shrunk: i8,
    pub ultrawarm: i8,
    pub has_raids: i8,
    pub respawn_anchor_works: i8,
    pub bed_works: i8,
    pub piglin_safe: i8,
    pub coordinate_scale: f32,
    pub logical_height: i32,
    pub infiniburn: String,
}

#[derive(Serialize, Clone)]
pub struct CJoinGameBiomeEffectsMoodSound {
    pub tick_delay: i32,
    pub offset: f32,
    pub sound: String,
    pub block_search_extent: i32,
}

#[derive(Serialize, Clone)]
pub struct CJoinGameBiomeEffects {
    pub sky_color: i32,
    pub water_fog_color: i32,
    pub fog_color: i32,
    pub water_color: i32,
    pub mood_sound: CJoinGameBiomeEffectsMoodSound,
}

#[derive(Serialize, Clone)]
pub struct CJoinGameBiomeElement {
    pub depth: f32,
    pub temperature: f32,
    pub downfall: f32,
    pub precipitation: String,
    pub category: String,
    pub scale: f32,
    pub effects: CJoinGameBiomeEffects,
}

pub struct CJoinGameDimensionCodec {
    pub dimensions: HashMap<String, CJoinGameDimensionElement>,
    pub biomes: HashMap<String, CJoinGameBiomeElement>,
}

#[derive(Serialize)]
struct CJoinGameDimensionCodecInner {
    #[serde(rename = "minecraft:dimension_type")]
    pub dimensions: NBTMap<CJoinGameDimensionElement>,
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biomes: NBTMap<CJoinGameBiomeElement>,
}

impl CJoinGameDimensionCodec {
    fn encode(&self, buf: &mut Vec<u8>) {
        let mut dimension_map: NBTMap<CJoinGameDimensionElement> =
            NBTMap::new("minecraft:dimension_type".to_owned());
        for (name, element) in &self.dimensions {
            dimension_map.push_element(name.clone(), element.clone());
        }
        let mut biome_map = NBTMap::new("minecraft:worldgen/biome".to_owned());
        for (name, element) in &self.biomes {
            biome_map.push_element(name.clone(), element.clone());
        }
        let codec = CJoinGameDimensionCodecInner {
            dimensions: dimension_map,
            biomes: biome_map,
        };
        buf.write_nbt(&codec);
    }
}

pub struct CJoinGame {
    pub entity_id: i32,
    pub is_hardcore: bool,
    pub gamemode: u8,
    pub previous_gamemode: u8,
    pub world_count: i32,
    pub world_names: Vec<String>,
    pub dimension_codec: CJoinGameDimensionCodec,
    pub dimension: CJoinGameDimensionElement,
    pub world_name: String,
    pub hashed_seed: i64,
    pub max_players: i32,
    pub view_distance: i32,
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
    pub is_debug: bool,
    pub is_flat: bool,
}

impl ClientBoundPacket for CJoinGame {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_int(self.entity_id);
        buf.write_bool(self.is_hardcore);
        buf.write_unsigned_byte(self.gamemode);
        buf.write_unsigned_byte(self.previous_gamemode);
        buf.write_varint(self.world_count);
        for world_name in &self.world_names {
            buf.write_string(32767, world_name);
        }
        self.dimension_codec.encode(&mut buf);
        buf.write_nbt(&self.dimension);
        buf.write_string(32767, &self.world_name);
        buf.write_long(self.hashed_seed);
        buf.write_varint(self.max_players);
        buf.write_varint(self.view_distance);
        buf.write_boolean(self.reduced_debug_info);
        buf.write_boolean(self.enable_respawn_screen);
        buf.write_boolean(self.is_debug);
        buf.write_boolean(self.is_flat);
        PacketEncoder::new(buf, 0x24)
    }
}

pub struct COpenSignEditor {
    pub pos_x: i32,
    pub pos_y: i32,
    pub pos_z: i32,
}

impl ClientBoundPacket for COpenSignEditor {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_position(self.pos_x, self.pos_y, self.pos_z);
        PacketEncoder::new(buf, 0x2E)
    }
}

pub struct CEntityPosition {
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

impl ClientBoundPacket for CEntityPosition {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_short(self.delta_x);
        buf.write_short(self.delta_y);
        buf.write_short(self.delta_z);
        buf.write_bool(self.on_ground);
        PacketEncoder::new(buf, 0x27)
    }
}

pub struct CEntityPositionAndRotation {
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl ClientBoundPacket for CEntityPositionAndRotation {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_short(self.delta_x);
        buf.write_short(self.delta_y);
        buf.write_short(self.delta_z);
        buf.write_byte(((self.yaw / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_byte(((self.pitch / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_bool(self.on_ground);
        PacketEncoder::new(buf, 0x28)
    }
}

pub struct CEntityRotation {
    pub entity_id: i32,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl ClientBoundPacket for CEntityRotation {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_byte(((self.yaw / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_byte(((self.pitch / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_bool(self.on_ground);
        PacketEncoder::new(buf, 0x29)
    }
}

pub struct CEntityMovement {
    pub entity_id: i32,
}

impl ClientBoundPacket for CEntityMovement {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        PacketEncoder::new(buf, 0x2A)
    }
}

pub struct CPlayerAbilities {
    pub flags: u8,
    pub fly_speed: f32,
    pub fov_modifier: f32,
}

impl ClientBoundPacket for CPlayerAbilities {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_unsigned_byte(self.flags);
        buf.write_float(self.fly_speed);
        buf.write_float(self.fov_modifier);
        PacketEncoder::new(buf, 0x30)
    }
}

pub struct CPlayerInfoAddPlayerProperty {
    name: String,
    value: String,
    signature: Option<String>,
}

pub struct CPlayerInfoAddPlayer {
    pub uuid: u128,
    pub name: String,
    pub properties: Vec<CPlayerInfoAddPlayerProperty>,
    pub gamemode: i32,
    pub ping: i32,
    pub display_name: Option<String>,
}

pub enum CPlayerInfo {
    AddPlayer(Vec<CPlayerInfoAddPlayer>),
    RemovePlayer(Vec<u128>),
    UpdateGamemode(u128, Gamemode),
}

impl ClientBoundPacket for CPlayerInfo {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        match self {
            CPlayerInfo::AddPlayer(ps) => {
                buf.write_varint(0);
                buf.write_varint(ps.len() as i32);
                for p in ps {
                    buf.write_uuid(p.uuid);
                    buf.write_string(16, &p.name);
                    buf.write_varint(p.properties.len() as i32);
                    for prop in &p.properties {
                        buf.write_string(32767, &prop.name);
                        buf.write_string(32767, &prop.value);
                        buf.write_boolean(prop.signature.is_some());
                        if let Some(signature) = &prop.signature {
                            buf.write_string(32767, signature);
                        }
                    }
                    buf.write_varint(p.gamemode);
                    buf.write_varint(p.ping);
                    buf.write_boolean(p.display_name.is_some());
                    if let Some(display_name) = &p.display_name {
                        buf.write_string(32767, display_name);
                    }
                }
            }
            CPlayerInfo::UpdateGamemode(uuid, gamemode) => {
                buf.write_varint(1);
                buf.write_varint(1);
                buf.write_uuid(*uuid);
                buf.write_varint(gamemode.get_id() as i32);
            }
            CPlayerInfo::RemovePlayer(uuids) => {
                buf.write_varint(4);
                buf.write_varint(uuids.len() as i32);
                for uuid in uuids {
                    buf.write_uuid(*uuid);
                }
            }
        }
        PacketEncoder::new(buf, 0x32)
    }
}

pub struct CPlayerPositionAndLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
    pub teleport_id: i32,
}

impl ClientBoundPacket for CPlayerPositionAndLook {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_float(self.yaw);
        buf.write_float(self.pitch);
        buf.write_unsigned_byte(self.flags);
        buf.write_varint(self.teleport_id);
        PacketEncoder::new(buf, 0x34)
    }
}

pub struct CDestroyEntities {
    pub entity_ids: Vec<i32>,
}

impl ClientBoundPacket for CDestroyEntities {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_ids.len() as i32);
        for entity_id in &self.entity_ids {
            buf.write_varint(*entity_id);
        }
        PacketEncoder::new(buf, 0x36)
    }
}

pub struct CEntityHeadLook {
    pub entity_id: i32,
    pub yaw: f32,
}

impl ClientBoundPacket for CEntityHeadLook {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_byte(((self.yaw / 360f32 * 256f32) as i32 % 256) as i8);
        PacketEncoder::new(buf, 0x3A)
    }
}

#[derive(Debug)]
pub struct C3BMultiBlockChangeRecord {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub block_id: u32,
}

#[derive(Debug)]
pub struct CMultiBlockChange {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub chunk_y: u32,
    pub records: Vec<C3BMultiBlockChangeRecord>,
}

impl ClientBoundPacket for CMultiBlockChange {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::with_capacity(self.records.len() * 8 + 12);
        let pos = ((self.chunk_x as i64 & 0x3FFFFF) << 42)
            | ((self.chunk_z as i64 & 0x3FFFFF) << 20)
            | (self.chunk_y as i64 & 0xFFFFF);
        buf.write_long(pos);
        buf.write_bool(true); // Always inverse the preceding Update Light packet's "Trust Edges" bool
        buf.write_varint(self.records.len() as i32); // Length of record array
        for record in &self.records {
            let long = ((record.block_id as u64) << 12)
                | ((record.x as u64) << 8)
                | ((record.z as u64) << 4)
                | (record.y as u64);
            buf.write_varlong(long as i64);
        }

        PacketEncoder::new(buf, 0x3B)
    }
}

pub struct CHeldItemChange {
    pub slot: i8,
}

impl ClientBoundPacket for CHeldItemChange {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_byte(self.slot);
        PacketEncoder::new(buf, 0x3F)
    }
}

pub struct CUpdateViewPosition {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl ClientBoundPacket for CUpdateViewPosition {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.chunk_x);
        buf.write_varint(self.chunk_z);
        PacketEncoder::new(buf, 0x40)
    }
}

pub struct CEntityMetadataEntry {
    pub index: u8,
    pub metadata_type: i32,
    pub value: Vec<u8>,
}

pub struct CEntityMetadata {
    pub entity_id: i32,
    pub metadata: Vec<CEntityMetadataEntry>,
}

impl ClientBoundPacket for CEntityMetadata {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        for entry in &self.metadata {
            buf.write_unsigned_byte(entry.index);
            buf.write_varint(entry.metadata_type);
            buf.write_bytes(&entry.value);
        }
        buf.write_byte(-1); // 0xFF
        PacketEncoder::new(buf, 0x44)
    }
}

pub struct CEntityEquipmentEquipment {
    pub slot: i32,
    pub item: Option<SlotData>,
}

pub struct CEntityEquipment {
    pub entity_id: i32,
    pub equipment: Vec<CEntityEquipmentEquipment>,
}

impl ClientBoundPacket for CEntityEquipment {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        for slot in &self.equipment {
            buf.write_varint(slot.slot);
            if let Some(slot) = &slot.item {
                buf.write_bool(true);
                buf.write_varint(slot.item_id);
                buf.write_byte(slot.item_count);
                if let Some(nbt) = &slot.nbt {
                    buf.write_nbt_blob(nbt);
                } else {
                    buf.write_byte(0); // End tag
                }
            } else {
                buf.write_bool(false);
            }
        }

        PacketEncoder::new(buf, 0x47)
    }
}

pub struct CTimeUpdate {
    pub world_age: i64,
    pub time_of_day: i64,
}

impl ClientBoundPacket for CTimeUpdate {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_long(self.world_age);
        buf.write_long(self.time_of_day);
        PacketEncoder::new(buf, 0x4E)
    }
}

pub struct CEntityTeleport {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl ClientBoundPacket for CEntityTeleport {
    fn encode(&self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_byte(((self.yaw / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_byte(((self.pitch / 360f32 * 256f32) as i32 % 256) as i8);
        buf.write_bool(self.on_ground);
        PacketEncoder::new(buf, 0x56)
    }
}
