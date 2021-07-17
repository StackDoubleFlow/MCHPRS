use super::{DecodeResult, PacketDecoderExt, SlotData};

pub trait ServerBoundPacketHandler {
    fn handle_handshake(&mut self, _packet: SHandshake, _player_idx: usize) {}
    fn handle_request(&mut self, _packet: SRequest, _player_idx: usize) {}
    fn handle_ping(&mut self, _packet: SPing, _player_idx: usize) {}
    fn handle_login_start(&mut self, _packet: SLoginStart, _player_idx: usize) {}
    fn handle_chat_message(&mut self, _packet: SChatMessage, _player_idx: usize) {}
    fn handle_client_settings(&mut self, _packet: SClientSettings, _player_idx: usize) {}
    fn handle_plugin_message(&mut self, _packet: SPluginMessage, _player_idx: usize) {}
    fn handle_keep_alive(&mut self, _packet: SKeepAlive, _player_idx: usize) {}
    fn handle_player_position(&mut self, _packet: SPlayerPosition, _player_idx: usize) {}
    fn handle_player_position_and_rotation(
        &mut self,
        _packet: SPlayerPositionAndRotation,
        _player_idx: usize,
    ) {
    }
    fn handle_player_rotation(&mut self, _packet: SPlayerRotation, _player_idx: usize) {}
    fn handle_player_movement(&mut self, _packet: SPlayerMovement, _player_idx: usize) {}
    fn handle_player_abilities(&mut self, _packet: SPlayerAbilities, _player_idx: usize) {}
    fn handle_player_digging(&mut self, _packet: SPlayerDigging, _player_idx: usize) {}
    fn handle_entity_action(&mut self, _packet: SEntityAction, _player_idx: usize) {}
    fn handle_animation(&mut self, _packet: SAnimation, _player_idx: usize) {}
    fn handle_player_block_placement(&mut self, _packet: SPlayerBlockPlacemnt, _player_idx: usize) {
    }
    fn handle_held_item_change(&mut self, _packet: SHeldItemChange, _player_idx: usize) {}
    fn handle_creative_inventory_action(
        &mut self,
        _packet: SCreativeInventoryAction,
        _player_idx: usize,
    ) {
    }
    fn handle_update_sign(&mut self, _packet: SUpdateSign, _player_idx: usize) {}
    fn handle_unknown(&mut self, _packet: SUnknown, _player_idx: usize) {}
}

pub trait ServerBoundPacket: Send {
    fn decode<T: PacketDecoderExt>(reader: &mut T) -> DecodeResult<Self>
    where
        Self: Sized;

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize);
}

pub struct SUnknown;

impl ServerBoundPacket for SUnknown {
    fn decode<T: PacketDecoderExt>(_: &mut T) -> DecodeResult<Self> {
        Ok(SUnknown)
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_unknown(*self, player_idx);
    }
}

pub struct SHandshake {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32,
}

impl ServerBoundPacket for SHandshake {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SHandshake {
            protocol_version: decoder.read_varint()?,
            server_address: decoder.read_string()?,
            server_port: decoder.read_unsigned_short()?,
            next_state: decoder.read_varint()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_handshake(*self, player_idx);
    }
}

pub struct SRequest;

impl ServerBoundPacket for SRequest {
    fn decode<T: PacketDecoderExt>(_decoder: &mut T) -> DecodeResult<Self> {
        Ok(SRequest)
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_request(*self, player_idx);
    }
}

pub struct SPing {
    pub payload: i64,
}

impl ServerBoundPacket for SPing {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SPing {
            payload: decoder.read_long()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_ping(*self, player_idx);
    }
}

pub struct SLoginStart {
    pub name: String,
}

impl ServerBoundPacket for SLoginStart {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SLoginStart {
            name: decoder.read_string()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_login_start(*self, player_idx);
    }
}

pub struct SChatMessage {
    pub message: String,
}

impl ServerBoundPacket for SChatMessage {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SChatMessage {
            message: decoder.read_string()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_chat_message(*self, player_idx);
    }
}

pub struct SClientSettings {
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: i32,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    pub main_hand: i32,
}

impl ServerBoundPacket for SClientSettings {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SClientSettings {
            locale: decoder.read_string()?,
            view_distance: decoder.read_byte()?,
            chat_mode: decoder.read_varint()?,
            chat_colors: decoder.read_bool()?,
            displayed_skin_parts: decoder.read_unsigned_byte()?,
            main_hand: decoder.read_varint()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_client_settings(*self, player_idx);
    }
}

pub struct SPluginMessage {
    pub channel: String,
    pub data: Vec<u8>,
}

impl ServerBoundPacket for SPluginMessage {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SPluginMessage {
            channel: decoder.read_string()?,
            data: PacketDecoderExt::read_to_end(decoder)?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_plugin_message(*self, player_idx);
    }
}

pub struct SKeepAlive {
    pub id: i64,
}

impl ServerBoundPacket for SKeepAlive {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SKeepAlive {
            id: decoder.read_long()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_keep_alive(*self, player_idx);
    }
}

pub struct SPlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

impl ServerBoundPacket for SPlayerPosition {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SPlayerPosition {
            x: decoder.read_double()?,
            y: decoder.read_double()?,
            z: decoder.read_double()?,
            on_ground: decoder.read_bool()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_player_position(*self, player_idx);
    }
}

pub struct SPlayerPositionAndRotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl ServerBoundPacket for SPlayerPositionAndRotation {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SPlayerPositionAndRotation {
            x: decoder.read_double()?,
            y: decoder.read_double()?,
            z: decoder.read_double()?,
            yaw: decoder.read_float()?,
            pitch: decoder.read_float()?,
            on_ground: decoder.read_bool()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_player_position_and_rotation(*self, player_idx);
    }
}

pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl ServerBoundPacket for SPlayerRotation {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SPlayerRotation {
            yaw: decoder.read_float()?,
            pitch: decoder.read_float()?,
            on_ground: decoder.read_bool()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_player_rotation(*self, player_idx);
    }
}

pub struct SPlayerMovement {
    pub on_ground: bool,
}

impl ServerBoundPacket for SPlayerMovement {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SPlayerMovement {
            on_ground: decoder.read_bool()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_player_movement(*self, player_idx);
    }
}

pub struct SPlayerAbilities {
    pub is_flying: bool,
}

impl ServerBoundPacket for SPlayerAbilities {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SPlayerAbilities {
            is_flying: decoder.read_byte()? != 0,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_player_abilities(*self, player_idx);
    }
}

pub struct SPlayerDigging {
    pub status: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: i8,
}

impl ServerBoundPacket for SPlayerDigging {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        let status = decoder.read_varint()?;
        let location = decoder.read_position()?;
        let face = decoder.read_byte()?;
        Ok(SPlayerDigging {
            x: location.0,
            y: location.1,
            z: location.2,
            status,
            face,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_player_digging(*self, player_idx);
    }
}

pub struct SEntityAction {
    pub entity_id: i32,
    pub action_id: i32,
    pub jump_boost: i32,
}

impl ServerBoundPacket for SEntityAction {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SEntityAction {
            entity_id: decoder.read_varint()?,
            action_id: decoder.read_varint()?,
            jump_boost: decoder.read_varint()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_entity_action(*self, player_idx);
    }
}

pub struct SAnimation {
    pub hand: i32,
}

impl ServerBoundPacket for SAnimation {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SAnimation {
            hand: decoder.read_varint()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_animation(*self, player_idx);
    }
}

pub struct SPlayerBlockPlacemnt {
    pub hand: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: i32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
    pub inside_block: bool,
}

impl ServerBoundPacket for SPlayerBlockPlacemnt {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        let hand = decoder.read_varint()?;
        let location = decoder.read_position()?;
        let face = decoder.read_varint()?;
        let cursor_x = decoder.read_float()?;
        let cursor_y = decoder.read_float()?;
        let cursor_z = decoder.read_float()?;
        let inside_block = decoder.read_bool()?;
        Ok(SPlayerBlockPlacemnt {
            x: location.0,
            y: location.1,
            z: location.2,
            hand,
            face,
            cursor_x,
            cursor_y,
            cursor_z,
            inside_block,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_player_block_placement(*self, player_idx);
    }
}

pub struct SHeldItemChange {
    pub slot: i16,
}

impl ServerBoundPacket for SHeldItemChange {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        Ok(SHeldItemChange {
            slot: decoder.read_short()?,
        })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_held_item_change(*self, player_idx);
    }
}

pub struct SCreativeInventoryAction {
    pub slot: i16,
    pub clicked_item: Option<SlotData>,
}

impl ServerBoundPacket for SCreativeInventoryAction {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        let slot = decoder.read_short()?;
        let clicked_item = if decoder.read_bool()? {
            Some(SlotData {
                item_id: decoder.read_varint()?,
                item_count: decoder.read_byte()?,
                nbt: decoder.read_nbt_blob()?,
            })
        } else {
            None
        };

        Ok(SCreativeInventoryAction { slot, clicked_item })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_creative_inventory_action(*self, player_idx);
    }
}

pub struct SUpdateSign {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub lines: [String; 4],
}

impl ServerBoundPacket for SUpdateSign {
    fn decode<T: PacketDecoderExt>(decoder: &mut T) -> DecodeResult<Self> {
        let (x, y, z) = decoder.read_position()?;
        let lines = [
            decoder.read_string()?,
            decoder.read_string()?,
            decoder.read_string()?,
            decoder.read_string()?,
        ];
        Ok(SUpdateSign { x, y, z, lines })
    }

    fn handle(self: Box<Self>, handler: &mut dyn ServerBoundPacketHandler, player_idx: usize) {
        handler.handle_update_sign(*self, player_idx);
    }
}
