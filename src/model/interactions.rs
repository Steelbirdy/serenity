//! Interactions information-related models.

use bitflags::__impl_bitflags;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde_json::{Map, Number, Value};

use super::prelude::*;
use crate::builder::{
    CreateApplicationCommand,
    CreateApplicationCommands,
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    EditInteractionResponse,
};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::utils;

/// Information about an interaction.
///
/// An interaction is sent when a user invokes a slash command and is the same
/// for slash commands and other future interaction types.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct Interaction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The type of interaction.
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// The data of the command which was triggered, if there is one.
    ///
    /// **Note**: It is always present if the interaction [`kind`] is
    /// [`ApplicationCommand`].
    ///
    /// [`ApplicationCommand`]: self::InteractionType::ApplicationCommand
    /// [`kind`]: Interaction::kind
    pub data: Option<ApplicationCommandInteractionData>,
    /// The guild Id this interaction was sent from, if there is one.
    pub guild_id: Option<GuildId>,
    /// The channel Id this interaction was sent from, if there is one.
    pub channel_id: Option<ChannelId>,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    pub member: Option<Member>,
    /// The `user` object for the invoking user.
    ///
    /// It is only present if the interaction is triggered in DM.
    pub user: Option<User>,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("guild_id").and_then(|x| x.as_str()).and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(member) = map.get_mut("member").and_then(|x| x.as_object_mut()) {
                member.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
            }

            if let Some(data) = map.get_mut("data") {
                if let Some(resolved) = data.get_mut("resolved") {
                    if let Some(roles) = resolved.get_mut("roles") {
                        if let Some(values) = roles.as_object_mut() {
                            for value in values.values_mut() {
                                value.as_object_mut().unwrap().insert(
                                    "guild_id".to_string(),
                                    Value::String(guild_id.to_string()),
                                );
                            }
                        }
                    }

                    if let Some(channels) = resolved.get_mut("channels") {
                        if let Some(values) = channels.as_object_mut() {
                            for value in values.values_mut() {
                                value.as_object_mut().unwrap().insert(
                                    "guild_id".to_string(),
                                    Value::String(guild_id.to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }

        let id = map
            .remove("id")
            .ok_or_else(|| DeError::custom("expected id"))
            .and_then(InteractionId::deserialize)
            .map_err(DeError::custom)?;

        let application_id = map
            .remove("application_id")
            .ok_or_else(|| DeError::custom("expected application id"))
            .and_then(ApplicationId::deserialize)
            .map_err(DeError::custom)?;

        let kind = map
            .remove("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(InteractionType::deserialize)
            .map_err(DeError::custom)?;

        let data = match map.contains_key("data") {
            true => Some(
                map.remove("data")
                    .ok_or_else(|| DeError::custom("expected data"))
                    .and_then(ApplicationCommandInteractionData::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let guild_id = match map.contains_key("guild_id") {
            true => Some(
                map.remove("guild_id")
                    .ok_or_else(|| DeError::custom("expected guild_id"))
                    .and_then(GuildId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let channel_id = match map.contains_key("channel_id") {
            true => Some(
                map.remove("channel_id")
                    .ok_or_else(|| DeError::custom("expected channel_id"))
                    .and_then(ChannelId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let member = match map.contains_key("member") {
            true => Some(
                map.remove("member")
                    .ok_or_else(|| DeError::custom("expected member"))
                    .and_then(Member::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let user = match map.contains_key("user") {
            true => Some(
                map.remove("user")
                    .ok_or_else(|| DeError::custom("expected user"))
                    .and_then(User::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let token = map
            .remove("token")
            .ok_or_else(|| DeError::custom("expected token"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        let version = map
            .remove("version")
            .ok_or_else(|| DeError::custom("expected version"))
            .and_then(u8::deserialize)
            .map_err(DeError::custom)?;

        Ok(Self {
            id,
            application_id,
            kind,
            data,
            guild_id,
            channel_id,
            member,
            user,
            token,
            version,
        })
    }
}

/// The type of an Interaction
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    Unknown = !0,
}

enum_number!(InteractionType {
    Ping,
    ApplicationCommand
});

/// The command data payload.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteractionData {
    /// The Id of the invoked command.
    pub id: CommandId,
    /// The name of the invoked command.
    pub name: String,
    #[serde(default)]
    /// The parameters and the given values.
    pub options: Vec<ApplicationCommandInteractionDataOption>,
    #[serde(default)]
    /// The converted objects from the given options.
    pub resolved: ApplicationCommandInteractionDataResolved,
}

impl<'de> Deserialize<'de> for ApplicationCommandInteractionData {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let name = map
            .remove("name")
            .ok_or_else(|| DeError::custom("expected value"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        let id = map
            .remove("id")
            .ok_or_else(|| DeError::custom("expected value"))
            .and_then(CommandId::deserialize)
            .map_err(DeError::custom)?;

        let resolved = match map.contains_key("resolved") {
            true => map
                .remove("resolved")
                .ok_or_else(|| DeError::custom("expected resolved"))
                .and_then(ApplicationCommandInteractionDataResolved::deserialize)
                .map_err(DeError::custom)?,
            false => ApplicationCommandInteractionDataResolved::default(),
        };

        let options = match map.contains_key("options") {
            true => map
                .remove("options")
                .ok_or_else(|| DeError::custom("expected options"))
                .and_then(|deserializer| deserialize_options_with_resolved(deserializer, &resolved))
                .map_err(DeError::custom)?,
            false => vec![],
        };

        Ok(Self {
            name,
            id,
            options,
            resolved,
        })
    }
}

/// The resolved data of a command data interaction payload.
/// It contains the objects of [`ApplicationCommandInteractionDataOption`]s.
#[derive(Clone, Debug, Serialize, Default)]
#[non_exhaustive]
pub struct ApplicationCommandInteractionDataResolved {
    pub users: HashMap<UserId, User>,
    pub members: HashMap<UserId, PartialMember>,
    pub roles: HashMap<RoleId, Role>,
    pub channels: HashMap<ChannelId, PartialChannel>,
}

impl<'de> Deserialize<'de> for ApplicationCommandInteractionDataResolved {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let members = match map.contains_key("members") {
            true => map
                .remove("members")
                .ok_or_else(|| DeError::custom("expected members"))
                .and_then(deserialize_partial_members_map)
                .map_err(DeError::custom)?,
            false => HashMap::new(),
        };

        let users = match map.contains_key("users") {
            true => map
                .remove("users")
                .ok_or_else(|| DeError::custom("expected users"))
                .and_then(deserialize_users)
                .map_err(DeError::custom)?,
            false => HashMap::new(),
        };

        let roles = match map.contains_key("roles") {
            true => map
                .remove("roles")
                .ok_or_else(|| DeError::custom("expected roles"))
                .and_then(deserialize_roles_map)
                .map_err(DeError::custom)?,
            false => HashMap::new(),
        };

        let channels = match map.contains_key("channels") {
            true => map
                .remove("channels")
                .ok_or_else(|| DeError::custom("expected chanels"))
                .and_then(deserialize_channels_map)
                .map_err(DeError::custom)?,
            false => HashMap::new(),
        };

        Ok(Self {
            users,
            members,
            roles,
            channels,
        })
    }
}

/// A set of a parameter and a value from the user.
///
/// All options have names and an option can either be a parameter and input `value` or it can denote a sub-command or group, in which case it will contain a
/// top-level key and another vector of `options`.
///
/// Their resolved objects can be found on [`ApplicationCommandInteractionData::resolved`].
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteractionDataOption {
    /// The name of the parameter.
    pub name: String,
    /// The given value.
    pub value: Option<Value>,
    /// The value type.
    #[serde(rename = "type")]
    pub kind: ApplicationCommandOptionType,
    /// The nested options.
    ///
    /// **Note**: It is only present if the option is
    /// a group or a subcommand.
    #[serde(default)]
    pub options: Vec<ApplicationCommandInteractionDataOption>,
    /// The resolved object of the given `value`, if there is one.
    #[serde(default)]
    pub resolved: Option<ApplicationCommandInteractionDataOptionValue>,
}

impl<'de> Deserialize<'de> for ApplicationCommandInteractionDataOption {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let name = map
            .remove("name")
            .ok_or_else(|| DeError::custom("expected value"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        let value = match map.contains_key("value") {
            true => Some(
                map.remove("value")
                    .ok_or_else(|| DeError::custom("expected value"))
                    .and_then(Value::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let kind = map
            .remove("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(ApplicationCommandOptionType::deserialize)
            .map_err(DeError::custom)?;

        let options = match map.contains_key("options") {
            true => map
                .remove("options")
                .ok_or_else(|| DeError::custom("expected type"))
                .and_then(deserialize_options)
                .map_err(DeError::custom)?,
            false => vec![],
        };

        Ok(Self {
            name,
            value,
            kind,
            options,
            resolved: None,
        })
    }
}

/// The resolved value of an [`ApplicationCommandInteractionDataOption`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
#[repr(u8)]
pub enum ApplicationCommandInteractionDataOptionValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    User(User, Option<PartialMember>),
    Channel(PartialChannel),
    Role(Role),
}

fn default_permission_value() -> bool {
    true
}

/// The base command model that belongs to an application.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommand {
    /// The command Id.
    pub id: CommandId,
    /// The parent application Id.
    pub application_id: ApplicationId,
    /// The command name.
    pub name: String,
    /// The command description.
    pub description: String,
    /// The parameters for the command.
    #[serde(default)]
    pub options: Vec<ApplicationCommandOption>,
    /// Whether the command is enabled by default when
    /// the application is added to a guild.
    #[serde(default = "self::default_permission_value")]
    pub default_permission: bool,
}

impl ApplicationCommand {
    /// Creates a global [`ApplicationCommand`],
    /// overriding an existing one with the same name if it exists.
    ///
    /// When a created [`ApplicationCommand`] is used, the [`InteractionCreate`] event will be emitted.
    ///
    /// **Note**: Global commands may take up to an hour to become available.
    ///
    /// As such, it is recommended that guild application commands be used for testing purposes.
    ///
    /// # Examples
    ///
    /// Create a simple ping command
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::{interactions::ApplicationCommand, id::ApplicationId};
    ///
    /// let _ = ApplicationCommand::create_global_application_command(&http, |a| {
    ///    a.name("ping")
    ///     .description("A simple ping command")
    /// })
    /// .await;
    /// # }
    /// ```
    ///
    /// Create a command that echoes what is inserted
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::{
    ///     interactions::{ApplicationCommand, ApplicationCommandOptionType},
    ///     id::ApplicationId
    /// };
    ///
    /// let _ = ApplicationCommand::create_global_application_command(&http, |a| {
    ///    a.name("echo")
    ///     .description("What is said is echoed")
    ///     .create_option(|o| {
    ///         o.name("to_say")
    ///          .description("What will be echoed")
    ///          .kind(ApplicationCommandOptionType::String)
    ///          .required(true)
    ///     })
    /// })
    /// .await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the [`ApplicationCommand`] is illformed,
    /// such as if more than 10 [`choices`] are set. See the [API Docs] for further details.
    ///
    /// Can also return an [`Error::Json`] if there is an error in deserializing
    /// the response.
    ///
    /// [`ApplicationCommand`]: crate::model::interactions::ApplicationCommand
    /// [`InteractionCreate`]: crate::client::EventHandler::interaction_create
    /// [API Docs]: https://discord.com/developers/docs/interactions/slash-commands
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    /// [`choices`]: crate::model::interactions::ApplicationCommandOption::choices
    pub async fn create_global_application_command<F>(
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<ApplicationCommand>
    where
        F: FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
    {
        let map = ApplicationCommand::build_application_command(f);
        http.as_ref().create_global_application_command(&Value::Object(map)).await
    }

    /// Same as [`create_global_application_command`] but allows
    /// to create more than one global command per call.
    ///
    /// [`create_global_application_command`]: Self::create_global_application_command
    pub async fn create_global_application_commands<F>(
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<Vec<ApplicationCommand>>
    where
        F: FnOnce(&mut CreateApplicationCommands) -> &mut CreateApplicationCommands,
    {
        let mut array = CreateApplicationCommands::default();

        f(&mut array);

        http.as_ref().create_global_application_commands(&Value::Array(array.0)).await
    }

    /// Edits a global command by its Id.
    pub async fn edit_global_application_command<F>(
        http: impl AsRef<Http>,
        command_id: CommandId,
        f: F,
    ) -> Result<ApplicationCommand>
    where
        F: FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
    {
        let map = ApplicationCommand::build_application_command(f);
        http.as_ref().edit_global_application_command(command_id.into(), &Value::Object(map)).await
    }

    /// Gets all global commands.
    pub async fn get_global_application_commands(
        http: impl AsRef<Http>,
    ) -> Result<Vec<ApplicationCommand>> {
        http.as_ref().get_global_application_commands().await
    }

    /// Gets a global command by its Id.
    pub async fn get_global_application_command(
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<ApplicationCommand> {
        http.as_ref().get_global_application_command(command_id.into()).await
    }

    /// Deletes a global command by its Id.
    pub async fn delete_global_application_command(
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<()> {
        http.as_ref().delete_global_application_command(command_id.into()).await
    }

    #[inline]
    pub(crate) fn build_application_command<F>(f: F) -> Map<String, Value>
    where
        F: FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
    {
        let mut create_application_command = CreateApplicationCommand::default();
        f(&mut create_application_command);
        utils::hashmap_to_json_map(create_application_command.0)
    }
}

/// The parameters for an [`ApplicationCommand`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandOption {
    /// The option type.
    #[serde(rename = "type")]
    pub kind: ApplicationCommandOptionType,
    /// The option name.
    pub name: String,
    /// The option description.
    pub description: String,
    /// Whether the parameter is optional or required.
    #[serde(default)]
    pub required: bool,
    /// Choices the user can pick from.
    ///
    /// **Note**: Only available for [`String`] and [`Integer`] [`ApplicationCommandOptionType`].
    ///
    /// [`String`]: ApplicationCommandOptionType::String
    /// [`Integer`]: ApplicationCommandOptionType::Integer
    #[serde(default)]
    pub choices: Vec<ApplicationCommandOptionChoice>,
    /// The nested options.
    ///
    /// **Note**: Only available for [`SubCommand`] or [`SubCommandGroup`].
    ///
    /// [`SubCommand`]: ApplicationCommandOptionType::SubCommand
    /// [`SubCommandGroup`]: ApplicationCommandOptionType::SubCommandGroup
    #[serde(default)]
    pub options: Vec<ApplicationCommandOption>,
}

/// An [`ApplicationCommand`] permission.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandPermission {
    /// The id of the command.
    pub id: CommandId,
    /// The id of the application the command belongs to.
    pub application_id: ApplicationId,
    /// The id of the guild.
    pub guild_id: GuildId,
    /// The permissions for the command in the guild.
    pub permissions: Vec<ApplicationCommandPermissionData>,
}

/// The [`ApplicationCommandPermission`] data.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandPermissionData {
    /// The [`RoleId`] or [`UserId`], depends on `kind` value.
    ///
    /// [`RoleId`]: crate::model::id::RoleId
    /// [`UserId`]: crate::model::id::UserId
    pub id: CommandPermissionId,
    /// The type of data this permissions applies to.
    #[serde(rename = "type")]
    pub kind: ApplicationCommandPermissionType,
    /// Whether or not the provided data can use the command or not.
    pub permission: bool,
}

/// The type of an [`ApplicationCommandOption`].
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ApplicationCommandOptionType {
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Integer = 4,
    Boolean = 5,
    User = 6,
    Channel = 7,
    Role = 8,
    Unknown = !0,
}

enum_number!(ApplicationCommandOptionType {
    SubCommand,
    SubCommandGroup,
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
});

/// The type of an [`ApplicationCommandPermissionData`].
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ApplicationCommandPermissionType {
    Role = 1,
    User = 2,
    Unknown = !0,
}

enum_number!(ApplicationCommandPermissionType {
    Role,
    User
});

/// The only valid values a user can pick in an [`ApplicationCommandOption`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandOptionChoice {
    /// The choice name.
    pub name: String,
    /// The choice value.
    pub value: Value,
}

/// The available responses types for an interaction response.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
}

/// The flags for an interaction response.
#[derive(Clone, Serialize)]
#[non_exhaustive]
pub struct InteractionApplicationCommandCallbackDataFlags {
    bits: u64,
}

__impl_bitflags! {
    InteractionApplicationCommandCallbackDataFlags: u64 {
        /// Interaction message will only be visible to sender and will
        /// be quickly deleted.
        EPHEMERAL = 0b0000_0000_0000_0000_0000_0000_0100_0000;
    }
}

/// Sent when a [`Message`] is a response to an [`Interaction`].
///
/// [`Message`]: crate::model::channel::Message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageInteraction {
    /// The id of the interaction.
    pub id: InteractionId,
    /// The type of the interaction.
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// The name of the [`ApplicationCommand`].
    pub name: String,
    /// The user who invoked the interaction.
    pub user: User,
}

impl Interaction {
    /// Creates a response to the interaction received.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long.
    /// May also return an [`Error::Http`] if the API returns an error,
    /// or an [`Error::Json`] if there is an error in deserializing the
    /// API response.
    ///
    /// # Errors
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn create_interaction_response<F>(&self, http: impl AsRef<Http>, f: F) -> Result<()>
    where
        F: FnOnce(&mut CreateInteractionResponse) -> &mut CreateInteractionResponse,
    {
        let mut interaction_response = CreateInteractionResponse::default();
        f(&mut interaction_response);

        let map = utils::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref().create_interaction_response(self.id.0, &self.token, &Value::Object(map)).await
    }

    /// Edits the initial interaction response.
    ///
    /// `application_id` will usually be the bot's [`UserId`], except in cases of bots being very old.
    ///
    /// Refer to Discord's docs for Edit Webhook Message for field information.
    ///
    /// **Note**:   Message contents must be under 2000 unicode code points, does not work on ephemeral messages.
    ///
    /// [`UserId`]: crate::model::id::UserId
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the edited content is too long.
    /// May also return [`Error::Http`] if the API returns an error,
    /// or an [`Error::Json`] if there is an error deserializing the response.
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn edit_original_interaction_response<F>(
        &self,
        http: impl AsRef<Http>,
        application_id: u64,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse,
    {
        let mut interaction_response = EditInteractionResponse::default();
        f(&mut interaction_response);

        let map = utils::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref()
            .edit_original_interaction_response(application_id, &self.token, &Value::Object(map))
            .await
    }

    /// Deletes the initial interaction response.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was already deleted.
    pub async fn delete_original_interaction_response(
        &self,
        http: impl AsRef<Http>,
        application_id: u64,
    ) -> Result<()> {
        http.as_ref().delete_original_interaction_response(application_id, &self.token).await
    }

    /// Creates a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Will return [`Error::Model`] if the content is too long.
    /// May also return [`Error::Http`] if the API returns an error,
    /// or a [`Error::Json`] if there is an error in deserializing the response.
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn create_followup_message<'a, F>(
        &self,
        http: impl AsRef<Http>,
        application_id: u64,
        wait: bool,
        f: F,
    ) -> Result<Option<Message>>
    where
        for<'b> F: FnOnce(
            &'b mut CreateInteractionResponseFollowup<'a>,
        ) -> &'b mut CreateInteractionResponseFollowup<'a>,
    {
        let mut interaction_response = CreateInteractionResponseFollowup::default();
        f(&mut interaction_response);

        let map = utils::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref().create_followup_message(application_id, &self.token, wait, &map).await
    }
}

impl CommandPermissionId {
    /// Converts this [`CommandPermissionId`] to [`UserId`].
    pub fn to_user_id(self) -> UserId {
        UserId(self.0)
    }

    /// Converts this [`CommandPermissionId`] to [`RoleId`].
    pub fn to_role_id(self) -> RoleId {
        RoleId(self.0)
    }
}

impl From<RoleId> for CommandPermissionId {
    fn from(id: RoleId) -> Self {
        Self(id.0)
    }
}

impl From<UserId> for CommandPermissionId {
    fn from(id: UserId) -> Self {
        Self(id.0)
    }
}

impl From<CommandPermissionId> for RoleId {
    fn from(id: CommandPermissionId) -> Self {
        Self(id.0)
    }
}

impl From<CommandPermissionId> for UserId {
    fn from(id: CommandPermissionId) -> Self {
        Self(id.0)
    }
}
