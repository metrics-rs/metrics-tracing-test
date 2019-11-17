#[derive(Default, Debug)]
pub struct Thingy;

impl Thingy {
    #[tracing::instrument]
    pub fn handle_unshaved(&self, yak: usize) {}
}
