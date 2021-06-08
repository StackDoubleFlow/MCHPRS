use super::{PacketEncoder, PacketEncoderExt, SlotData};
use crate::player::Gamemode;
use crate::utils::NBTMap;
use serde::Serialize;
use std::collections::HashMap;

pub trait ClientBoundPacket {
    fn encode(self) -> PacketEncoder;
}

macro_rules! clientbound_packets {
    (
        $(
            $(#[$($struct_meta:meta),+])?
            $name:ident -> $packet_id:literal {
                $(pub $pfield:ident: $pfield_ty:ident,)*
                $(|$($field:ident: $field_ty:path),+| $field_write:ident($($field_expr:expr),+); )*
            },
        )*
    ) => {
        $(
            $(#[$($struct_meta),+])?
            pub struct $name {
                $($(pub $field: $field_ty, )*)*
                $(pub $pfield: $pfield_ty, )*
            }

            impl ClientBoundPacket for $name {
                fn encode(self) -> PacketEncoder {
                    let mut buf = Vec::new();
                    $( let $pfield = self.$pfield; )*
                    $($( let $field = self.$field; )*)*

                    $( buf.$field_write($($field_expr),*); )*

                    PacketEncoder::new(buf, $packet_id)
                }
            }
        )*
    };
}

clientbound_packets!(
    // Server List Ping Packets
    CResponse -> 0x00 {
        |json_response: String| write_string(32767, &json_response);
    },

    // Login Packets
    CDisconnectLogin -> 0x00 {
        |reason: String| write_string(32767, &reason);
    },
    CPong -> 0x01 {
        |payload: i64| write_long(payload);
    },
    CLoginSuccess -> 0x02 {
        |uuid: u128| write_uuid(uuid);
        |username: String| write_string(16, &username);
    },
    CSetCompression -> 0x03 {
        |threshold: i32| write_varint(threshold);
    },

    // Play Packets
    CSpawnEntity -> 0x00 {
        |entity_id: i32| write_varint(entity_id);
        |object_uuid: u128| write_uuid(object_uuid);
        |entity_type: i32| write_varint(entity_type);
        |x: f64| write_double(x);
        |y: f64| write_double(y);
        |z: f64| write_double(z);
        |yaw: f32| write_byte(((yaw / 360f32 * 256f32) as i32 % 256) as i8);
        |pitch: f32| write_byte(((pitch / 360f32 * 256f32) as i32 % 256) as i8);
        |data: i32| write_int(data);
        |velocity_x: i16| write_short(velocity_x);
        |velocity_y: i16| write_short(velocity_y);
        |velocity_z: i16| write_short(velocity_z);
    },
    CSpawnLivingEntity -> 0x00 {
        |entity_id: i32| write_varint(entity_id);
        |entity_uuid: u128| write_uuid(entity_uuid);
        |entity_type: i32| write_varint(entity_type);
        |x: f64| write_double(x);
        |y: f64| write_double(y);
        |z: f64| write_double(z);
        |yaw: f32| write_byte(((yaw / 360f32 * 256f32) as i32 % 256) as i8);
        |pitch: f32| write_byte(((pitch / 360f32 * 256f32) as i32 % 256) as i8);
        |head_pitch: f32| write_byte(((pitch / 360f32 * 256f32) as i32 % 256) as i8);
        |velocity_x: i16| write_short(velocity_x);
        |velocity_y: i16| write_short(velocity_y);
        |velocity_z: i16| write_short(velocity_z);
    },
    CSpawnPlayer -> 0x04 {
        pub on_ground: bool,

        |entity_id: i32| write_varint(entity_id);
        |uuid: u128| write_uuid(uuid);
        |x: f64| write_double(x);
        |y: f64| write_double(y);
        |z: f64| write_double(z);
        |yaw: f32| write_byte(((yaw / 360f32 * 256f32) as i32 % 256) as i8);
        |pitch: f32| write_byte(((pitch / 360f32 * 256f32) as i32 % 256) as i8);
    },
    CEntityAnimation -> 0x05 {
        |entity_id: i32| write_varint(entity_id);
        |animation: u8| write_unsigned_byte(animation);
    },
    CBlockEntityData -> 0x09 {
        |x: i32, y: i32, z: i32| write_position(x, y, z);
        |action: u8| write_unsigned_byte(action);
        |nbt: nbt::Blob| write_nbt_blob(nbt);
    },
    CBlockChange -> 0x0B {
        |x: i32, y: i32, z: i32| write_position(x, y, z);
        |block_id: i32| write_varint(block_id);
    },
    CChatMessage -> 0x0E {
        |message: String| write_string(32767, &message);
        |position: i8| write_byte(position);
        |sender: u128| write_uuid(sender);
    },
    CPluginMessage -> 0x17 {
        |channel: String| write_string(32767, &channel);
        |data: Vec<u8>| write_bytes(data);
    },
    CDisconnect -> 0x19 {
        |reason: String| write_string(32767, &reason);
    },
    CEntityStatus -> 0x1A {
        |entity_id: i32| write_int(entity_id);
        |status: CEntityStatuses| write_byte(match status {
            CEntityStatuses::LivingEntityGenericHurt => 2,
            CEntityStatuses::LivingEntityDeath => 3,
            CEntityStatuses::LivingEntityDrownHurt => 36,
            CEntityStatuses::LivingEntityFireHurt => 37,
            CEntityStatuses::LivingEntitySweetBerryBushHurt => 44,
        });
    },
    #[derive(Debug)]
    CUnloadChunk -> 0x1C {
        |chunk_x: i32| write_int(chunk_x);
        |chunk_z: i32| write_int(chunk_z);
    },
    CChangeGameState -> 0x1D {
        |reason: CChangeGameStateReason| write_unsigned_byte(match reason {
            CChangeGameStateReason::ChangeGamemode => 3,
            // CChangeGameStateReason::WinGame => buf.write_unsigned_byte(4),
            // CChangeGameStateReason::ArrowHitPlayer => buf.write_unsigned_byte(6),
            CChangeGameStateReason::ChangeRespawnScreenMode => 11,
        });
        |value: f32| write_float(value);
    },
    CKeepAlive -> 0x1F {
        |id: i64| write_long(id);
    },
    CEffect -> 0x21 {
        |effect_id: i32| write_int(effect_id);
        |x: i32, y: i32, z: i32| write_position(x, y, z);
        |data: i32| write_int(data);
        |disable_relative_volume: bool| write_bool(disable_relative_volume);
    },
    COpenSignEditor -> 0x2E {
        |pos_x: i32, pos_y: i32, pos_z: i32| write_position(pos_x, pos_y, pos_z);
    },
    CEntityPosition -> 0x27 {
        |entity_id: i32| write_varint(entity_id);
        |delta_x: i16| write_short(delta_x);
        |delta_y: i16| write_short(delta_y);
        |delta_z: i16| write_short(delta_z);
        |on_ground: bool| write_bool(on_ground);
    },
    CEntityPositionAndRotation -> 0x28 {
        |entity_id: i32| write_varint(entity_id);
        |delta_x: i16| write_short(delta_x);
        |delta_y: i16| write_short(delta_y);
        |delta_z: i16| write_short(delta_z);
        |yaw: f32| write_byte(((yaw / 360f32 * 256f32) as i32 % 256) as i8);
        |pitch: f32| write_byte(((pitch / 360f32 * 256f32) as i32 % 256) as i8);
        |on_ground: bool| write_bool(on_ground);
    },
    CEntityRotation -> 0x29 {
        |entity_id: i32| write_varint(entity_id);
        |yaw: f32| write_byte(((yaw / 360f32 * 256f32) as i32 % 256) as i8);
        |pitch: f32| write_byte(((pitch / 360f32 * 256f32) as i32 % 256) as i8);
        |on_ground: bool| write_bool(on_ground);
    },
    CEntityMovement -> 0x2A {
        |entity_id: i32| write_varint(entity_id);
    },
    CPlayerAbilities -> 0x30 {
        |flags: u8| write_unsigned_byte(flags);
        |fly_speed: f32| write_float(fly_speed);
        |fov_modifier: f32| write_float(fov_modifier);
    },
    CPlayerPositionAndLook -> 0x34 {
        |x: f64| write_double(x);
        |y: f64| write_double(y);
        |z: f64| write_double(z);
        |yaw: f32| write_float(yaw);
        |pitch: f32| write_float(pitch);
        |flags: u8| write_unsigned_byte(flags);
        |teleport_id: i32| write_varint(teleport_id);
    },
    CRespawn -> 0x39 {
        |dimension: CJoinGameDimensionElement| write_nbt(dimension);
        |world_name: String| write_string(32767, &world_name);
        |hashed_seed: i64| write_long(hashed_seed);
        |gamemode: u8| write_unsigned_byte(gamemode);
        |previous_gamemode: u8| write_unsigned_byte(previous_gamemode);
        |is_debug: bool| write_boolean(is_debug);
        |is_flat: bool| write_boolean(is_flat);
        |copy_metadata: bool| write_boolean(copy_metadata);
    },
    CEntityHeadLook -> 0x3A {
        |entity_id: i32| write_varint(entity_id);
        |yaw: f32| write_byte(((yaw / 360f32 * 256f32) as i32 % 256) as i8);
    },
    CHeldItemChange -> 0x3F {
        |slot: i8| write_byte(slot);
    },
    CUpdateViewPosition -> 0x40 {
        |chunk_x: i32| write_varint(chunk_x);
        |chunk_z: i32| write_varint(chunk_z);
    },
    CEntityTeleport -> 0x56 {
        |entity_id: i32| write_varint(entity_id);
        |x: f64| write_double(x);
        |y: f64| write_double(y);
        |z: f64| write_double(z);
        |yaw: f32| write_byte(((yaw / 360f32 * 256f32) as i32 % 256) as i8);
        |pitch: f32| write_byte(((pitch / 360f32 * 256f32) as i32 % 256) as i8);
        |on_ground: bool| write_bool(on_ground);
    },


    CTimeUpdate -> 0x4E {
        |world_age: i64| write_long(world_age);
        |time_of_day: i64| write_long(time_of_day);
    },
);

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
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.nodes.len() as i32);
        for node in self.nodes {
            buf.write_byte(node.flags);
            buf.write_varint(node.children.len() as i32);
            for child in node.children {
                buf.write_varint(child);
            }
            if let Some(redirect_node) = node.redirect_node {
                buf.write_varint(redirect_node);
            }
            if let Some(name) = node.name {
                buf.write_string(32767, &name);
            }
            if let Some(parser) = node.parser {
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
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_unsigned_byte(self.window_id);
        buf.write_short(self.slot_data.len() as i16);
        for slot_data in self.slot_data {
            if let Some(slot) = slot_data {
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
        }
        PacketEncoder::new(buf, 0x13)
    }
}

pub struct CSetSlot {
    pub window: i8,
    pub slot: i16,
    pub slot_data: Option<SlotData>,
}

impl ClientBoundPacket for CSetSlot {
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_byte(self.window);
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

pub enum CEntityStatuses {
    LivingEntityGenericHurt,
    LivingEntityDeath,
    LivingEntityDrownHurt,
    LivingEntityFireHurt,
    LivingEntitySweetBerryBushHurt,
}

pub enum CChangeGameStateReason {
    ChangeGamemode,
    // WinGame,
    // ArrowHitPlayer,
    ChangeRespawnScreenMode,
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
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_int(self.chunk_x);
        buf.write_int(self.chunk_z);
        buf.write_boolean(self.full_chunk);
        buf.write_varint(self.primary_bit_mask);
        let mut heightmaps = Vec::new();
        self.heightmaps.to_writer(&mut heightmaps).unwrap();
        buf.write_bytes(heightmaps);
        if let Some(biomes) = self.biomes {
            buf.write_varint(biomes.len() as i32);
            for biome in biomes {
                buf.write_varint(biome);
            }
        }
        let mut data = Vec::new();
        for chunk_section in self.chunk_sections {
            data.write_short(chunk_section.block_count);
            data.write_unsigned_byte(chunk_section.bits_per_block);
            if let Some(palette) = chunk_section.palette {
                data.write_varint(palette.len() as i32);
                for palette_entry in palette {
                    data.write_varint(palette_entry);
                }
            }
            data.write_varint(chunk_section.data_array.len() as i32);
            for long in chunk_section.data_array {
                data.write_long(long as i64);
            }
        }
        buf.write_varint(data.len() as i32);
        buf.write_bytes(data);
        // Number of block entities
        buf.write_varint(self.block_entities.len() as i32);
        for block_entity in self.block_entities {
            buf.write_nbt_blob(block_entity);
        }
        PacketEncoder::new(buf, 0x20)
    }
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct CJoinGameBiomeEffectsMoodSound {
    pub tick_delay: i32,
    pub offset: f32,
    pub sound: String,
    pub block_search_extent: i32,
}

#[derive(Serialize)]
pub struct CJoinGameBiomeEffects {
    pub sky_color: i32,
    pub water_fog_color: i32,
    pub fog_color: i32,
    pub water_color: i32,
    pub mood_sound: CJoinGameBiomeEffectsMoodSound,
}

#[derive(Serialize)]
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
    fn encode(self, buf: &mut Vec<u8>) {
        let mut dimension_map = NBTMap::new("minecraft:dimension_type".to_owned());
        for (name, element) in self.dimensions {
            dimension_map.push_element(name, element);
        }
        let mut biome_map = NBTMap::new("minecraft:worldgen/biome".to_owned());
        for (name, element) in self.biomes {
            biome_map.push_element(name, element);
        }
        let codec = CJoinGameDimensionCodecInner {
            dimensions: dimension_map,
            biomes: biome_map,
        };
        buf.write_nbt(codec);
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
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_int(self.entity_id);
        buf.write_bool(self.is_hardcore);
        buf.write_unsigned_byte(self.gamemode);
        buf.write_unsigned_byte(self.previous_gamemode);
        buf.write_varint(self.world_count);
        for world_name in self.world_names {
            buf.write_string(32767, &world_name);
        }
        self.dimension_codec.encode(&mut buf);
        buf.write_nbt(self.dimension);
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

/// The Notchain client only uses the entity dead event, so all other events are ignroed
pub struct CCombatEvent {
    pub player_id: i32,  // Target player
    pub entity_id: i32,  // Killer entity/player
    pub message: String, // Death message
}

impl ClientBoundPacket for CCombatEvent {
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(2); // entity dead event enum
        buf.write_varint(self.player_id);
        buf.write_int(self.entity_id);
        buf.write_string(32767, &self.message);
        PacketEncoder::new(buf, 0x31)
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
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        match self {
            CPlayerInfo::AddPlayer(ps) => {
                buf.write_varint(0);
                buf.write_varint(ps.len() as i32);
                for p in ps {
                    buf.write_uuid(p.uuid);
                    buf.write_string(16, &p.name);
                    buf.write_varint(p.properties.len() as i32);
                    for prop in p.properties {
                        buf.write_string(32767, &prop.name);
                        buf.write_string(32767, &prop.value);
                        buf.write_boolean(prop.signature.is_some());
                        if let Some(signature) = prop.signature {
                            buf.write_string(32767, &signature);
                        }
                    }
                    buf.write_varint(p.gamemode);
                    buf.write_varint(p.ping);
                    buf.write_boolean(p.display_name.is_some());
                    if let Some(display_name) = p.display_name {
                        buf.write_string(32767, &display_name);
                    }
                }
            }
            CPlayerInfo::UpdateGamemode(uuid, gamemode) => {
                buf.write_varint(1);
                buf.write_varint(1);
                buf.write_uuid(uuid);
                buf.write_varint(gamemode.get_id() as i32);
            }
            CPlayerInfo::RemovePlayer(uuids) => {
                buf.write_varint(4);
                buf.write_varint(uuids.len() as i32);
                for uuid in uuids {
                    buf.write_uuid(uuid);
                }
            }
        }
        PacketEncoder::new(buf, 0x32)
    }
}

pub struct CDestroyEntities {
    pub entity_ids: Vec<i32>,
}

impl ClientBoundPacket for CDestroyEntities {
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_ids.len() as i32);
        for entity_id in self.entity_ids {
            buf.write_varint(entity_id);
        }
        PacketEncoder::new(buf, 0x36)
    }
}

#[derive(Debug)]
pub struct CMultiBlockChangeRecord {
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
    pub records: Vec<CMultiBlockChangeRecord>,
}

impl ClientBoundPacket for CMultiBlockChange {
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        let pos = ((self.chunk_x as i64 & 0x3FFFFF) << 42)
            | ((self.chunk_z as i64 & 0x3FFFFF) << 20)
            | (self.chunk_y as i64 & 0xFFFFF);
        buf.write_long(pos);
        buf.write_bool(true); // Always inverse the preceding Update Light packet's "Trust Edges" bool
        buf.write_varint(self.records.len() as i32); // Length of record array
        for record in self.records {
            let long = ((record.block_id as u64) << 12)
                | ((record.x as u64) << 8)
                | ((record.z as u64) << 4)
                | (record.y as u64);
            buf.write_varlong(long as i64);
        }

        PacketEncoder::new(buf, 0x3B)
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
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        for entry in self.metadata {
            buf.write_unsigned_byte(entry.index);
            buf.write_varint(entry.metadata_type);
            buf.write_bytes(entry.value);
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
    fn encode(self) -> PacketEncoder {
        let mut buf = Vec::new();
        buf.write_varint(self.entity_id);
        for slot in self.equipment {
            buf.write_varint(slot.slot);
            if let Some(slot) = slot.item {
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
        }

        PacketEncoder::new(buf, 0x47)
    }
}
