use crate::models::ctx::CtxError;
use crate::models::link::LinkError;
use crate::models::streaming_server::Settings as StreamingServerSettings;
use crate::runtime::EnvError;
use crate::types::addon::{Descriptor, Manifest, ResourceRequest, ResourceResponse};
use crate::types::api::{
    APIRequest, AuthRequest, DatastoreRequest, LinkCodeResponse, LinkDataResponse,
};
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::profile::{Auth, Profile};
use url::Url;

pub type CtxStorageResponse = (
    Option<Profile>,
    Option<LibraryBucket>,
    Option<LibraryBucket>,
);

pub type AuthResponse = (Auth, Vec<Descriptor>, Vec<LibraryItem>);

pub type LibraryPlanResponse = (Vec<String>, Vec<String>);

//
// Those messages are meant to be dispatched and hanled only inside stremio-core crate
//
pub enum Internal {
    // Result for authenticate to API.
    CtxAuthResult(AuthRequest, Result<AuthResponse, CtxError>),
    // Result for pull addons from API.
    AddonsAPIResult(APIRequest, Result<Vec<Descriptor>, CtxError>),
    // Result for library sync plan with API.
    LibrarySyncPlanResult(DatastoreRequest, Result<LibraryPlanResponse, CtxError>),
    // Result for pull library items from API.
    LibraryPullResult(DatastoreRequest, Result<Vec<LibraryItem>, CtxError>),
    // Dispatched when library item needs to be updated in the memory, storage and API.
    UpdateLibraryItem(LibraryItem),
    // Dispatched when some of auth, addons or settings changed.
    ProfileChanged,
    // Dispatched when library changes with a flag if its already persisted.
    LibraryChanged(bool),
    // Dispatched when streams bucket changes with a flag if its already persisted.
    StreamsChanged(bool),
    // Result for loading link code.
    LinkCodeResult(Result<LinkCodeResponse, LinkError>),
    // Result for loading link data.
    LinkDataResult(String, Result<LinkDataResponse, LinkError>),
    // Result for loading streaming server settings.
    StreamingServerSettingsResult(Url, Result<StreamingServerSettings, EnvError>),
    // Result for loading streaming server base url.
    StreamingServerBaseURLResult(Url, Result<Url, EnvError>),
    // Result for updating streaming server settings.
    StreamingServerUpdateSettingsResult(Url, Result<(), EnvError>),
    // Result for fetching resource from addons.
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    // Result for fetching manifest from addon.
    ManifestRequestResult(Url, Result<Manifest, EnvError>),
}
