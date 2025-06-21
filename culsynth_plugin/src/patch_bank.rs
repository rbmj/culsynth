use culsynth::voice::modulation::ModMatrix;
use culsynth::voice::VoiceParams;

trait Patch: Clone {
    fn name(&self) -> &str;
    fn tags(&self) -> impl Iterator<Item = &str>;
    fn load(&self) -> (VoiceParams<i16>, ModMatrix<i16>);
}

trait PatchBank {
    fn iter(&self) -> impl Iterator<Item = impl Patch>;
}
