# Serialization

This repo makes heavy use of both _serialization_ and _deserialization_ and most of that functionality is derived from the popular [**serde**]() crate. Here's we're simply providing a handy reference to key aspects of **serde** for contributors to this crate.

## The `Value` enum

The **serde** crate exposes a `Value` enumeration which is a generic way of storing values. Each variant it provides is a  