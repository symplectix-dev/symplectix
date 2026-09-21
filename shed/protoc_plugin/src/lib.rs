#![allow(missing_docs)]
use std::io;
use std::io::{
    Read,
    Write,
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

pub trait GenCode {
    fn gen_code(&self, req: CodeGeneratorRequest) -> CodeGeneratorResponse;
}

pub trait GenFile {
    fn gen_file(&self, target_proto: &str, fd: &FileDescriptor) -> Result<File, String>;
}

impl<T: GenFile> GenCode for T {
    fn gen_code(&self, req: CodeGeneratorRequest) -> CodeGeneratorResponse {
        let pool = create_descriptor_pool(&req);

        let mut response = CodeGeneratorResponse {
            error: None,
            supported_features: Some(Feature::Proto3Optional as u64),
            file: Vec::with_capacity(req.file_to_generate.len()),
        };

        for target_proto in &req.file_to_generate {
            let file_desc = pool
                .get_file_by_name(target_proto.as_str())
                .expect("failed to fetch a file descriptor from a pool");

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

fn create_descriptor_pool(req: &CodeGeneratorRequest) -> DescriptorPool {
    let fd_set = FileDescriptorSet { file: req.proto_file.clone() };
    DescriptorPool::from_file_descriptor_set(fd_set).expect("failed to create descriptor pool")
}

pub fn gen_code<T: GenCode>(generator: T) -> anyhow::Result<()> {
    let mut buf = Vec::with_capacity(1 << 10);
    let n = io::stdin().read_to_end(&mut buf)?;
    let req = CodeGeneratorRequest::decode(&buf[..n])?;

    let resp = generator.gen_code(req);

    buf.clear();
    resp.encode(&mut buf)?;

    Ok(io::stdout().write_all(&buf)?)
}
