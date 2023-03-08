use crate::constants::LIBRARY_COLLECTION_NAME;
use crate::models::common::{DescriptorLoadable, ResourceLoadable};
use crate::models::ctx::{
    update_library, update_notifications, update_profile, update_trakt_addon, CtxError,
};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt, Update};
use crate::types::api::{
    fetch_api, APIRequest, APIResult, AuthRequest, AuthResponse, CollectionResponse,
    DatastoreCommand, DatastoreRequest, LibraryItemsResponse, SuccessResponse,
};
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::{Auth, AuthKey, Profile};
use crate::types::resource::MetaItem;
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};
use serde::Serialize;

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub enum CtxStatus {
    Loading(AuthRequest),
    Ready,
}

#[derive(Serialize, Clone, Debug)]
pub struct Ctx {
    pub profile: Profile,
    // TODO StreamsBucket
    // TODO SubtitlesBucket
    // TODO SearchesBucket
    #[serde(skip)]
    pub library: LibraryBucket,
    #[serde(skip)]
    pub notifications: NotificationsBucket,
    #[serde(skip)]
    pub status: CtxStatus,
    #[serde(skip)]
    pub trakt_addon: Option<DescriptorLoadable>,
    /// Last videos catalog of all MetaItems that are marked for receiving notifications for new episodes.
    ///
    /// Loaded from all the user's addons.
    #[serde(skip)]
    pub last_videos_catalogs: Vec<ResourceLoadable<Vec<MetaItem>>>,
}

impl Ctx {
    pub fn new(
        profile: Profile,
        library: LibraryBucket,
        notifications: NotificationsBucket,
    ) -> Self {
        Self {
            profile,
            library,
            notifications,
            status: CtxStatus::Ready,
            trakt_addon: None,
            last_videos_catalogs: vec![],
        }
    }
}

impl<E: Env + 'static> Update<E> for Ctx {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Ctx(ActionCtx::Authenticate(auth_request))) => {
                self.status = CtxStatus::Loading(auth_request.to_owned());
                Effects::one(authenticate::<E>(auth_request)).unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) | Msg::Internal(Internal::Logout) => {
                let uid = self.profile.uid();
                let session_effects = match self.profile.auth_key() {
                    Some(auth_key) => Effects::one(delete_session::<E>(auth_key)).unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
                let trakt_addon_effects = update_trakt_addon::<E>(
                    &mut self.trakt_addon,
                    &self.profile,
                    &self.status,
                    msg,
                );
                let notifications_effects = update_notifications::<E>(
                    &mut self.notifications,
                    &mut self.last_videos_catalogs,
                    &self.profile,
                    &self.library,
                    &self.status,
                    msg,
                );
                self.status = CtxStatus::Ready;
                Effects::msg(Msg::Event(Event::UserLoggedOut { uid }))
                    .unchanged()
                    .join(session_effects)
                    .join(profile_effects)
                    .join(library_effects)
                    .join(trakt_addon_effects)
                    .join(notifications_effects)
            }
            Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => {
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
                let trakt_addon_effects = update_trakt_addon::<E>(
                    &mut self.trakt_addon,
                    &self.profile,
                    &self.status,
                    msg,
                );
                let notifications_effects = update_notifications::<E>(
                    &mut self.notifications,
                    &mut self.last_videos_catalogs,
                    &self.profile,
                    &self.library,
                    &self.status,
                    msg,
                );
                let ctx_effects = match &self.status {
                    CtxStatus::Loading(loading_auth_request)
                        if loading_auth_request == auth_request =>
                    {
                        self.status = CtxStatus::Ready;
                        match result {
                            Ok(_) => Effects::msg(Msg::Event(Event::UserAuthenticated {
                                auth_request: auth_request.to_owned(),
                            }))
                            .unchanged(),
                            Err(error) => Effects::msg(Msg::Event(Event::Error {
                                error: error.to_owned(),
                                source: Box::new(Event::UserAuthenticated {
                                    auth_request: auth_request.to_owned(),
                                }),
                            }))
                            .unchanged(),
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                profile_effects
                    .join(library_effects)
                    .join(trakt_addon_effects)
                    .join(notifications_effects)
                    .join(ctx_effects)
            }
            _ => {
                let profile_effects = update_profile::<E>(&mut self.profile, &self.status, msg);
                let library_effects =
                    update_library::<E>(&mut self.library, &self.profile, &self.status, msg);
                let trakt_addon_effects = update_trakt_addon::<E>(
                    &mut self.trakt_addon,
                    &self.profile,
                    &self.status,
                    msg,
                );
                let notifications_effects = update_notifications::<E>(
                    &mut self.notifications,
                    &mut self.last_videos_catalogs,
                    &self.profile,
                    &self.library,
                    &self.status,
                    msg,
                );
                profile_effects
                    .join(library_effects)
                    .join(trakt_addon_effects)
                    .join(notifications_effects)
            }
        }
    }
}

fn authenticate<E: Env + 'static>(auth_request: &AuthRequest) -> Effect {
    EffectFuture::Concurrent(
        E::flush_analytics()
            .then(enclose!((auth_request) move |_| {
                fetch_api::<E, _, _, _>(&APIRequest::Auth(auth_request))
            }))
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok { result } => future::ok(result),
                APIResult::Err { error } => future::err(CtxError::from(error)),
            })
            .map_ok(|AuthResponse { key, user }| Auth { key, user })
            .and_then(|auth| {
                future::try_join(
                    fetch_api::<E, _, _, _>(&APIRequest::AddonCollectionGet {
                        auth_key: auth.key.to_owned(),
                        update: true,
                    })
                    .map_err(CtxError::from)
                    .and_then(|result| match result {
                        APIResult::Ok { result } => future::ok(result),
                        APIResult::Err { error } => future::err(CtxError::from(error)),
                    })
                    .map_ok(|CollectionResponse { addons, .. }| addons),
                    fetch_api::<E, _, _, LibraryItemsResponse>(&DatastoreRequest {
                        auth_key: auth.key.to_owned(),
                        collection: LIBRARY_COLLECTION_NAME.to_owned(),
                        command: DatastoreCommand::Get {
                            ids: vec![],
                            all: true,
                        },
                    })
                    .map_err(CtxError::from)
                    .and_then(|result| match result {
                        APIResult::Ok { result } => future::ok(result.0),
                        APIResult::Err { error } => future::err(CtxError::from(error)),
                    }),
                )
                .map_ok(move |(addons, library_items)| (auth, addons, library_items))
            })
            .map(enclose!((auth_request) move |result| {
                Msg::Internal(Internal::CtxAuthResult(auth_request, result))
            }))
            .boxed_env(),
    )
    .into()
}

fn delete_session<E: Env + 'static>(auth_key: &AuthKey) -> Effect {
    EffectFuture::Concurrent(
        E::flush_analytics()
            .then(enclose!((auth_key) move |_| {
                fetch_api::<E, _, _, SuccessResponse>(&APIRequest::Logout { auth_key })
            }))
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok { result } => future::ok(result),
                APIResult::Err { error } => future::err(CtxError::from(error)),
            })
            .map(enclose!((auth_key) move |result| match result {
                Ok(_) => Msg::Event(Event::SessionDeleted { auth_key }),
                Err(error) => Msg::Event(Event::Error {
                    error,
                    source: Box::new(Event::SessionDeleted { auth_key }),
                }),
            }))
            .boxed_env(),
    )
    .into()
}
