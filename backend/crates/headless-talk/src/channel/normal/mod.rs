pub mod user;

use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use talk_loco_client::talk::{
    channel::ChannelMetaType, session::channel::chat_on::NormalChatOnChannel,
};

use crate::{
    conn::Conn,
    database::{
        model::{
            channel::ChannelListRow,
            user::{
                normal::{NormalChannelUserModel, NormalChannelUserUpdate},
                UserProfileModel, UserProfileUpdate,
            },
        },
        schema::{channel_meta, normal_channel_user, user_profile},
        DatabasePool, PoolTaskError,
    },
    user::DisplayUser,
    ClientResult, HeadlessTalk,
};

use self::user::NormalChannelUser;

use super::ListChannelProfile;

#[derive(Debug)]
pub struct NormalChannel {
    id: i64,
    pub(super) conn: Conn,
}

impl NormalChannel {
    pub const fn id(&self) -> i64 {
        self.id
    }

    pub async fn users(&self) -> Result<Vec<(i64, NormalChannelUser)>, PoolTaskError> {
        let users = self
            .conn
            .pool
            .spawn({
                let id = self.id;

                move |conn| {
                    let users: Vec<(i64, NormalChannelUser)> = user_profile::table
                        .inner_join(
                            normal_channel_user::table.on(normal_channel_user::channel_id
                                .eq(user_profile::channel_id)
                                .and(normal_channel_user::id.eq(user_profile::id))),
                        )
                        .filter(user_profile::channel_id.eq(id))
                        .select((
                            user_profile::id,
                            UserProfileModel::as_select(),
                            NormalChannelUserModel::as_select(),
                        ))
                        .load_iter::<(i64, UserProfileModel, NormalChannelUserModel), _>(conn)?
                        .map(|res| {
                            res.map(|(user_id, profile, normal)| {
                                (user_id, NormalChannelUser::from_models(profile, normal))
                            })
                        })
                        .collect::<Result<_, _>>()?;

                    Ok(users)
                }
            })
            .await?;

        Ok(users)
    }
}

pub(super) async fn load_list_profile(
    pool: &DatabasePool,
    display_users: &[DisplayUser],
    row: &ChannelListRow,
) -> Result<ListChannelProfile, PoolTaskError> {
    let id = row.id;

    let (name, image_url) = pool
        .spawn(move |conn| {
            let name: Option<String> = channel_meta::table
                .filter(
                    channel_meta::channel_id
                        .eq(id)
                        .and(channel_meta::type_.eq(ChannelMetaType::Title as i32)),
                )
                .select(channel_meta::content)
                .first(conn)
                .optional()?;

            let image_url: Option<String> = channel_meta::table
                .filter(
                    channel_meta::channel_id
                        .eq(id)
                        .and(channel_meta::type_.eq(ChannelMetaType::Profile as i32)),
                )
                .select(channel_meta::content)
                .first(conn)
                .optional()?;

            Ok((name, image_url))
        })
        .await?;

    let name = name.unwrap_or_else(|| {
        display_users
            .iter()
            .map(|user| user.profile.nickname.as_str())
            .collect::<Vec<&str>>()
            .join(", ")
    });

    Ok(ListChannelProfile { name, image_url })
}

pub(crate) async fn open_channel(
    id: i64,
    client: &HeadlessTalk,
    normal: NormalChatOnChannel,
) -> ClientResult<NormalChannel> {
    if let Some(users) = normal.users {
        client
            .conn
            .pool
            .spawn(move |conn| {
                conn.transaction(move |conn| {
                    for user in &users {
                        diesel::update(user_profile::table)
                            .filter(
                                user_profile::id
                                    .eq(user.user_id)
                                    .and(user_profile::channel_id.eq(id)),
                            )
                            .set(UserProfileUpdate {
                                nickname: &user.nickname,
                                profile_url: &user.profile_image_url,
                                full_profile_url: &user.full_profile_image_url,
                                original_profile_url: &user.original_profile_image_url,
                            })
                            .execute(conn)?;

                        diesel::update(normal_channel_user::table)
                            .filter(
                                normal_channel_user::id
                                    .eq(user.user_id)
                                    .and(normal_channel_user::channel_id.eq(id)),
                            )
                            .set(NormalChannelUserUpdate {
                                country_iso: &user.country_iso,
                                account_id: user.account_id,
                                status_message: &user.status_message,
                                linked_services: &user.linked_services,
                                suspended: user.suspended,
                            })
                            .execute(conn)?;
                    }

                    Ok(())
                })
            })
            .await?;
    }

    Ok(NormalChannel {
        id,
        conn: client.conn.clone(),
    })
}
