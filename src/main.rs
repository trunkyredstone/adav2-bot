use config::Config;
use serenity::model::guild::{AuditLogEntry, AuditLogs, Member};
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::user::User;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler;

struct BotConfig;

impl TypeMapKey for BotConfig {
    type Value = Config;
}

#[async_trait]
impl EventHandler for Handler {
    // For when a user joins the server
    async fn guild_member_addition(&self, ctx: Context, _gid: GuildId, new_member: Member) {
        println!("Event | Join");

        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let channel_id = ChannelId(settings.get("welcome").expect("Config is wrong"));

        if let Err(why) = channel_id
            .say(&ctx, format!("✅ {} has joined | <@{}>", new_member.user.name, new_member.user.id))
            .await
        {
            println!("Client error {:?}", why)
        }
    }

    // For when a user leaves the server
    async fn guild_member_removal(
        &self,
        ctx: Context,
        _: GuildId,
        member: User,
        _: Option<Member>,
    ) {
        println!("Event | Leave");

        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let channel_id = ChannelId(settings.get("welcome").expect("Config is wrong"));

        if let Err(why) = channel_id
            .say(&ctx, format!("❎ {} has left | <@{}>", member.name, member.id))
            .await
        {
            println!("Client error {:?}", why)
        }
    }

    // For when a user changes their nickname
    async fn guild_member_update(&self, ctx: Context, old: Option<Member>, new: Member) {
        println!("Event | Nickname change");

        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let channel_id = ChannelId(settings.get("welcome").expect("Config is wrong"));

        let old_nick = old.to_owned().expect("").nick;
        let new_nick = new.nick;
        let old_name = old.to_owned().expect("").user.name;
        let new_name = new.user.name;

        if old_nick.is_none() && new_nick.is_some() {
            if let Err(why) = channel_id
                .say(&ctx, format!("ℹ️ {} ➡ ️{} | <@{}>", old_name, new_nick.expect(""), new.user.id))
                .await
            {
                println!("Client error {:?}", why)
            }
        } else if old_nick.is_some() && new_nick.is_some() {
            if let Err(why) = channel_id
                .say(
                    &ctx,
                    format!("ℹ️ {} ➡ ️{} | <@{}>", old_nick.expect(""), new_nick.expect(""), new.user.id),
                )
                .await
            {
                println!("Client error {:?}", why)
            }
        } else if old_nick.is_some() && new_nick.is_none() {
            if let Err(why) = channel_id
                .say(&ctx, format!("ℹ️ {} ➡ ️{} | <@{}>", old_nick.expect(""), new_name, new.user.id))
                .await
            {
                println!("Client error {:?}", why)
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn message_delete(&self, ctx: Context, cid: ChannelId, msg: MessageId) {
        println!("Event | Message Deleted");


        let settings: Config = ctx
            .data
            .read()
            .await
            .get::<BotConfig>()
            .expect("Unable to find the config.")
            .clone();

        let to_ignore = settings
            .get_array("ignore_delete")
            .expect("Config is wrong");
        let to_ignore_iter = to_ignore.iter();
        for val in to_ignore_iter {
            if val.to_owned().into_str().expect("Config is wrong") == cid.as_u64().to_string() {
                println!("A message was deleted in a channel that should be ignored");
                return;
            }
        }

        let channel_id = ChannelId(settings.get("log").expect("Config is wrong"));
        let channel_id_self = ChannelId(settings.get("message-log").expect("Config is wrong"));

        let channel = cid.to_channel(&ctx).await.unwrap();
        let guild = channel.guild().unwrap();
        let gid = guild.guild_id;
        let audit_log: AuditLogs = gid
            .audit_logs(&ctx, Option::from(72_u8), None, None, Option::from(1_u8))
            .await
            .unwrap();
        let audit_entry: &AuditLogEntry = audit_log.entries.iter().next().unwrap().1;
        let user_id = audit_entry.user_id.as_u64();

        let message_maybe = ctx.cache.message(cid, msg).await;
        if message_maybe.clone().is_none() {
            if let Err(why) = channel_id.say(&ctx, format!("A message from someone was deleted in <#{}> by <@{}>, but it wasn't in the cache.",  cid.as_u64(), user_id)).await {
                    println!("Client error {:?}", why)
                }
            return;
        }

        let message = message_maybe.clone().unwrap();
        let msg_content = message.content;

        // println!("{}", &audit_entry.target_id.expect("Unable to grab a target ID from the audit log"));
        // println!("{}", message.author.id.as_u64());

        if &audit_entry
            .target_id
            .expect("Unable to grab a target ID from the audit log")
            != message.author.id.as_u64()
        {
            if let Err(why) = channel_id_self
                .say(
                    &ctx,
                    format!(
                        "<@{}> deleted their message in <#{}>: {}",
                        message.author.id.as_u64(),
                        cid.as_u64(),
                        msg_content
                    ),
                )
                .await
            {
                println!("Client error {:?}", why)
            }
        } else {
            if let Err(why) = channel_id
                .say(
                    &ctx,
                    format!(
                        "A message from <@{}> was deleted in <#{}> by possibly <@{}>: {}",
                        message.author.id.as_u64(),
                        cid.as_u64(),
                        user_id,
                        msg_content
                    ),
                )
                .await
            {
                println!("Client error {:?}", why)
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.cache.set_max_messages(1000).await;

        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let mut data = ctx.data.write().await;
        data.insert::<BotConfig>(settings);

        println!("{} is Ready", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    println!("ADAv2 -> V1.1.5");
    println!("ADAv2 -> Initialising");
    let mut settings = Config::default();
    settings.merge(config::File::with_name("Settings")).unwrap();

    let token: String = settings.get("token").unwrap();

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
