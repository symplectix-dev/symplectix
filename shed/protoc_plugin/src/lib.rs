//! Scaffolding for a protoc plugin. A plugin implements
//! [`GenFile`] and calls [`run`] from its `main`.

use std::collections::BTreeMap;
use std::io::Write;
use std::{
    io,
    mem,
};

use protobuf::plugin::code_generator_response::{
    Feature,
    File,
};
use protobuf::plugin::{
    CodeGeneratorRequest,
    CodeGeneratorResponse,
};
use protobuf::reflect::FileDescriptor;
use protobuf::{
    Enum,
    Message,
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
        let descriptors = build_descriptors(&mut req);

        let mut response = CodeGeneratorResponse {
            error: None,
            supported_features: Some(Feature::FEATURE_PROTO3_OPTIONAL.value() as u64),
            file: Vec::with_capacity(req.file_to_generate.len()),
            ..Default::default()
        };

        for target_proto in &req.file_to_generate {
            let file_desc = descriptors
                .get(target_proto.as_str())
                .expect("target proto is missing from the request");

            match self.gen_file(target_proto, file_desc) {
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

/// Every file in the request, by name, imports included. protoc sends
/// those along with the protos to generate, so the request resolves
/// against itself and needs no descriptors from outside it, the
/// well-known types included.
fn build_descriptors(req: &mut CodeGeneratorRequest) -> BTreeMap<String, FileDescriptor> {
    FileDescriptor::new_dynamic_fds(mem::take(&mut req.proto_file), &[])
        .expect("failed to build file descriptors")
        .into_iter()
        .map(|fd| (fd.name().to_owned(), fd))
        .collect()
}

/// Reads the request from stdin, runs `generator`, and writes the
/// response to stdout, which is the protocol protoc speaks to a plugin.
pub fn run<T: GenCode>(generator: T) -> anyhow::Result<()> {
    let req = CodeGeneratorRequest::parse_from_reader(&mut io::stdin().lock())?;
    let resp = generator.gen_code(req);

    // `write_to_writer` buffers through a `CodedOutputStream` of its own
    // and flushes that into stdout, leaving stdout's own buffer to us.
    let mut stdout = io::stdout().lock();
    resp.write_to_writer(&mut stdout)?;
    Ok(stdout.flush()?)
}
