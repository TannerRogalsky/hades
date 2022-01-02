#[cfg(feature = "web")]
pub mod web;

const FILE_SIGNATURE: &[u8; 4] = b"\x53\x47\x42\x31";
#[allow(unused)]
const SAVE_DATA_V14_LENGTH: usize = 3145720;
#[allow(unused)]
const SAVE_DATA_V15_LENGTH: usize = 3145720;
#[allow(unused)]
const SAVE_DATA_V16_LENGTH: usize = 3145728;
#[allow(unused)]
const SAV15_UNCOMPRESSED_SIZE: usize = 9388032;
const SAV16_UNCOMPRESSED_SIZE: usize = 9388032;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VersionId {
    pub checksum: u32,
    pub version: u32,
}

pub fn load_version_id(data: &[u8]) -> nom::IResult<&[u8], VersionId> {
    let (data, _sig) = nom::bytes::complete::tag(FILE_SIGNATURE)(data)?;
    let (data, checksum) = nom::number::complete::le_u32(data)?;
    let (data, version) = nom::number::complete::le_u32(data)?;
    Ok((data, VersionId { checksum, version }))
}

#[derive(Debug, Default)]
pub struct Version16<'a> {
    time: u64,
    location: std::borrow::Cow<'a, str>,
    runs: u32,
    active_meta_points: u32,
    active_shrine_points: u32,
    god_mode_enabled: bool,
    hell_mode_enabled: bool,
    lua_keys: Vec<std::borrow::Cow<'a, str>>,
    current_map_name: std::borrow::Cow<'a, str>,
    start_next_map: std::borrow::Cow<'a, str>,
    lua_state_compressed: std::borrow::Cow<'a, [u8]>,
}

impl<'a> Version16<'a> {
    pub fn load(data: &'a [u8]) -> nom::IResult<&'a [u8], Self> {
        let (data, time) = nom::number::complete::le_u64(data)?;
        let (data, location) = pascal_str(data)?;
        let (data, runs) = nom::number::complete::le_u32(data)?;
        let (data, active_meta_points) = nom::number::complete::le_u32(data)?;
        let (data, active_shrine_points) = nom::number::complete::le_u32(data)?;
        let (data, god_mode_enabled) = nom::number::complete::u8(data)?;
        let (data, hell_mode_enabled) = nom::number::complete::u8(data)?;
        let (data, lua_keys) = nom::multi::length_count(
            nom::number::complete::le_u32,
            nom::combinator::map(pascal_str, std::borrow::Cow::Borrowed),
        )(data)?;
        let (data, current_map_name) = pascal_str(data)?;
        let (data, start_next_map) = pascal_str(data)?;
        let (data, lua_state_compressed) =
            nom::multi::length_value(nom::number::complete::le_u32, nom::combinator::rest)(data)?;
        assert_eq!(data.len(), 0);

        Ok((
            data,
            Self {
                time,
                location: location.into(),
                runs,
                active_meta_points,
                active_shrine_points,
                god_mode_enabled: god_mode_enabled != 0,
                hell_mode_enabled: hell_mode_enabled != 0,
                lua_keys: lua_keys.into(),
                current_map_name: current_map_name.into(),
                start_next_map: start_next_map.into(),
                lua_state_compressed: std::borrow::Cow::Borrowed(lua_state_compressed),
            },
        ))
    }

    pub fn to_owned(&self) -> Version16<'static> {
        Version16 {
            time: self.time,
            location: self.location.to_string().into(),
            runs: self.runs,
            active_meta_points: self.active_meta_points,
            active_shrine_points: self.active_shrine_points,
            god_mode_enabled: self.god_mode_enabled,
            hell_mode_enabled: self.hell_mode_enabled,
            lua_keys: self
                .lua_keys
                .iter()
                .map(|key| key.to_string().into())
                .collect::<Vec<_>>(),
            current_map_name: self.current_map_name.to_string().into(),
            start_next_map: self.start_next_map.to_string().into(),
            lua_state_compressed: self.lua_state_compressed.to_vec().into(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.lua_state_compressed.len());
        result.extend_from_slice(FILE_SIGNATURE);
        let checksum_pos = result.len();
        result.extend_from_slice(&0u32.to_le_bytes());
        result.extend_from_slice(&16u32.to_le_bytes());

        result.extend_from_slice(&self.time.to_le_bytes());
        write_pascal_str(&mut result, &self.location);
        result.extend_from_slice(&self.runs.to_le_bytes());
        result.extend_from_slice(&self.active_meta_points.to_le_bytes());
        result.extend_from_slice(&self.active_shrine_points.to_le_bytes());
        result.push(self.god_mode_enabled as u8);
        result.push(self.hell_mode_enabled as u8);
        result.extend_from_slice(&(self.lua_keys.len() as u32).to_le_bytes());
        for lua_key in self.lua_keys.iter() {
            write_pascal_str(&mut result, lua_key);
        }
        write_pascal_str(&mut result, &self.current_map_name);
        write_pascal_str(&mut result, &self.start_next_map);
        result.extend_from_slice(&(self.lua_state_compressed.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.lua_state_compressed);

        let checksum = adler32::RollingAdler32::from_buffer(&result[8..]).hash();
        result[checksum_pos..(checksum_pos + 4)].copy_from_slice(&checksum.to_le_bytes());

        result
    }

    pub fn decompress_lua_state(&self) -> eyre::Result<LuaState> {
        let decompressed =
            lz4_flex::block::decompress(&self.lua_state_compressed, SAV16_UNCOMPRESSED_SIZE)?;
        let (rest, mut state) =
            luabins::load(&decompressed).map_err(|err| eyre::eyre!("{}", err))?;
        assert!(rest.is_empty());
        assert_eq!(state.len(), 1);
        match state.remove(0) {
            luabins::Value::Table(table) => Ok(LuaState(table)),
            _ => Err(eyre::eyre!("Wrong type.")),
        }
    }

    pub fn set_lua_state(&mut self, state: LuaState) {
        let state = vec![luabins::Value::Table(state.0)];
        let bytes = luabins::save(&state);
        let compressed = lz4_flex::block::compress(&bytes);
        self.lua_state_compressed = std::borrow::Cow::Owned(compressed);
    }
}

fn write_pascal_str(result: &mut Vec<u8>, string: &str) {
    result.extend_from_slice(&(string.len() as u32).to_le_bytes());
    result.extend_from_slice(string.as_bytes());
}

fn pascal_str(data: &[u8]) -> nom::IResult<&[u8], &str> {
    nom::combinator::map_res(
        nom::multi::length_data(nom::number::complete::le_u32),
        std::str::from_utf8,
    )(data)
}

type KV = (luabins::Key, luabins::Value);
pub struct LuaState(Vec<KV>);

impl LuaState {
    const DARKNESS: [&'static str; 3] = ["GameState", "Resources", "MetaPoints"];

    pub fn darkness(&self) -> u32 {
        self.deep_find(&Self::DARKNESS)
            .and_then(luabins::Value::get_number)
            .unwrap_or(0.) as _
    }

    pub fn set_darkness(&mut self, darkness: u32) {
        self.deep_find_mut(&Self::DARKNESS).map(|value| {
            if let luabins::Value::Number(inner) = value {
                *inner = darkness as _;
            }
        });
    }

    pub fn to_json(&self) -> serde_json::Value {
        fn is_array(kv: &[KV]) -> bool {
            for index in 1..=kv.len() {
                let is_index = kv.iter().find(|(key, _value)| {
                    key.get_number()
                        .filter(|n| n.into_inner() as usize == index)
                        .is_some()
                });
                if !is_index.is_some() {
                    return false;
                }
            }
            true
        }

        fn json_from_value(value: &luabins::Value) -> serde_json::Value {
            use luabins::Value;
            match value {
                Value::Nil => serde_json::Value::Null,
                Value::Boolean(inner) => serde_json::Value::Bool(*inner),
                Value::Number(inner) => {
                    serde_json::Value::Number(serde_json::Number::from_f64(*inner).unwrap())
                }
                Value::String(inner) => serde_json::Value::String(inner.clone()),
                Value::Table(inner) => {
                    if is_array(inner) {
                        to_array(inner)
                    } else {
                        to_object(inner)
                    }
                }
            }
        }

        fn to_array(kv: &[KV]) -> serde_json::Value {
            let list = kv
                .iter()
                .map(|(_k, v)| json_from_value(v))
                .collect::<Vec<serde_json::Value>>();
            serde_json::Value::Array(list)
        }

        fn to_object(kv: &[KV]) -> serde_json::Value {
            use luabins::Key;
            let map = kv
                .iter()
                .map(|(k, v)| {
                    let k = match k {
                        Key::String(inner) => inner.clone(),
                        Key::Number(inner) => inner.to_string(),
                        _ => unimplemented!(),
                    };
                    let v = json_from_value(v);
                    (k, v)
                })
                .collect::<serde_json::Map<String, serde_json::Value>>();
            serde_json::Value::Object(map)
        }

        to_object(&self.0)
    }

    pub fn from_json(json: serde_json::Map<String, serde_json::Value>) -> LuaState {
        fn value_from_json(value: serde_json::Value) -> luabins::Value {
            use serde_json::Value;
            match value {
                Value::Null => luabins::Value::Nil,
                Value::Bool(inner) => luabins::Value::Boolean(inner),
                Value::Number(inner) => luabins::Value::Number(inner.as_f64().unwrap()),
                Value::String(inner) => luabins::Value::String(inner),
                Value::Array(inner) => luabins::Value::Table(
                    inner
                        .into_iter()
                        .enumerate()
                        .map(|(index, value)| {
                            let key = luabins::Key::Number(
                                std::convert::TryInto::try_into(index as f64).unwrap(),
                            );
                            let value = value_from_json(value);
                            (key, value)
                        })
                        .collect(),
                ),
                Value::Object(inner) => luabins::Value::Table(
                    inner
                        .into_iter()
                        .map(|(key, value)| {
                            let key = luabins::Key::String(key);
                            let value = value_from_json(value);
                            (key, value)
                        })
                        .collect(),
                ),
            }
        }

        let state = json
            .into_iter()
            .map(|(key, value)| (luabins::Key::String(key), value_from_json(value)))
            .collect();
        LuaState(state)
    }

    fn deep_find<'a, T: AsRef<[&'a str]>>(&self, targets: T) -> Option<&luabins::Value> {
        fn find(target: &str) -> impl FnMut(&KV) -> Option<&luabins::Value> + '_ {
            move |(k, v)| match k {
                luabins::Key::String(k) => (k == target).then(|| v),
                _ => None,
            }
        }

        targets.as_ref().split_last().and_then(|(last, targets)| {
            let mut current = self.0.as_slice();
            for target in targets {
                match current
                    .iter()
                    .find_map(find(target))
                    .and_then(luabins::Value::get_table)
                {
                    None => return None,
                    Some(next) => current = next,
                }
            }
            current.iter().find_map(find(last))
        })
    }

    fn deep_find_mut<'a, T: AsRef<[&'a str]>>(
        &mut self,
        targets: T,
    ) -> Option<&mut luabins::Value> {
        fn find(target: &str) -> impl FnMut(&mut KV) -> Option<&mut luabins::Value> + '_ {
            move |(k, v)| match k {
                luabins::Key::String(k) => (k == target).then(|| v),
                _ => None,
            }
        }

        targets.as_ref().split_last().and_then(|(last, targets)| {
            let mut current = self.0.as_mut_slice();
            for target in targets {
                match current
                    .iter_mut()
                    .find_map(find(target))
                    .and_then(|value| match value {
                        luabins::Value::Table(table) => Some(table.as_mut_slice()),
                        _ => None,
                    }) {
                    None => return None,
                    Some(next) => current = next,
                }
            }
            current.iter_mut().find_map(find(last))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luabins::Value;

    #[test]
    fn it_works() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("Profile2.sav");
        let data = std::fs::read(path).unwrap();
        let checksum = adler32::RollingAdler32::from_buffer(&data[8..]).hash();
        let (rest, result) = load_version_id(&data).unwrap();
        assert_eq!(result.checksum, checksum);
        assert_eq!(result.version, 16);
        match Version16::load(rest) {
            Ok((_data, result)) => {
                let state = result.decompress_lua_state().unwrap();
                assert_eq!(state.darkness(), 144);
                let bin = result.to_bytes();
                assert_eq!(data, bin);

                fn to_json(kv: &[KV]) -> serde_json::Value {
                    use luabins::Key;
                    let map = kv
                        .iter()
                        .map(|(k, v)| {
                            let k = match k {
                                Key::String(inner) => inner.clone(),
                                Key::Number(inner) => inner.to_string(),
                                _ => unimplemented!(),
                            };
                            let v = match v {
                                Value::Nil => serde_json::Value::Null,
                                Value::Boolean(inner) => serde_json::Value::Bool(*inner),
                                Value::Number(inner) => serde_json::Value::Number(
                                    serde_json::Number::from_f64(*inner).unwrap(),
                                ),
                                Value::String(inner) => serde_json::Value::String(inner.clone()),
                                Value::Table(inner) => to_json(inner),
                            };
                            (k, v)
                        })
                        .collect::<serde_json::Map<String, serde_json::Value>>();
                    serde_json::Value::Object(map)
                    // for (key, value) in kv {
                    //     match key {
                    //         Key::String(_) | Key::Number(_) => {
                    //             if let Some(table) = value.get_table() {
                    //                 to_json(table);
                    //             }
                    //         }
                    //         _ => panic!(),
                    //     }
                    // }
                }
                let json = to_json(&state.0);
                let json = serde_json::to_string_pretty(&json).unwrap();
                std::fs::write(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("out.json"),
                    json,
                )
                .unwrap();
            }
            Err(err) => {
                use nom::Err;
                match err {
                    Err::Incomplete(err) => {
                        eprintln!("incomplete {:?}", err);
                    }
                    Err::Error(err) => {
                        eprintln!("err {:?}", err.code);
                    }
                    Err::Failure(err) => {
                        eprintln!("failure {:?}", err.code);
                    }
                }
            }
        }
    }
}
