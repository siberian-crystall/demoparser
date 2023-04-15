use crate::parsing::class::Class;
use crate::parsing::entities_utils::*;
use crate::parsing::parser_settings::Parser;
use crate::parsing::read_bits::Bitreader;
use crate::parsing::variants::PropData;
use ahash::HashMap;
use bitter::BitReader;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use protobuf::Message;
use smallvec::smallvec;

const NSERIALBITS: u32 = 17;

pub struct Entity {
    pub cls_id: u32,
    pub entity_id: i32,
    pub props: HashMap<String, PropData>,
}

#[derive(Debug, Clone)]
pub struct PlayerMetaData {
    pub player_entity_id: i32,
    pub steamid: u64,
    pub controller_entid: i32,
    pub name: String,
    pub team_num: i32,
}

impl Parser {
    pub fn get_prop_for_ent(&self, prop: &str, entity_id: &i32) -> Option<PropData> {
        if let Some(ent) = self.entities.get(&entity_id) {
            if let Some(prop) = ent.props.get(prop) {
                return Some(prop.clone());
            }
        }
        None
    }
    pub fn parse_packet_ents(&mut self, bytes: &[u8]) -> Result<(), BitReaderError> {
        if !self.parse_entities {
            return Ok(());
        }
        let packet_ents: CSVCMsg_PacketEntities = Message::parse_from_bytes(&bytes).unwrap();
        Ok(self._parse_packet_ents(packet_ents)?)
    }
    fn _parse_packet_ents(
        &mut self,
        packet_ents: CSVCMsg_PacketEntities,
    ) -> Result<(), BitReaderError> {
        let n_updates = packet_ents.updated_entries();
        let data = match packet_ents.entity_data {
            Some(data) => data,
            None => return Err(BitReaderError::MalformedMessage),
        };
        let mut bitreader = Bitreader::new(&data);
        let mut entity_id: i32 = -1;

        for _ in 0..n_updates {
            entity_id += 1 + (bitreader.read_u_bit_var()? as i32);
            // ents.push(entity_id);
            // If the enitity should be deleted
            if bitreader.read_boolie()? {
                self.projectiles.remove(&entity_id);
                self.entities.remove(&entity_id);
                bitreader.read_boolie()?;
                continue;
            }
            let is_new_entity = bitreader.read_boolie()?;
            // Should we create the entity, or refer to an old one
            if is_new_entity {
                let cls_id = bitreader.read_nbits(self.cls_bits.unwrap())?;
                // Both of these are not used. Don't think they are interesting for the parser
                let _serial = bitreader.read_nbits(NSERIALBITS)?;
                let _unknown = bitreader.read_varint();

                let entity = Entity {
                    entity_id: entity_id,
                    cls_id: cls_id,
                    props: HashMap::default(),
                };

                self.entities.insert(entity_id, entity);

                if let Some(baseline_bytes) = self.baselines.get(&cls_id) {
                    let b = &baseline_bytes.clone();
                    let mut br = Bitreader::new(&b);
                    self.decode_entity_update(&mut br, entity_id, true)?;
                };
                if self.cls_by_id[&cls_id].name == "CCSGameRulesProxy" {
                    self.rules_entity_id = Some(entity_id);
                }
                if self.cls_by_id[&cls_id].name.contains("Projectile") {
                    self.projectiles.insert(entity_id);
                }
                self.decode_entity_update(&mut bitreader, entity_id, false)?;
            } else {
                // Entity already exists, don't create it
                self.decode_entity_update(&mut bitreader, entity_id, false)?;
            }
        }
        Ok(())
    }

    pub fn parse_paths(
        &mut self,
        bitreader: &mut Bitreader,
        cls_id: i32,
    ) -> Result<usize, BitReaderError> {
        let mut fp = generate_fp();
        // This is does decoding of huffman tree. If this is too confusing then
        // look up huffman tree decoding. It's very common and bunch of material is vailable.

        // Read bits against a static huffman tree created when parser is initialized.
        // In reality we just store the tree as an array where key is the code interpreted as a usize

        // Example when code exists: 0b10 => huffman_codes[2] => 39
        // Example when code does not exist: 0b11 => huffman_codes[3] => -1

        // Read bits into a usize until we find a match in huffman_codes
        // The symbol is then mapped into a function in do_op()
        // symbol 39 signals that we should stop
        let mut idx = 0;
        let mut val = 0;
        loop {
            val <<= 1;
            val |= bitreader.read_boolie()? as usize;
            let symbol = self.huffman_codes[val];
            if symbol != -1 {
                // Stop reading
                if symbol == 39 {
                    break;
                }
                do_op(symbol, bitreader, &mut fp)?;
                let key = path_to_key(&fp, cls_id);
                match self.pattern_cache.get(&key) {
                    Some(decorder) => self.paths[idx] = PathVariant::Cache(decorder.clone()),
                    None => self.paths[idx] = PathVariant::Normal(fp.clone()),
                };
                idx += 1;
                val = 0;
            }
        }
        Ok(idx)
    }
    pub fn decode_entity_update(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
        is_baseline: bool,
    ) -> Result<(), BitReaderError> {
        let cls_id = match self.entities.get(&entity_id) {
            Some(e) => e.cls_id,
            None => return Err(BitReaderError::EntityNotFound),
        };
        let n_paths = self.parse_paths(bitreader, cls_id as i32)?;
        let entity = match self.entities.get_mut(&(entity_id)) {
            Some(ent) => ent,
            None => return Err(BitReaderError::EntityNotFound),
        };
        let class = match self.cls_by_id.get(&entity.cls_id) {
            Some(cls) => cls,
            None => return Err(BitReaderError::ClassNotFound),
        };
        if class.name == "CCSPlayerControllerq" {
            // hacky solution for now
            /*
            let player_md = Parser::fill_player_data(&paths, bitreader, cls, entity, is_baseline);
            if player_md.player_entity_id != -1 {
                self.players.insert(player_md.player_entity_id, player_md);
            }
            */
        } else {
            for path in &self.paths[..n_paths] {
                if let PathVariant::Normal(p) = path {
                    //serializer_print(&class.serializer, &p.path);
                }

                // probably problem with baseline, this seems to fix
                if is_baseline && bitreader.reader.bits_remaining().unwrap() < 32 {
                    break;
                }
                let decoder = match path {
                    PathVariant::Cache(dec) => dec.clone(),
                    PathVariant::Normal(n) => {
                        let (name, f, decoder) = class.serializer.find_decoder(&n, 0, is_baseline);
                        let key = path_to_key(&n, class.class_id);
                        // self.pattern_cache.insert(key, decoder.clone());
                        decoder
                    }
                };
                let result = bitreader.decode(&decoder);
                /*
                let key = path_to_key(&path, cls.class_id);
                match self.pattern_cache.get(&key) {
                    Some(e) => {
                        let result = bitreader.decode(e);
                        continue;
                    }
                    None => {
                        let (name, f, decoder) = cls.serializer.find_decoder(&path, 0, is_baseline);
                        let result = bitreader.decode(&decoder);
                        self.pattern_cache.insert(key, decoder);
                        continue;
                    }
                }

                let (name, f, decoder) = cls.serializer.find_decoder(&path, 0, is_baseline);
                let result = bitreader.decode(&decoder);

                // println!("{} {} {:?} {:?}", name, cls.name, decoder, path);
                if cls.name == "CCSTeam" && name == "m_iTeamNum" {
                    if let PropData::U32(t) = result {
                        match t {
                            1 => self.teams.team1_entid = Some(entity_id),
                            2 => self.teams.team2_entid = Some(entity_id),
                            3 => self.teams.team3_entid = Some(entity_id),
                            _ => {}
                        }
                    }
                }
                if self.count_props {
                    self.props_counter
                        .entry(name.clone())
                        .and_modify(|counter| *counter += 1)
                        .or_insert(1);
                }

                if (name == "m_vecX" && f.var_name != "CBodyComponent")
                    || (name == "m_vecY" && f.var_name != "CBodyComponent")
                {
                } else {
                    // entity.props.insert(name, result);
                }
                */
            }
        }
        Ok(())
    }

    pub fn fill_player_data(
        paths: &[FieldPath],
        bitreader: &mut Bitreader,
        cls: &Class,
        entity: &mut Entity,
        is_baseline: bool,
    ) -> Result<PlayerMetaData, BitReaderError> {
        let mut player = PlayerMetaData {
            player_entity_id: -1,
            controller_entid: entity.entity_id,
            team_num: -1,
            name: "".to_string(),
            steamid: 0,
        };
        if is_baseline {
            return Ok(player);
        }
        for path in paths {
            let (var_name, _field, decoder) = cls.serializer.find_decoder(&path, 0, is_baseline);
            let result = bitreader.decode(&decoder)?;
            entity.props.insert(var_name.clone(), result.clone());

            match var_name.as_str() {
                "m_iTeamNum" => {}
                "m_iszPlayerName" => {
                    player.name = match result {
                        PropData::String(n) => n,
                        _ => "Broken name!".to_owned(),
                    };
                }
                "m_steamID" => {
                    player.steamid = match result {
                        PropData::U64(xuid) => xuid,
                        _ => 99999999,
                    };
                }
                "m_hPlayerPawn" => {
                    player.player_entity_id = match result {
                        PropData::U32(handle) => {
                            // create helper value
                            entity.props.insert(
                                "player_entid".to_string(),
                                PropData::I32((handle & 0x7FF) as i32),
                            );
                            (handle & 0x7FF) as i32
                        }
                        _ => -1,
                    };
                }
                _ => {}
            }
        }
        Ok(player)
    }
}
use crate::parsing::sendtables::Decoder;

use super::read_bits::BitReaderError;
use super::sendtables::serializer_print;
#[derive(Clone, Debug)]
pub enum PathVariant {
    Normal(FieldPath),
    Cache(Decoder),
}

fn generate_fp() -> FieldPath {
    FieldPath {
        done: false,
        path: [-1, 0, 0, 0, 0, 0, 0],
        last: 0,
        decoder: None,
    }
}

#[inline(always)]
pub fn path_to_key(field_path: &FieldPath, cls_id: i32) -> u64 {
    let mut key: u64 = 0;
    for idx in 0..field_path.last + 1 {
        key |= field_path.path[idx] as u64;
        key <<= 14;
    }
    key | cls_id as u64
}
