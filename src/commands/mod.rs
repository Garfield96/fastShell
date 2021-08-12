use crate::intermediate::Intermediate;

pub trait Command {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>);
}
pub mod cat;
pub mod echo;
pub mod grep;
pub mod head;
pub mod shellCommand;
pub mod shuf;
pub mod sort;
pub mod tail;
pub mod test;
pub mod uniq;
pub mod wc;
