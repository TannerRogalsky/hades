use crate::LuaState;
use wasm_bindgen::prelude::*;

fn to_js<E: std::error::Error>(err: E) -> JsValue {
    JsValue::from_str(&format!("{}", err))
}

fn eyre_to_js(err: eyre::Report) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[wasm_bindgen]
pub struct HadesSave {
    inner: super::Version16<'static>,
}

#[wasm_bindgen]
impl HadesSave {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<HadesSave, JsValue> {
        fn parse_inner(data: &[u8]) -> nom::IResult<&[u8], super::Version16> {
            let (rest, version) = super::load_version_id(&data)?;
            assert_eq!(version.version, 16);

            let (rest, inner) = super::Version16::load(&rest)?;
            assert!(rest.is_empty());

            Ok((rest, inner))
        }

        let (_rest, inner) = parse_inner(&data).map_err(to_js)?;
        Ok(Self {
            inner: inner.to_owned(),
        })
    }

    pub fn read_json(&self) -> Result<String, JsValue> {
        let inner = self.inner.decompress_lua_state().map_err(eyre_to_js)?;
        let inner = inner.to_json();
        serde_json::to_string(&inner).map_err(to_js)
    }

    pub fn write_json(&mut self, json: &str) -> Result<(), JsValue> {
        let json = serde_json::from_str(json).map_err(to_js)?;
        let state = LuaState::from_json(json);
        self.inner.set_lua_state(state);
        Ok(())
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        self.inner.to_bytes().into_boxed_slice()
    }
}
