use crate::{
    ids::{PluginID, SystemID, SystemTypeID},
    private::{id_store::IDStore, system::System},
};

pub(crate) type SystemStorage = IDStore<(PluginID, SystemTypeID), SystemID, System>;
