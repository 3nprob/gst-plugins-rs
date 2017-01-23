//  Copyright (C) 2016 Sebastian Dröge <sebastian@centricular.com>
//
//  This library is free software; you can redistribute it and/or
//  modify it under the terms of the GNU Library General Public
//  License as published by the Free Software Foundation; either
//  version 2 of the License, or (at your option) any later version.
//
//  This library is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  Library General Public License for more details.
//
//  You should have received a copy of the GNU Library General Public
//  License along with this library; if not, write to the
//  Free Software Foundation, Inc., 51 Franklin St, Fifth Floor,
//  Boston, MA 02110-1301, USA.

use std::os::raw::c_void;
use std::fmt;
use libc::c_char;
use std::ffi::{CStr, CString};
use utils::*;
use value::*;

pub trait Tag {
    type TagType: ValueType;
    fn tag_name() -> &'static str;
}

macro_rules! impl_tag(
    ($name:ident, $t:ty, $tag:expr) => {
        pub struct $name;
        impl Tag for $name {
            type TagType = $t;
            fn tag_name() -> &'static str {
                $tag
            }
        }
    };
);

impl_tag!(Title, String, "title");
impl_tag!(Album, String, "album");
impl_tag!(Artist, String, "artist");
impl_tag!(Encoder, String, "encoder");
impl_tag!(AudioCodec, String, "audio-codec");
impl_tag!(VideoCodec, String, "video-codec");
impl_tag!(SubtitleCodec, String, "subtitle-codec");
impl_tag!(ContainerFormat, String, "container-format");
// TODO: Should ideally enforce this to be ISO-639
impl_tag!(LanguageCode, String, "language-code");
impl_tag!(Duration, u64, "duration");
impl_tag!(NominalBitrate, u32, "nominal-bitrate");

#[repr(C)]
pub enum MergeMode {
    ReplaceAll = 1,
    Replace,
    Append,
    Prepend,
    Keep,
    KeepAll,
}

#[derive(Eq)]
pub struct TagList(*mut c_void);

impl TagList {
    pub fn new() -> Self {
        extern "C" {
            fn gst_tag_list_new_empty() -> *mut c_void;
        }

        TagList(unsafe { gst_tag_list_new_empty() })
    }

    pub unsafe fn new_from_ptr(ptr: *mut c_void) -> TagList {
        extern "C" {
            fn gst_mini_object_ref(mini_object: *mut c_void) -> *mut c_void;
        }

        TagList(gst_mini_object_ref(ptr))
    }

    pub fn add<T: Tag>(&mut self, value: T::TagType, mode: MergeMode)
        where Value: From<<T as Tag>::TagType>
    {
        extern "C" {
            fn gst_tag_list_add_value(list: *mut c_void,
                                      mode: u32,
                                      tag: *const c_char,
                                      value: *const GValue);
        }

        let v = Value::from(value);
        let gvalue = v.to_gvalue();
        let tag_name = CString::new(T::tag_name()).unwrap();

        unsafe {
            gst_tag_list_add_value(self.0,
                                   mode as u32,
                                   tag_name.as_ptr(),
                                   &gvalue as *const GValue);
        }
    }

    pub fn get<T: Tag>(&mut self) -> Option<TypedValue<T::TagType>>
        where Value: From<<T as Tag>::TagType>
    {
        extern "C" {
            fn gst_tag_list_copy_value(value: *mut GValue,
                                       list: *mut c_void,
                                       tag: *const c_char)
                                       -> GBoolean;
        }

        let mut gvalue = GValue::new();
        let tag_name = CString::new(T::tag_name()).unwrap();

        let found = unsafe {
            gst_tag_list_copy_value(&mut gvalue as *mut GValue, self.0, tag_name.as_ptr())
        };

        if !found.to_bool() {
            return None;
        }

        match Value::from_gvalue(&gvalue) {
            Some(value) => Some(TypedValue::new(value)),
            None => None,
        }
    }

    pub fn to_string(&self) -> String {
        extern "C" {
            fn gst_tag_list_to_string(tag_list: *mut c_void) -> *mut c_char;
            fn g_free(ptr: *mut c_char);
        }

        unsafe {
            let ptr = gst_tag_list_to_string(self.0);
            let s = CStr::from_ptr(ptr).to_str().unwrap().into();
            g_free(ptr);

            s
        }
    }

    pub unsafe fn as_ptr(&self) -> *const c_void {
        self.0
    }

    pub fn make_writable(self: TagList) -> TagList {
        extern "C" {
            fn gst_mini_object_make_writable(obj: *mut c_void) -> *mut c_void;
        }

        let raw = unsafe { gst_mini_object_make_writable(self.0) };

        TagList(raw)
    }

    pub fn copy(&self) -> TagList {
        extern "C" {
            fn gst_mini_object_copy(obj: *const c_void) -> *mut c_void;
        }
        unsafe { TagList(gst_mini_object_copy(self.0)) }
    }
}

impl Clone for TagList {
    fn clone(&self) -> Self {
        extern "C" {
            fn gst_mini_object_ref(mini_object: *mut c_void) -> *mut c_void;
        }

        unsafe { TagList(gst_mini_object_ref(self.0)) }
    }
}

impl Drop for TagList {
    fn drop(&mut self) {
        extern "C" {
            fn gst_mini_object_unref(mini_object: *mut c_void);
        }

        unsafe { gst_mini_object_unref(self.0) }
    }
}

impl fmt::Debug for TagList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl PartialEq for TagList {
    fn eq(&self, other: &TagList) -> bool {
        extern "C" {
            fn gst_tag_list_is_equal(a: *const c_void, b: *const c_void) -> GBoolean;
        }

        unsafe { gst_tag_list_is_equal(self.0, other.0).to_bool() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use value::*;
    use std::ptr;
    use std::os::raw::c_void;

    fn init() {
        extern "C" {
            fn gst_init(argc: *mut c_void, argv: *mut c_void);
        }

        unsafe {
            gst_init(ptr::null_mut(), ptr::null_mut());
        }
    }

    #[test]
    fn test_add() {
        init();

        let mut tags = TagList::new();
        assert_eq!(tags.to_string(), "taglist;");
        tags.add::<Title>("some title".into(), MergeMode::Append);
        tags.add::<Duration>((1000u64 * 1000 * 1000 * 120).into(), MergeMode::Append);
        assert_eq!(tags.to_string(),
                   "taglist, title=(string)\"some\\ title\", duration=(guint64)120000000000;");
    }

    #[test]
    fn test_get() {
        init();

        let mut tags = TagList::new();
        assert_eq!(tags.to_string(), "taglist;");
        tags.add::<Title>("some title".into(), MergeMode::Append);
        tags.add::<Duration>((1000u64 * 1000 * 1000 * 120).into(), MergeMode::Append);

        assert_eq!(*tags.get::<Title>().unwrap(), "some title");
        assert_eq!(*tags.get::<Duration>().unwrap(), (1000u64 * 1000 * 1000 * 120));
    }
}
