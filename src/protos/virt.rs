// This file is generated by rust-protobuf 2.25.2. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `virt.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_25_2;

#[derive(PartialEq,Clone,Default)]
pub struct VirtualScenarioItem {
    // message fields
    pub event_type: ::std::string::String,
    pub port: i32,
    pub id: ::std::string::String,
    pub state: bool,
    pub buffer: ::std::vec::Vec<u8>,
    pub timeout: i32,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a VirtualScenarioItem {
    fn default() -> &'a VirtualScenarioItem {
        <VirtualScenarioItem as ::protobuf::Message>::default_instance()
    }
}

impl VirtualScenarioItem {
    pub fn new() -> VirtualScenarioItem {
        ::std::default::Default::default()
    }

    // string event_type = 1;


    pub fn get_event_type(&self) -> &str {
        &self.event_type
    }
    pub fn clear_event_type(&mut self) {
        self.event_type.clear();
    }

    // Param is passed by value, moved
    pub fn set_event_type(&mut self, v: ::std::string::String) {
        self.event_type = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_event_type(&mut self) -> &mut ::std::string::String {
        &mut self.event_type
    }

    // Take field
    pub fn take_event_type(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.event_type, ::std::string::String::new())
    }

    // int32 port = 2;


    pub fn get_port(&self) -> i32 {
        self.port
    }
    pub fn clear_port(&mut self) {
        self.port = 0;
    }

    // Param is passed by value, moved
    pub fn set_port(&mut self, v: i32) {
        self.port = v;
    }

    // string id = 3;


    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn clear_id(&mut self) {
        self.id.clear();
    }

    // Param is passed by value, moved
    pub fn set_id(&mut self, v: ::std::string::String) {
        self.id = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_id(&mut self) -> &mut ::std::string::String {
        &mut self.id
    }

    // Take field
    pub fn take_id(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.id, ::std::string::String::new())
    }

    // bool state = 4;


    pub fn get_state(&self) -> bool {
        self.state
    }
    pub fn clear_state(&mut self) {
        self.state = false;
    }

    // Param is passed by value, moved
    pub fn set_state(&mut self, v: bool) {
        self.state = v;
    }

    // bytes buffer = 5;


    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    // Param is passed by value, moved
    pub fn set_buffer(&mut self, v: ::std::vec::Vec<u8>) {
        self.buffer = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_buffer(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.buffer
    }

    // Take field
    pub fn take_buffer(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.buffer, ::std::vec::Vec::new())
    }

    // int32 timeout = 6;


    pub fn get_timeout(&self) -> i32 {
        self.timeout
    }
    pub fn clear_timeout(&mut self) {
        self.timeout = 0;
    }

    // Param is passed by value, moved
    pub fn set_timeout(&mut self, v: i32) {
        self.timeout = v;
    }
}

impl ::protobuf::Message for VirtualScenarioItem {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.event_type)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.port = tmp;
                },
                3 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.id)?;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.state = tmp;
                },
                5 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.buffer)?;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.timeout = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.event_type.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.event_type);
        }
        if self.port != 0 {
            my_size += ::protobuf::rt::value_size(2, self.port, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.id.is_empty() {
            my_size += ::protobuf::rt::string_size(3, &self.id);
        }
        if self.state != false {
            my_size += 2;
        }
        if !self.buffer.is_empty() {
            my_size += ::protobuf::rt::bytes_size(5, &self.buffer);
        }
        if self.timeout != 0 {
            my_size += ::protobuf::rt::value_size(6, self.timeout, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.event_type.is_empty() {
            os.write_string(1, &self.event_type)?;
        }
        if self.port != 0 {
            os.write_int32(2, self.port)?;
        }
        if !self.id.is_empty() {
            os.write_string(3, &self.id)?;
        }
        if self.state != false {
            os.write_bool(4, self.state)?;
        }
        if !self.buffer.is_empty() {
            os.write_bytes(5, &self.buffer)?;
        }
        if self.timeout != 0 {
            os.write_int32(6, self.timeout)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> VirtualScenarioItem {
        VirtualScenarioItem::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "event_type",
                |m: &VirtualScenarioItem| { &m.event_type },
                |m: &mut VirtualScenarioItem| { &mut m.event_type },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "port",
                |m: &VirtualScenarioItem| { &m.port },
                |m: &mut VirtualScenarioItem| { &mut m.port },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "id",
                |m: &VirtualScenarioItem| { &m.id },
                |m: &mut VirtualScenarioItem| { &mut m.id },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                "state",
                |m: &VirtualScenarioItem| { &m.state },
                |m: &mut VirtualScenarioItem| { &mut m.state },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "buffer",
                |m: &VirtualScenarioItem| { &m.buffer },
                |m: &mut VirtualScenarioItem| { &mut m.buffer },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "timeout",
                |m: &VirtualScenarioItem| { &m.timeout },
                |m: &mut VirtualScenarioItem| { &mut m.timeout },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<VirtualScenarioItem>(
                "VirtualScenarioItem",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static VirtualScenarioItem {
        static instance: ::protobuf::rt::LazyV2<VirtualScenarioItem> = ::protobuf::rt::LazyV2::INIT;
        instance.get(VirtualScenarioItem::new)
    }
}

impl ::protobuf::Clear for VirtualScenarioItem {
    fn clear(&mut self) {
        self.event_type.clear();
        self.port = 0;
        self.id.clear();
        self.state = false;
        self.buffer.clear();
        self.timeout = 0;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for VirtualScenarioItem {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for VirtualScenarioItem {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct VirtualScenarioItems {
    // message fields
    pub items: ::protobuf::RepeatedField<VirtualScenarioItem>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a VirtualScenarioItems {
    fn default() -> &'a VirtualScenarioItems {
        <VirtualScenarioItems as ::protobuf::Message>::default_instance()
    }
}

impl VirtualScenarioItems {
    pub fn new() -> VirtualScenarioItems {
        ::std::default::Default::default()
    }

    // repeated .VirtualScenarioItem items = 1;


    pub fn get_items(&self) -> &[VirtualScenarioItem] {
        &self.items
    }
    pub fn clear_items(&mut self) {
        self.items.clear();
    }

    // Param is passed by value, moved
    pub fn set_items(&mut self, v: ::protobuf::RepeatedField<VirtualScenarioItem>) {
        self.items = v;
    }

    // Mutable pointer to the field.
    pub fn mut_items(&mut self) -> &mut ::protobuf::RepeatedField<VirtualScenarioItem> {
        &mut self.items
    }

    // Take field
    pub fn take_items(&mut self) -> ::protobuf::RepeatedField<VirtualScenarioItem> {
        ::std::mem::replace(&mut self.items, ::protobuf::RepeatedField::new())
    }
}

impl ::protobuf::Message for VirtualScenarioItems {
    fn is_initialized(&self) -> bool {
        for v in &self.items {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.items)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.items {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        for v in &self.items {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> VirtualScenarioItems {
        VirtualScenarioItems::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<VirtualScenarioItem>>(
                "items",
                |m: &VirtualScenarioItems| { &m.items },
                |m: &mut VirtualScenarioItems| { &mut m.items },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<VirtualScenarioItems>(
                "VirtualScenarioItems",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static VirtualScenarioItems {
        static instance: ::protobuf::rt::LazyV2<VirtualScenarioItems> = ::protobuf::rt::LazyV2::INIT;
        instance.get(VirtualScenarioItems::new)
    }
}

impl ::protobuf::Clear for VirtualScenarioItems {
    fn clear(&mut self) {
        self.items.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for VirtualScenarioItems {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for VirtualScenarioItems {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\nvirt.proto\"\xa0\x01\n\x13VirtualScenarioItem\x12\x1d\n\nevent_type\
    \x18\x01\x20\x01(\tR\teventType\x12\x12\n\x04port\x18\x02\x20\x01(\x05R\
    \x04port\x12\x0e\n\x02id\x18\x03\x20\x01(\tR\x02id\x12\x14\n\x05state\
    \x18\x04\x20\x01(\x08R\x05state\x12\x16\n\x06buffer\x18\x05\x20\x01(\x0c\
    R\x06buffer\x12\x18\n\x07timeout\x18\x06\x20\x01(\x05R\x07timeout\"B\n\
    \x14VirtualScenarioItems\x12*\n\x05items\x18\x01\x20\x03(\x0b2\x14.Virtu\
    alScenarioItemR\x05itemsb\x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
