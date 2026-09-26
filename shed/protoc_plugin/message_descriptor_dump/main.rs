//! An example protoc plugin, dumping the message descriptors of each
//! proto it is given.

use protobuf::plugin::code_generator_response::File;
use protobuf::reflect::FileDescriptor;

fn main() -> anyhow::Result<()> {
    protoc_plugin::run(MessageDescriptorDump::default())
}

#[derive(Debug, Default, Clone)]
struct MessageDescriptorDump {}

impl protoc_plugin::GenFile for MessageDescriptorDump {
    fn gen_file(&self, target_proto: &str, fd: &FileDescriptor) -> Result<File, String> {
        let file_name = {
            let stem = target_proto
                .strip_suffix(".proto")
                .ok_or_else(|| format!("unexpected proto '{}'", target_proto))?;
            format!("{}.message_descriptor_dump", stem)
        };

        let mut buf = String::with_capacity(1 << 10);
        for msg_desc in fd.messages() {
            buf.push_str(&format!("{:#?}\n", msg_desc));
        }

        Ok(File { name: Some(file_name), content: Some(buf), ..Default::default() })
    }
}
