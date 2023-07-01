use fxhash::FxBuildHasher;

pub type Map<K, V> = hashbrown::HashMap<K, V, FxBuildHasher>;
