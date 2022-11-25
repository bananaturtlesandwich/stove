use unreal_asset::Asset;

impl super::Actor {
    pub fn transplant(&self, recipient: &mut Asset, donor: &Asset) {
        let children = self.get_actor_exports(donor, donor.exports.len());
    }
}
