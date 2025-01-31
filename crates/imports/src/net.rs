type Rid = i32;
use super::html::Node;
use super::std::{Rid as ValueRid, ValueRef, StringRef};

use super::alloc::vec::Vec;
use super::alloc::string::String;

#[repr(C)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

#[link(wasm_import_module = "net")]
extern "C" {
    #[link_name = "init"]
    fn request_init(method: HttpMethod) -> Rid;
    #[link_name = "send"]
    fn request_send(rd: Rid);
    #[link_name = "close"]
    fn request_close(rd: Rid);

    #[link_name = "set_url"]
    fn request_set_url(rd: Rid, value: *const u8, len: usize);
    #[link_name = "set_header"]
    fn request_set_header(rd: Rid, key: *const u8, key_len: usize, val: *const u8, val_len: usize);
    #[link_name = "set_body"]
    fn request_set_body(rd: Rid, value: *const u8, len: usize);

    #[link_name = "get_url"]
    fn request_get_url(rd: Rid) -> Rid;
    #[link_name = "get_data"]
    fn request_get_data(rd: Rid, buffer: *mut u8, size: usize);
    #[link_name = "get_data_size"]
    fn request_get_data_size(rd: Rid) -> usize;

    #[link_name = "json"]
    fn request_json(rd: Rid) -> ValueRid;
    #[link_name = "html"]
    fn request_html(rd: Rid) -> ValueRid;
}

/// A type that makes a HTTP request.
pub struct Request(Rid);
impl Request {
    /// Start a new request with a URL and HTTP method
    ///
    /// Usage:
    /// ```rs
    /// Request::new("https://example.com", HttpMethod::Get)
    /// ```
    pub fn new(url: &str, method: HttpMethod) -> Self {
        unsafe {
            let rd = request_init(method);
            request_set_url(rd, url.as_ptr(), url.len());
            Self(rd)
        }
    }

    /// Set a header
    pub fn header(self, key: &str, val: &str) -> Self {
        unsafe {
            request_set_header(self.0, key.as_ptr(), key.len(), val.as_ptr(), val.len());
        };
        self
    }

    /// Set the body's data
    pub fn body(self, data: &[u8]) -> Self {
        unsafe { request_set_body(self.0, data.as_ptr(), data.len()) };
        self
    }

    fn send(&self) {
        unsafe { request_send(self.0) }
    }

    fn close(&self) {
        unsafe { request_close(self.0) }
    }

    /// Get the URL of the request
    pub fn url(&self) -> StringRef {
        let rid = unsafe { request_get_url(self.0) };
        StringRef(ValueRef::new(rid))
    }

    /// Get the raw data from the response
    pub fn data<'a>(self) -> Vec<u8> {
        self.send();
        let size = unsafe { request_get_data_size(self.0) };
        let mut buffer: Vec<u8> = Vec::with_capacity(size);
        unsafe {
            request_get_data(self.0, buffer.as_mut_ptr(), size);
            buffer.set_len(size);
        }
        self.close();
        buffer
    }

    pub fn string<'a>(self) -> String {
        String::from_utf8(self.data()).unwrap_or(String::new())
    }

    /// Get the data as JSON
    pub fn json(self) -> ValueRef {
        self.send();
        let rid = unsafe { request_json(self.0) };
        self.close();
        ValueRef::new(rid)
    }

    /// Get the data as a HTML scraper
    pub fn html(self) -> Node {
        self.send();
        let rid = unsafe { request_html(self.0) };
        self.close();
        Node::from(rid)
    }
}
