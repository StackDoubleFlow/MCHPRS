mod commands;
mod packets;
mod storage;
mod worldedit;

use crate::blocks::{Block, BlockPos};
use crate::network::packets::clientbound::*;
use crate::player::Player;
use crate::server::{Message, PrivMessage};
use bus::BusReader;
use log::debug;
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use storage::{Chunk, ChunkData, PlotData};

pub struct Plot {
    players: Vec<Player>,
    tps: u32,
    message_receiver: BusReader<Message>,
    message_sender: Sender<Message>,
    priv_message_receiver: Receiver<PrivMessage>,
    last_player_time: SystemTime,
    running: bool,
    x: i32,
    z: i32,
    show_redstone: bool,
    always_running: bool,
    chunks: Vec<Chunk>,
}

impl Plot {
    fn get_chunk_index_for_chunk(&self, chunk_x: i32, chunk_z: i32) -> usize {
        let local_x = chunk_x - self.x * 8;
        let local_z = chunk_z - self.z * 8;
        (local_x * 8 + local_z).abs() as usize
    }

    fn get_chunk_index_for_block(&self, block_x: i32, block_z: i32) -> usize {
        let chunk_x = (block_x - self.x * 128) >> 4;
        let chunk_z = (block_z - self.z * 128) >> 4;
        (chunk_x * 8 + chunk_z).abs() as usize
    }

    /// Sets a block in storage without sending a block change packet to the client. Returns true if a block was changed.
    fn set_block_raw(&mut self, pos: &BlockPos, block: u32) -> bool {
        let chunk_index = self.get_chunk_index_for_block(pos.x, pos.z);
        let chunk = &mut self.chunks[chunk_index];
        chunk.set_block((pos.x & 0xF) as u32, pos.y, (pos.z & 0xF) as u32, block)
    }

    /// Returns true if a block was changed
    pub fn set_block(&mut self, pos: &BlockPos, block: Block) -> bool {
        let block_id = Block::get_id(block);
        let changed = self.set_block_raw(pos, block_id);
        if changed {
            self.send_block_change(pos, block_id);
        }
        changed
    }

    pub fn get_block(&mut self, pos: &BlockPos) -> Block {
        let chunk_index = self.get_chunk_index_for_block(pos.x, pos.z);
        let chunk = &self.chunks[chunk_index];
        Block::from_block_state(chunk.get_block((pos.x & 0xF) as u32, pos.y, (pos.z & 0xF) as u32))
    }

    pub fn send_block_change(&mut self, pos: &BlockPos, id: u32) {
        let block_change = C0CBlockChange {
            block_id: id as i32,
            x: pos.x,
            y: pos.y as i32,
            z: pos.z,
        }
        .encode();
        for player in &mut self.players {
            player.client.send_packet(&block_change);
        }
    }

    fn tick(&mut self) {}

    fn enter_plot(&mut self, mut player: Player) {
        debug!("Player enter plot!");
        self.save();
        for chunk in &self.chunks {
            player.client.send_packet(&chunk.encode_packet(true));
        }
        let spawn_player = C05SpawnPlayer {
            entity_id: player.entity_id as i32,
            uuid: player.uuid,
            on_ground: player.on_ground,
            pitch: player.pitch,
            yaw: player.yaw,
            x: player.x,
            y: player.y,
            z: player.z,
        }
        .encode();
        let mut metadata_entries = Vec::new();
        metadata_entries.push(C44EntityMetadataEntry {
            index: 16,
            metadata_type: 0,
            value: vec![player.skin_parts.bits() as u8],
        });
        let metadata = C44EntityMetadata {
            entity_id: player.entity_id as i32,
            metadata: metadata_entries,
        }
        .encode();
        for other_player in &mut self.players {
            other_player.client.send_packet(&spawn_player);
            other_player.client.send_packet(&metadata);

            let spawn_other_player = C05SpawnPlayer {
                entity_id: other_player.entity_id as i32,
                uuid: other_player.uuid,
                on_ground: other_player.on_ground,
                pitch: other_player.pitch,
                yaw: other_player.yaw,
                x: other_player.x,
                y: other_player.y,
                z: other_player.z,
            }
            .encode();
            player.client.send_packet(&spawn_other_player);

            let mut other_metadata_entries = Vec::new();
            other_metadata_entries.push(C44EntityMetadataEntry {
                index: 16,
                metadata_type: 0,
                value: vec![other_player.skin_parts.bits() as u8],
            });
            let other_metadata = C44EntityMetadata {
                entity_id: other_player.entity_id as i32,
                metadata: other_metadata_entries,
            }
            .encode();
            player.client.send_packet(&other_metadata);
        }
        player.send_system_message(&format!("Entering plot ({}, {})", self.x, self.z));
        self.players.push(player);
    }

    fn destroy_entity(&mut self, entity_id: u32) {
        let destroy_entities = C38DestroyEntities {
            entity_ids: vec![entity_id as i32],
        }
        .encode();
        for player in &mut self.players {
            player.client.send_packet(&destroy_entities);
        }
    }

    fn leave_plot(&mut self, player_index: usize) -> Player {
        let mut player = self.players.remove(player_index);
        let mut entity_ids = Vec::new();
        for player in &self.players {
            entity_ids.push(player.entity_id as i32);
        }
        let destroy_other_entities = C38DestroyEntities {
            entity_ids,
        }.encode();
        player.client.send_packet(&destroy_other_entities);
        self.destroy_entity(player.entity_id);
        player
    }

    fn in_plot_bounds(plot_x: i32, plot_z: i32, x: i32, z: i32) -> bool {
        x >= plot_x * 128 && x < (plot_x + 1) * 128 && z >= plot_z * 128 && z < (plot_z + 1) * 128
    }

    fn update(&mut self) {
        // Handle messages from the message channel
        while let Ok(message) = self.message_receiver.try_recv() {
            match message {
                Message::Chat(message) => {
                    for player in &mut self.players {
                        player.send_raw_chat(message.clone());
                    }
                }
                Message::PlayerJoinedInfo(player_join_info) => {
                    let player_info = C34PlayerInfo::AddPlayer(vec![C34PlayerInfoAddPlayer {
                        name: player_join_info.username,
                        properties: Vec::new(),
                        gamemode: 1,
                        ping: 0,
                        uuid: player_join_info.uuid,
                        display_name: None,
                    }])
                    .encode();
                    for player in &mut self.players {
                        player.client.send_packet(&player_info);
                    }
                }
                Message::PlayerLeft(uuid) => {
                    let player_info = C34PlayerInfo::RemovePlayer(vec![uuid]).encode();
                    for player in &mut self.players {
                        player.client.send_packet(&player_info);
                    }
                }
                Message::Shutdown => {
                    let mut players: Vec<Player> = self.players.drain(..).collect();
                    for player in players.iter_mut() {
                        player.kick(
                            json!({
                                "text": "Server closed"
                            })
                            .to_string(),
                        );
                    }
                    self.always_running = false;
                    self.running = false;
                    return;
                }
                _ => {}
            }
        }
        // Handle messages from the private message channel
        while let Ok(message) = self.priv_message_receiver.try_recv() {
            match message {
                PrivMessage::PlayerEnterPlot(player) => {
                    self.enter_plot(player);
                }
                PrivMessage::PlayerTeleportOther(mut player, username) => {
                    if let Some(other) = self.players.iter().find(|p| p.username == username) {
                        player.teleport(other.x, other.y, other.z);
                    }
                    self.enter_plot(player);
                }
            }
        }
        // Only tick if there are players in the plot
        if !self.players.is_empty() {
            self.last_player_time = SystemTime::now();
            self.tick();
        } else {
            // Unload plot after 600 seconds unless the plot should be always loaded
            if self.last_player_time.elapsed().unwrap().as_secs() > 600 && !self.always_running {
                self.running = false;
            }
        }
        // Update players
        for player in &mut self.players {
            player.update();
        }
        // Handle received packets
        for player in 0..self.players.len() {
            self.handle_packets_for_player(player);
        }

        let message_sender = &mut self.message_sender;

        // Remove disconnected players
        let mut disconnected_players = Vec::new();
        self.players.retain(|player| {
            let alive = player.client.alive;
            if !alive {
                player.save();
                message_sender
                    .send(Message::PlayerLeft(player.uuid))
                    .unwrap();
                disconnected_players.push(player.entity_id);
            }
            alive
        });
        for entity_id in disconnected_players {
            self.destroy_entity(entity_id);
        }

        // Remove players outside of the plot
        let mut outside_players = Vec::new();
        for player in 0..self.players.len() {
            if !Plot::in_plot_bounds(
                self.x,
                self.z,
                self.players[player].x as i32,
                self.players[player].z as i32,
            ) {
                outside_players.push(player);
            }
        }
        for player_index in outside_players {
            let player = self.leave_plot(player_index);
            let player_leave_plot = Message::PlayerLeavePlot(Arc::from(player));
            self.message_sender.send(player_leave_plot).unwrap();
        }
    }

    fn load_from_file(
        data: Vec<u8>,
        x: i32,
        z: i32,
        rx: BusReader<Message>,
        tx: Sender<Message>,
        priv_rx: Receiver<PrivMessage>,
        always_running: bool,
    ) -> Plot {
        let chunk_x_offset = x << 3;
        let chunk_z_offset = z << 3;
        let plot_data: PlotData = bincode::deserialize(&data).unwrap();
        let chunks: Vec<Chunk> = plot_data
            .chunk_data
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                Chunk::load(
                    chunk_x_offset + i as i32 / 8,
                    chunk_z_offset + i as i32 % 8,
                    c,
                )
            })
            .collect();
        Plot {
            last_player_time: SystemTime::now(),
            message_receiver: rx,
            message_sender: tx,
            priv_message_receiver: priv_rx,
            players: Vec::new(),
            running: true,
            show_redstone: plot_data.show_redstone,
            tps: plot_data.tps as u32,
            x,
            z,
            always_running,
            chunks,
        }
    }

    fn load(
        x: i32,
        z: i32,
        rx: BusReader<Message>,
        tx: Sender<Message>,
        priv_rx: Receiver<PrivMessage>,
        always_running: bool,
    ) -> Plot {
        if let Ok(data) = fs::read(format!("./world/plots/p{},{}", x, z)) {
            Plot::load_from_file(data, x, z, rx, tx, priv_rx, always_running)
        } else if Path::new("./world/plots/pTEMPLATE").exists() {
            let data = fs::read("./world/plots/pTEMPLATE").unwrap();
            Plot::load_from_file(data, x, z, rx, tx, priv_rx, always_running)
        } else {
            debug!("Plot {},{} does not exist yet, generating now.", x, z);
            let chunk_x_offset = x << 3;
            let chunk_z_offset = z << 3;
            let mut chunks = Vec::new();
            for chunk_x in 0..8 {
                for chunk_z in 0..8 {
                    chunks.push(Chunk::generate(
                        8,
                        chunk_x + chunk_x_offset,
                        chunk_z + chunk_z_offset,
                    ));
                }
            }
            Plot {
                last_player_time: SystemTime::now(),
                message_receiver: rx,
                message_sender: tx,
                priv_message_receiver: priv_rx,
                players: Vec::new(),
                running: true,
                show_redstone: true,
                tps: 20,
                x,
                z,
                always_running,
                chunks,
            }
        }
    }

    fn save(&self) {
        debug!("Saving plot {},{}", self.x, self.z);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("./world/plots/p{},{}", self.x, self.z))
            .unwrap();
        let chunk_data: Vec<ChunkData> = self.chunks.iter().map(|c| c.save()).collect();
        let encoded: Vec<u8> = bincode::serialize(&PlotData {
            tps: self.tps as i32,
            show_redstone: self.show_redstone,
            chunk_data,
        })
        .unwrap();
        file.write_all(&encoded).unwrap();
        file.sync_data().unwrap();
    }

    fn run(&mut self, initial_player: Option<Player>) {
        debug!("Running new plot!");
        if let Some(player) = initial_player {
            debug!("Sending initial player into plot!");
            self.enter_plot(player);
        }
        while self.running {
            self.update();
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn load_and_run(
        x: i32,
        z: i32,
        rx: BusReader<Message>,
        tx: Sender<Message>,
        priv_rx: Receiver<PrivMessage>,
        always_running: bool,
        initial_player: Option<Player>,
    ) {
        let mut plot = Plot::load(x, z, rx, tx, priv_rx, always_running);
        thread::Builder::new()
            .name(format!("p{},{}", x, z))
            .spawn(move || {
                plot.run(initial_player);
            })
            .unwrap();
    }
}

impl Drop for Plot {
    fn drop(&mut self) {
        if !self.players.is_empty() {
            // TODO: send all players to spawn and send them message along the lines of:
            // "The plot you were previously in has crashed, you have been teleported to the spawn plot."
            for player in &mut self.players {
                player.send_system_message("The plot you were previously in has crashed!");
            }
        }
        self.save();
        debug!("Plot {},{} unloaded", self.x, self.z);
        self.message_sender
            .send(Message::PlotUnload(self.x, self.z))
            .unwrap();
    }
}

#[test]
fn chunk_save_and_load_test() {
    let mut chunk = Chunk::empty(1, 1);
    chunk.set_block(13, 63, 12, 332);
    chunk.set_block(13, 62, 12, 331);
    let chunk_data = chunk.save();
    let loaded_chunk = Chunk::load(1, 1, chunk_data);
    assert_eq!(loaded_chunk.get_block(13, 63, 12), 332);
    assert_eq!(loaded_chunk.get_block(13, 62, 12), 331);
    assert_eq!(loaded_chunk.get_block(13, 64, 12), 0);
}
