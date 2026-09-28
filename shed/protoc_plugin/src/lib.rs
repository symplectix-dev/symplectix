//! Scaffolding for a protoc plugin. A plugin implements
//! [`GenFile`] and calls [`run`] from its `main`.

use std::io::{
    Read,
    Write,
};
use std::{
    io,
    mem,
};

use prost::Message;
use prost_reflect::{
    DescriptorPool,
    FileDescriptor,
};
use prost_types::FileDescriptorSet;
use prost_types::compiler::code_generator_response::{
    Feature,
    File,
};
use prost_types::compiler::{
    CodeGeneratorRequest,
    CodeGeneratorResponse,
};

/// A plugin: one request in, one response out.
///
/// Implement it directly only for a plugin that does not emit one file
/// per proto, such as one writing to an insertion point. Anything else
/// wants [`GenFile`], which a blanket impl lifts into a `GenCode`.
pub trait GenCode {
    /// Generates every file the request asks for.
    fn gen_code(&self, req: CodeGeneratorRequest) -> CodeGeneratorResponse;
}

/// A plugin emitting one file per proto protoc asked for.
///
/// `target_proto` is that proto's path as the request spells it, and
/// `fd` its descriptor with the imports already resolved. An `Err`
/// becomes the response's error, which abandons the run: protoc reports
/// the message and writes nothing, the files already generated included.
pub trait GenFile {
    /// Generates the one file for `target_proto`.
    fn gen_file(&self, target_proto: &str, fd: &FileDescriptor) -> Result<File, String>;
}

impl<T: GenFile> GenCode for T {
    fn gen_code(&self, mut req: CodeGeneratorRequest) -> CodeGeneratorResponse {
        let pool = build_descriptor_pool(&mut req);

        let mut response = CodeGeneratorResponse {
            error: None,
            supported_features: Some(Feature::Proto3Optional as u64),
            file: Vec::with_capacity(req.file_to_generate.len()),
        };

        for target_proto in &req.file_to_generate {
            let file_desc = pool
                .get_file_by_name(target_proto)
                .expect("target proto is missing from the request");

            match self.gen_file(target_proto, &file_desc) {
                Ok(generated_file) => response.file.push(generated_file),
                Err(err_message) => {
                    response.error = Some(err_message);
                    return response;
                }
            }
        }

        response
    }
}

/// Every file in the request, imports included. protoc sends those along
/// with the protos to generate, so the request resolves against itself
/// and needs no descriptors from outside it, the well-known types
/// included.
fn build_descriptor_pool(req: &mut CodeGeneratorRequest) -> DescriptorPool {
    let files = FileDescriptorSet { file: mem::take(&mut req.proto_file) };
    DescriptorPool::from_file_descriptor_set(files).expect("failed to build the descriptor pool")
}

/// Reads the request from stdin, runs `generator`, and writes the
/// response to stdout, which is the protocol protoc speaks to a plugin.
///
/// prost parses a `Buf` rather than a reader, so the request is read
/// whole before it is parsed, and the same buffer carries the response
/// back out.
pub fn run<T: GenCode>(generator: T) -> anyhow::Result<()> {
    let mut buf = Vec::with_capacity(1 << 10);
    io::stdin().lock().read_to_end(&mut buf)?;
    let req = CodeGeneratorRequest::decode(&buf[..])?;

    let resp = generator.gen_code(req);

    buf.clear();
    resp.encode(&mut buf)?;

    let mut stdout = io::stdout().lock();
    stdout.write_all(&buf)?;
    Ok(stdout.flush()?)
}
