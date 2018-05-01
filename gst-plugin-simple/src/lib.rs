// Copyright (C) 2017 Sebastian Dröge <sebastian@centricular.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate glib;
extern crate gobject_subclass;
extern crate gst_plugin;
#[macro_use]
extern crate gstreamer as gst;
extern crate gstreamer_base as gst_base;

extern crate url;

pub mod demuxer;
pub mod error;
pub mod sink;
pub mod source;

pub type UriValidator = Fn(&url::Url) -> Result<(), error::UriError> + Send + Sync + 'static;
