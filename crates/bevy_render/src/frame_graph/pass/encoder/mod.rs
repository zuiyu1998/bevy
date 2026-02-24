use crate::frame_graph::{PassCommand, PassContext};

pub trait EncoderCommand: 'static + Send + Sync {
    fn execute(&self, context: &mut PassContext);
}

#[derive(Default)]
pub struct EncoderCommands {
    pub(crate) commands: Vec<Box<dyn EncoderCommand>>,
}

impl PassCommand for EncoderCommands {
    fn execute(&self, context: &mut PassContext) {
        for command in self.commands.iter() {
            command.execute(context);
        }
    }
}
