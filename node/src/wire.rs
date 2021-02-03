use crate::state::State;

pub trait Wire {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(state: &State, data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn len() -> usize;
}
