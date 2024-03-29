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
//! Generated file from `alarm.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_25_2;

#[derive(PartialEq,Clone,Default)]
#[cfg_attr(feature = "with-serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct AlarmZoneValue {
    // message fields
    pub value: f32,
    pub offset: f32,
    // special fields
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub unknown_fields: ::protobuf::UnknownFields,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a AlarmZoneValue {
    fn default() -> &'a AlarmZoneValue {
        <AlarmZoneValue as ::protobuf::Message>::default_instance()
    }
}

impl AlarmZoneValue {
    pub fn new() -> AlarmZoneValue {
        ::std::default::Default::default()
    }

    // float value = 1;


    pub fn get_value(&self) -> f32 {
        self.value
    }
    pub fn clear_value(&mut self) {
        self.value = 0.;
    }

    // Param is passed by value, moved
    pub fn set_value(&mut self, v: f32) {
        self.value = v;
    }

    // float offset = 2;


    pub fn get_offset(&self) -> f32 {
        self.offset
    }
    pub fn clear_offset(&mut self) {
        self.offset = 0.;
    }

    // Param is passed by value, moved
    pub fn set_offset(&mut self, v: f32) {
        self.offset = v;
    }
}

impl ::protobuf::Message for AlarmZoneValue {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.value = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.offset = tmp;
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
        if self.value != 0. {
            my_size += 5;
        }
        if self.offset != 0. {
            my_size += 5;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.value != 0. {
            os.write_float(1, self.value)?;
        }
        if self.offset != 0. {
            os.write_float(2, self.offset)?;
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

    fn new() -> AlarmZoneValue {
        AlarmZoneValue::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                "value",
                |m: &AlarmZoneValue| { &m.value },
                |m: &mut AlarmZoneValue| { &mut m.value },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                "offset",
                |m: &AlarmZoneValue| { &m.offset },
                |m: &mut AlarmZoneValue| { &mut m.offset },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<AlarmZoneValue>(
                "AlarmZoneValue",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static AlarmZoneValue {
        static instance: ::protobuf::rt::LazyV2<AlarmZoneValue> = ::protobuf::rt::LazyV2::INIT;
        instance.get(AlarmZoneValue::new)
    }
}

impl ::protobuf::Clear for AlarmZoneValue {
    fn clear(&mut self) {
        self.value = 0.;
        self.offset = 0.;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for AlarmZoneValue {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for AlarmZoneValue {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
#[cfg_attr(feature = "with-serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct FieldAlarm {
    // message fields
    pub moduleId: ::std::string::String,
    pub property: ::std::string::String,
    pub veryLow: ::protobuf::SingularPtrField<AlarmZoneValue>,
    pub low: ::protobuf::SingularPtrField<AlarmZoneValue>,
    pub high: ::protobuf::SingularPtrField<AlarmZoneValue>,
    pub veryHigh: ::protobuf::SingularPtrField<AlarmZoneValue>,
    pub sync: ::protobuf::SingularPtrField<super::sync::SyncInfo>,
    // special fields
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub unknown_fields: ::protobuf::UnknownFields,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a FieldAlarm {
    fn default() -> &'a FieldAlarm {
        <FieldAlarm as ::protobuf::Message>::default_instance()
    }
}

impl FieldAlarm {
    pub fn new() -> FieldAlarm {
        ::std::default::Default::default()
    }

    // string moduleId = 1;


    pub fn get_moduleId(&self) -> &str {
        &self.moduleId
    }
    pub fn clear_moduleId(&mut self) {
        self.moduleId.clear();
    }

    // Param is passed by value, moved
    pub fn set_moduleId(&mut self, v: ::std::string::String) {
        self.moduleId = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_moduleId(&mut self) -> &mut ::std::string::String {
        &mut self.moduleId
    }

    // Take field
    pub fn take_moduleId(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.moduleId, ::std::string::String::new())
    }

    // string property = 2;


    pub fn get_property(&self) -> &str {
        &self.property
    }
    pub fn clear_property(&mut self) {
        self.property.clear();
    }

    // Param is passed by value, moved
    pub fn set_property(&mut self, v: ::std::string::String) {
        self.property = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_property(&mut self) -> &mut ::std::string::String {
        &mut self.property
    }

    // Take field
    pub fn take_property(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.property, ::std::string::String::new())
    }

    // .AlarmZoneValue veryLow = 3;


    pub fn get_veryLow(&self) -> &AlarmZoneValue {
        self.veryLow.as_ref().unwrap_or_else(|| <AlarmZoneValue as ::protobuf::Message>::default_instance())
    }
    pub fn clear_veryLow(&mut self) {
        self.veryLow.clear();
    }

    pub fn has_veryLow(&self) -> bool {
        self.veryLow.is_some()
    }

    // Param is passed by value, moved
    pub fn set_veryLow(&mut self, v: AlarmZoneValue) {
        self.veryLow = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_veryLow(&mut self) -> &mut AlarmZoneValue {
        if self.veryLow.is_none() {
            self.veryLow.set_default();
        }
        self.veryLow.as_mut().unwrap()
    }

    // Take field
    pub fn take_veryLow(&mut self) -> AlarmZoneValue {
        self.veryLow.take().unwrap_or_else(|| AlarmZoneValue::new())
    }

    // .AlarmZoneValue low = 4;


    pub fn get_low(&self) -> &AlarmZoneValue {
        self.low.as_ref().unwrap_or_else(|| <AlarmZoneValue as ::protobuf::Message>::default_instance())
    }
    pub fn clear_low(&mut self) {
        self.low.clear();
    }

    pub fn has_low(&self) -> bool {
        self.low.is_some()
    }

    // Param is passed by value, moved
    pub fn set_low(&mut self, v: AlarmZoneValue) {
        self.low = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_low(&mut self) -> &mut AlarmZoneValue {
        if self.low.is_none() {
            self.low.set_default();
        }
        self.low.as_mut().unwrap()
    }

    // Take field
    pub fn take_low(&mut self) -> AlarmZoneValue {
        self.low.take().unwrap_or_else(|| AlarmZoneValue::new())
    }

    // .AlarmZoneValue high = 5;


    pub fn get_high(&self) -> &AlarmZoneValue {
        self.high.as_ref().unwrap_or_else(|| <AlarmZoneValue as ::protobuf::Message>::default_instance())
    }
    pub fn clear_high(&mut self) {
        self.high.clear();
    }

    pub fn has_high(&self) -> bool {
        self.high.is_some()
    }

    // Param is passed by value, moved
    pub fn set_high(&mut self, v: AlarmZoneValue) {
        self.high = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_high(&mut self) -> &mut AlarmZoneValue {
        if self.high.is_none() {
            self.high.set_default();
        }
        self.high.as_mut().unwrap()
    }

    // Take field
    pub fn take_high(&mut self) -> AlarmZoneValue {
        self.high.take().unwrap_or_else(|| AlarmZoneValue::new())
    }

    // .AlarmZoneValue veryHigh = 6;


    pub fn get_veryHigh(&self) -> &AlarmZoneValue {
        self.veryHigh.as_ref().unwrap_or_else(|| <AlarmZoneValue as ::protobuf::Message>::default_instance())
    }
    pub fn clear_veryHigh(&mut self) {
        self.veryHigh.clear();
    }

    pub fn has_veryHigh(&self) -> bool {
        self.veryHigh.is_some()
    }

    // Param is passed by value, moved
    pub fn set_veryHigh(&mut self, v: AlarmZoneValue) {
        self.veryHigh = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_veryHigh(&mut self) -> &mut AlarmZoneValue {
        if self.veryHigh.is_none() {
            self.veryHigh.set_default();
        }
        self.veryHigh.as_mut().unwrap()
    }

    // Take field
    pub fn take_veryHigh(&mut self) -> AlarmZoneValue {
        self.veryHigh.take().unwrap_or_else(|| AlarmZoneValue::new())
    }

    // .SyncInfo sync = 20;


    pub fn get_sync(&self) -> &super::sync::SyncInfo {
        self.sync.as_ref().unwrap_or_else(|| <super::sync::SyncInfo as ::protobuf::Message>::default_instance())
    }
    pub fn clear_sync(&mut self) {
        self.sync.clear();
    }

    pub fn has_sync(&self) -> bool {
        self.sync.is_some()
    }

    // Param is passed by value, moved
    pub fn set_sync(&mut self, v: super::sync::SyncInfo) {
        self.sync = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_sync(&mut self) -> &mut super::sync::SyncInfo {
        if self.sync.is_none() {
            self.sync.set_default();
        }
        self.sync.as_mut().unwrap()
    }

    // Take field
    pub fn take_sync(&mut self) -> super::sync::SyncInfo {
        self.sync.take().unwrap_or_else(|| super::sync::SyncInfo::new())
    }
}

impl ::protobuf::Message for FieldAlarm {
    fn is_initialized(&self) -> bool {
        for v in &self.veryLow {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.low {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.high {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.veryHigh {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.sync {
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
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.moduleId)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.property)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.veryLow)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.low)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.high)?;
                },
                6 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.veryHigh)?;
                },
                20 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.sync)?;
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
        if !self.moduleId.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.moduleId);
        }
        if !self.property.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.property);
        }
        if let Some(ref v) = self.veryLow.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.low.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.high.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.veryHigh.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.sync.as_ref() {
            let len = v.compute_size();
            my_size += 2 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.moduleId.is_empty() {
            os.write_string(1, &self.moduleId)?;
        }
        if !self.property.is_empty() {
            os.write_string(2, &self.property)?;
        }
        if let Some(ref v) = self.veryLow.as_ref() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.low.as_ref() {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.high.as_ref() {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.veryHigh.as_ref() {
            os.write_tag(6, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.sync.as_ref() {
            os.write_tag(20, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
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

    fn new() -> FieldAlarm {
        FieldAlarm::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "moduleId",
                |m: &FieldAlarm| { &m.moduleId },
                |m: &mut FieldAlarm| { &mut m.moduleId },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "property",
                |m: &FieldAlarm| { &m.property },
                |m: &mut FieldAlarm| { &mut m.property },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<AlarmZoneValue>>(
                "veryLow",
                |m: &FieldAlarm| { &m.veryLow },
                |m: &mut FieldAlarm| { &mut m.veryLow },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<AlarmZoneValue>>(
                "low",
                |m: &FieldAlarm| { &m.low },
                |m: &mut FieldAlarm| { &mut m.low },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<AlarmZoneValue>>(
                "high",
                |m: &FieldAlarm| { &m.high },
                |m: &mut FieldAlarm| { &mut m.high },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<AlarmZoneValue>>(
                "veryHigh",
                |m: &FieldAlarm| { &m.veryHigh },
                |m: &mut FieldAlarm| { &mut m.veryHigh },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::sync::SyncInfo>>(
                "sync",
                |m: &FieldAlarm| { &m.sync },
                |m: &mut FieldAlarm| { &mut m.sync },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<FieldAlarm>(
                "FieldAlarm",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static FieldAlarm {
        static instance: ::protobuf::rt::LazyV2<FieldAlarm> = ::protobuf::rt::LazyV2::INIT;
        instance.get(FieldAlarm::new)
    }
}

impl ::protobuf::Clear for FieldAlarm {
    fn clear(&mut self) {
        self.moduleId.clear();
        self.property.clear();
        self.veryLow.clear();
        self.low.clear();
        self.high.clear();
        self.veryHigh.clear();
        self.sync.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for FieldAlarm {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for FieldAlarm {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
#[cfg_attr(feature = "with-serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct FieldAlarmEvent {
    // message fields
    pub moduleId: ::std::string::String,
    pub property: ::std::string::String,
    pub previousZone: AlarmZone,
    pub currentZone: AlarmZone,
    pub currentValue: f32,
    pub previousValue: f32,
    pub changedAt: u32,
    // special fields
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub unknown_fields: ::protobuf::UnknownFields,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a FieldAlarmEvent {
    fn default() -> &'a FieldAlarmEvent {
        <FieldAlarmEvent as ::protobuf::Message>::default_instance()
    }
}

impl FieldAlarmEvent {
    pub fn new() -> FieldAlarmEvent {
        ::std::default::Default::default()
    }

    // string moduleId = 1;


    pub fn get_moduleId(&self) -> &str {
        &self.moduleId
    }
    pub fn clear_moduleId(&mut self) {
        self.moduleId.clear();
    }

    // Param is passed by value, moved
    pub fn set_moduleId(&mut self, v: ::std::string::String) {
        self.moduleId = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_moduleId(&mut self) -> &mut ::std::string::String {
        &mut self.moduleId
    }

    // Take field
    pub fn take_moduleId(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.moduleId, ::std::string::String::new())
    }

    // string property = 2;


    pub fn get_property(&self) -> &str {
        &self.property
    }
    pub fn clear_property(&mut self) {
        self.property.clear();
    }

    // Param is passed by value, moved
    pub fn set_property(&mut self, v: ::std::string::String) {
        self.property = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_property(&mut self) -> &mut ::std::string::String {
        &mut self.property
    }

    // Take field
    pub fn take_property(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.property, ::std::string::String::new())
    }

    // .AlarmZone previousZone = 3;


    pub fn get_previousZone(&self) -> AlarmZone {
        self.previousZone
    }
    pub fn clear_previousZone(&mut self) {
        self.previousZone = AlarmZone::UNKNOW;
    }

    // Param is passed by value, moved
    pub fn set_previousZone(&mut self, v: AlarmZone) {
        self.previousZone = v;
    }

    // .AlarmZone currentZone = 4;


    pub fn get_currentZone(&self) -> AlarmZone {
        self.currentZone
    }
    pub fn clear_currentZone(&mut self) {
        self.currentZone = AlarmZone::UNKNOW;
    }

    // Param is passed by value, moved
    pub fn set_currentZone(&mut self, v: AlarmZone) {
        self.currentZone = v;
    }

    // float currentValue = 5;


    pub fn get_currentValue(&self) -> f32 {
        self.currentValue
    }
    pub fn clear_currentValue(&mut self) {
        self.currentValue = 0.;
    }

    // Param is passed by value, moved
    pub fn set_currentValue(&mut self, v: f32) {
        self.currentValue = v;
    }

    // float previousValue = 6;


    pub fn get_previousValue(&self) -> f32 {
        self.previousValue
    }
    pub fn clear_previousValue(&mut self) {
        self.previousValue = 0.;
    }

    // Param is passed by value, moved
    pub fn set_previousValue(&mut self, v: f32) {
        self.previousValue = v;
    }

    // uint32 changedAt = 7;


    pub fn get_changedAt(&self) -> u32 {
        self.changedAt
    }
    pub fn clear_changedAt(&mut self) {
        self.changedAt = 0;
    }

    // Param is passed by value, moved
    pub fn set_changedAt(&mut self, v: u32) {
        self.changedAt = v;
    }
}

impl ::protobuf::Message for FieldAlarmEvent {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.moduleId)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.property)?;
                },
                3 => {
                    ::protobuf::rt::read_proto3_enum_with_unknown_fields_into(wire_type, is, &mut self.previousZone, 3, &mut self.unknown_fields)?
                },
                4 => {
                    ::protobuf::rt::read_proto3_enum_with_unknown_fields_into(wire_type, is, &mut self.currentZone, 4, &mut self.unknown_fields)?
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.currentValue = tmp;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.previousValue = tmp;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.changedAt = tmp;
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
        if !self.moduleId.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.moduleId);
        }
        if !self.property.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.property);
        }
        if self.previousZone != AlarmZone::UNKNOW {
            my_size += ::protobuf::rt::enum_size(3, self.previousZone);
        }
        if self.currentZone != AlarmZone::UNKNOW {
            my_size += ::protobuf::rt::enum_size(4, self.currentZone);
        }
        if self.currentValue != 0. {
            my_size += 5;
        }
        if self.previousValue != 0. {
            my_size += 5;
        }
        if self.changedAt != 0 {
            my_size += ::protobuf::rt::value_size(7, self.changedAt, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.moduleId.is_empty() {
            os.write_string(1, &self.moduleId)?;
        }
        if !self.property.is_empty() {
            os.write_string(2, &self.property)?;
        }
        if self.previousZone != AlarmZone::UNKNOW {
            os.write_enum(3, ::protobuf::ProtobufEnum::value(&self.previousZone))?;
        }
        if self.currentZone != AlarmZone::UNKNOW {
            os.write_enum(4, ::protobuf::ProtobufEnum::value(&self.currentZone))?;
        }
        if self.currentValue != 0. {
            os.write_float(5, self.currentValue)?;
        }
        if self.previousValue != 0. {
            os.write_float(6, self.previousValue)?;
        }
        if self.changedAt != 0 {
            os.write_uint32(7, self.changedAt)?;
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

    fn new() -> FieldAlarmEvent {
        FieldAlarmEvent::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "moduleId",
                |m: &FieldAlarmEvent| { &m.moduleId },
                |m: &mut FieldAlarmEvent| { &mut m.moduleId },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "property",
                |m: &FieldAlarmEvent| { &m.property },
                |m: &mut FieldAlarmEvent| { &mut m.property },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<AlarmZone>>(
                "previousZone",
                |m: &FieldAlarmEvent| { &m.previousZone },
                |m: &mut FieldAlarmEvent| { &mut m.previousZone },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<AlarmZone>>(
                "currentZone",
                |m: &FieldAlarmEvent| { &m.currentZone },
                |m: &mut FieldAlarmEvent| { &mut m.currentZone },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                "currentValue",
                |m: &FieldAlarmEvent| { &m.currentValue },
                |m: &mut FieldAlarmEvent| { &mut m.currentValue },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                "previousValue",
                |m: &FieldAlarmEvent| { &m.previousValue },
                |m: &mut FieldAlarmEvent| { &mut m.previousValue },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                "changedAt",
                |m: &FieldAlarmEvent| { &m.changedAt },
                |m: &mut FieldAlarmEvent| { &mut m.changedAt },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<FieldAlarmEvent>(
                "FieldAlarmEvent",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static FieldAlarmEvent {
        static instance: ::protobuf::rt::LazyV2<FieldAlarmEvent> = ::protobuf::rt::LazyV2::INIT;
        instance.get(FieldAlarmEvent::new)
    }
}

impl ::protobuf::Clear for FieldAlarmEvent {
    fn clear(&mut self) {
        self.moduleId.clear();
        self.property.clear();
        self.previousZone = AlarmZone::UNKNOW;
        self.currentZone = AlarmZone::UNKNOW;
        self.currentValue = 0.;
        self.previousValue = 0.;
        self.changedAt = 0;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for FieldAlarmEvent {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for FieldAlarmEvent {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
#[cfg_attr(feature = "with-serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum AlarmZone {
    UNKNOW = 0,
    MIDDLE = 1,
    VERY_LOW = 2,
    LOW = 3,
    HIGH = 4,
    VERY_HIGH = 5,
}

impl ::protobuf::ProtobufEnum for AlarmZone {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<AlarmZone> {
        match value {
            0 => ::std::option::Option::Some(AlarmZone::UNKNOW),
            1 => ::std::option::Option::Some(AlarmZone::MIDDLE),
            2 => ::std::option::Option::Some(AlarmZone::VERY_LOW),
            3 => ::std::option::Option::Some(AlarmZone::LOW),
            4 => ::std::option::Option::Some(AlarmZone::HIGH),
            5 => ::std::option::Option::Some(AlarmZone::VERY_HIGH),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [AlarmZone] = &[
            AlarmZone::UNKNOW,
            AlarmZone::MIDDLE,
            AlarmZone::VERY_LOW,
            AlarmZone::LOW,
            AlarmZone::HIGH,
            AlarmZone::VERY_HIGH,
        ];
        values
    }

    fn enum_descriptor_static() -> &'static ::protobuf::reflect::EnumDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::EnumDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            ::protobuf::reflect::EnumDescriptor::new_pb_name::<AlarmZone>("AlarmZone", file_descriptor_proto())
        })
    }
}

impl ::std::marker::Copy for AlarmZone {
}

impl ::std::default::Default for AlarmZone {
    fn default() -> Self {
        AlarmZone::UNKNOW
    }
}

impl ::protobuf::reflect::ProtobufValue for AlarmZone {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Enum(::protobuf::ProtobufEnum::descriptor(self))
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0balarm.proto\x1a\nsync.proto\">\n\x0eAlarmZoneValue\x12\x14\n\x05va\
    lue\x18\x01\x20\x01(\x02R\x05value\x12\x16\n\x06offset\x18\x02\x20\x01(\
    \x02R\x06offset\"\x83\x02\n\nFieldAlarm\x12\x1a\n\x08moduleId\x18\x01\
    \x20\x01(\tR\x08moduleId\x12\x1a\n\x08property\x18\x02\x20\x01(\tR\x08pr\
    operty\x12)\n\x07veryLow\x18\x03\x20\x01(\x0b2\x0f.AlarmZoneValueR\x07ve\
    ryLow\x12!\n\x03low\x18\x04\x20\x01(\x0b2\x0f.AlarmZoneValueR\x03low\x12\
    #\n\x04high\x18\x05\x20\x01(\x0b2\x0f.AlarmZoneValueR\x04high\x12+\n\x08\
    veryHigh\x18\x06\x20\x01(\x0b2\x0f.AlarmZoneValueR\x08veryHigh\x12\x1d\n\
    \x04sync\x18\x14\x20\x01(\x0b2\t.SyncInfoR\x04sync\"\x8f\x02\n\x0fFieldA\
    larmEvent\x12\x1a\n\x08moduleId\x18\x01\x20\x01(\tR\x08moduleId\x12\x1a\
    \n\x08property\x18\x02\x20\x01(\tR\x08property\x12.\n\x0cpreviousZone\
    \x18\x03\x20\x01(\x0e2\n.AlarmZoneR\x0cpreviousZone\x12,\n\x0bcurrentZon\
    e\x18\x04\x20\x01(\x0e2\n.AlarmZoneR\x0bcurrentZone\x12\"\n\x0ccurrentVa\
    lue\x18\x05\x20\x01(\x02R\x0ccurrentValue\x12$\n\rpreviousValue\x18\x06\
    \x20\x01(\x02R\rpreviousValue\x12\x1c\n\tchangedAt\x18\x07\x20\x01(\rR\t\
    changedAt*S\n\tAlarmZone\x12\n\n\x06UNKNOW\x10\0\x12\n\n\x06MIDDLE\x10\
    \x01\x12\x0c\n\x08VERY_LOW\x10\x02\x12\x07\n\x03LOW\x10\x03\x12\x08\n\
    \x04HIGH\x10\x04\x12\r\n\tVERY_HIGH\x10\x05B+\n)ca.berlingoqc.growbe_and\
    roid_module.protob\x06proto3\
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
