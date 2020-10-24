use config::Config;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use serenity::model::guild::{Member, AuditLogs, AuditLogEntry};
use serenity::model::user::User;
use serenity::model::id::{ChannelId, MessageId, GuildId};

struct Handler;

struct BotConfig;

impl TypeMapKey for BotConfig {
    type Value = Config;
}

#[async_trait]
impl EventHandler for Handler {
    // For when a user joins the server
    async fn guild_member_addition(&self, ctx: Context, _gid: GuildId, new_member: Member) {
        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let channel_id = ChannelId(settings.get("welcome").expect("Config is wrong"));

        if let Err(why) = channel_id.say(&ctx, format!("✅ {} has joined.", new_member.user.name)).await {
            println!("Client error {:?}", why)
        }

        println!("Event | Join");
    }

    // For when a user leaves the server
    async fn guild_member_removal(&self, ctx: Context, _: GuildId, member: User, _: Option<Member>) {
        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let channel_id = ChannelId(settings.get("welcome").expect("Config is wrong"));

        if let Err(why) = channel_id.say(&ctx, format!("❎ {} has left.", member.name)).await {
            println!("Client error {:?}", why)
        }

        println!("Event | Leave");
    }

    // For when a user changes their nickname
    async fn guild_member_update(&self, ctx: Context, old: Option<Member>, new: Member) {
        let mut settings = Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();

        let channel_id = ChannelId(settings.get("welcome").expect("Config is wrong"));

        let old_nick = old.to_owned().expect("").nick;
        let new_nick = new.nick;
        let old_name = old.to_owned().expect("").user.name;
        let new_name = new.user.name;

        if old_nick.is_none() && new_nick.is_some() {
            if let Err(why) = channel_id.say(&ctx, format!("ℹ️ {} ➡ ️{}", old_name, new_nick.expect(""))).await {
                println!("Client error {:?}", why)
            }
        } else if old_nick.is_some() && new_nick.is_some() {
            if let Err(why) = channel_id.say(&ctx, format!("ℹ️ {} ➡ ️{}", old_nick.expect(""), new_nick.expect(""))).await {
                println!("Client error {:?}", why)
            }
        } else if old_nick.is_some() && new_nick.is_none() {
            if let Err(why) = channel_id.say(&ctx, format!("ℹ️ {} ➡ ️{}", old_nick.expect(""), new_name)).await {
                println!("Client error {:?}", why)
            }
        }

        println!("Event | Nickname change");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn message_delete(&self, ctx: Context, cid: ChannelId, msg: MessageId) {
        let settings: Config = ctx.data.read().await.get::<BotConfig>().expect("Unable to find the config.").clone();

        let channel_id = ChannelId(settings.get("log").expect("Config is wrong"));

        let channel = cid.to_channel(&ctx).await.unwrap();
        let guild = channel.guild().unwrap();
        let gid = guild.guild_id;
        let audit_log: AuditLogs = gid.audit_logs(&ctx, Option::from(72_u8), None, None, Option::from(1_u8)).await.unwrap();
        let audit_entry: &AuditLogEntry = audit_log.entries.iter().next().unwrap().1;
        let user_id = audit_entry.user_id.as_u64();

        let message_maybe = ctx.cache.message(cid, msg).await;
        if message_maybe.clone().is_none() {
            if audit_entry.target_id.expect("") == msg.as_u64().clone() {
                if let Err(why) = channel_id.say(&ctx, format!("<@{}> deleted their message in <#{}>, but it wasn't in the cache.", user_id, cid.as_u64())).await {
                    println!("Client error {:?}", why)
                }
            } else {
                if let Err(why) = channel_id.say(&ctx, format!("A message from someone was deleted in <#{}> by <@{}>, but it wasn't in the cache.",  cid.as_u64(), user_id)).await {
                    println!("Client error {:?}", why)
                }
            }
            return
        }

        let message = message_maybe.clone().unwrap();
        let msg_content = message.content;

        if audit_entry.target_id.expect("") == msg.as_u64().clone() {
            if let Err(why) = channel_id.say(&ctx, format!("<@{}> deleted their message in <#{}>: {}", user_id, cid.as_u64(), msg_content)).await {
                println!("Client error {:?}", why)
            }
        } else {
            if let Err(why) = channel_id.say(&ctx, format!("A message from <@{}> was deleted in <#{}> by <@{}>: {}", message.author.id.as_u64(), cid.as_u64(), user_id, msg_content)).await {
                println!("Client error {:?}", why)
            }
        }

        println!("Event | Message Deleted");
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
    println!("ADAv2 -> Initialising");
    let mut settings = Config::default();
    settings
        .merge(config::File::with_name("Settings")).unwrap();

    let token : String = settings.get("token").unwrap();

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
