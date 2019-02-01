use std::cell::RefCell;
use std::rc::Rc;
use std::io::ErrorKind;
use std::str;
use web_sys::{
    XmlHttpRequest,
    XmlHttpRequestResponseType
};
use wasm_bindgen::prelude::*;
use web_sys::{Blob,File};
use js_sys::ArrayBuffer;
pub type IoError = std::io::Error;
pub struct FileSystem {}

enum BufferState {
    Empty,
    Buffer(Vec<u8>),
    Error(String),
}

pub struct File {
    blob: Blob,
}

impl FileSystem {
    pub fn open(s: &str) -> Result<File>{
        let buffer:[u8]  = ArrayBuffer::new(50);
        let f = File{
            blob:File::new_with_u8_array_sequence(ArrayBuffer,s)
        };
        ok(f)
    }
} 
impl FileSystem {
    pub fn open(s: &str) -> Result<File, IoError> {

        let buffer_state = Rc::new(RefCell::new(BufferState::Empty));

        let on_get_buffer = {
            let buffer_state = buffer_state.clone();
            move |ab: Vec<u8>| {
                let data = ab.to_vec();
                if data.len() > 0 {
                    *buffer_state.borrow_mut() = BufferState::Buffer(data);
                }
            }
        };

        let on_error = {
            let buffer_state = buffer_state.clone();
            move |s: String| {
                let msg = format!("Fail to read file from web {}", s);
                *buffer_state.borrow_mut() = BufferState::Error(msg);
            }
        };
        let oReq = XmlHttpRequest::new();
        oReq.open("GET",s,true).unwrap();
        oReq.set_response_type(XmlHttpRequestResponseType::Arraybuffer);
        let on_error_js = move|s|{
                var on_error = on_error.clone();
                on_error(s);
                on_error.drop();
            };
        let js_fn = Closure::wrap(Box::new(move |s| {
                on_error(s);
                on_error.drop();
            }) as Box<dyn Fn()>);
        oReq.set_onload(Some(js_fn));
        let js_fn_2 = Closure::wrap(Box::new(move |oEvent|{
            let status = oReq.status().unwrap();
            let arrayBuffer = oReq.response().unwrap();
            if (status == 200 && arrayBuffer){
                on_get_buffer(arrayBuffer);
                on_get_buffer.drop()
            } else{
                on_error_js("Fail to get array buffer from network..");
            }
        }) as Box<dyn Fn()>);
        oReq.set_onerror(Some(js_fn_2));
        let js_fn_3 = Closure::wrap(Box::new(move |oEvent|{
                on_error_js("Fail to read from network..");
            }) as Box<dyn Fn()>); 
        oReq.onerror(Some(js_fn_3))?;
        oReq.send()?;

        Ok(File {
            buffer_state: buffer_state,
        })
    }
}

impl File {
    pub fn is_ready(&self) -> bool {
        let bs = self.buffer_state.borrow();
        match *bs {
            BufferState::Empty => false,
            BufferState::Error(_) => true,
            BufferState::Buffer(_) => true,
        }
    }

    pub fn read_binary(&mut self) -> Result<Vec<u8>, IoError> {
        let mut bs = self.buffer_state.borrow_mut();
        match *bs {
            BufferState::Error(ref s) => Err(std::io::Error::new(ErrorKind::Other, s.clone())),
            BufferState::Buffer(ref mut v) => Ok({
                let mut r = Vec::new();
                r.append(v);
                r
            }),
            _ => unreachable!(),
        }
    }

    pub fn read_text(&mut self) -> Result<String, IoError> {
        let mut bs = self.buffer_state.borrow_mut();
        match *bs {
            BufferState::Error(ref s) => Err(std::io::Error::new(ErrorKind::Other, s.clone())),
            BufferState::Buffer(ref mut v) => match str::from_utf8(v) {
                Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
                Ok(v) => Ok(v.to_owned()),
            },
            _ => unreachable!(),
        }
    }
}
